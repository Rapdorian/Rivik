use std::{
    mem::{self, ManuallyDrop},
    num::NonZeroU64,
};

use glam::{Mat3, Mat4, Vec3, Vec4};
use mint::{ColumnMatrix4, Quaternion, Vector3};
use snafu::{Backtrace, ResultExt, Snafu};
use types::{
    egui::{ClippedPrimitive, TexturesDelta},
    light::Light,
    Model,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    SurfaceError,
};

use crate::{
    context::{device, queue, surface, surface_config},
    gbuffer::{self, HDR},
    inputs::{material, sampler, sun_light, transform},
    pipes::{ambient, mesh::MESH_PIPE, sun},
    types,
};

/// An error constructing a frame
#[derive(Debug, Snafu)]
#[snafu(display("Failed to create the next frame"))]
pub struct FrameError {
    source: SurfaceError,
    backtrace: Backtrace,
}

pub struct Frame<'a> {
    vertex_buffers: &'a [wgpu::Buffer],
    index_buffers: &'a [(wgpu::Buffer, usize)],
    textures: &'a [(wgpu::Texture, wgpu::TextureView)],

    // Store render info
    meshes: Vec<(&'a Model<u32>, Mat4)>,
    lights: Vec<Light>,
    camera: Mat4,
}

impl<'a> Frame<'a> {
    /// Try to fetch a new frame
    /// The frame will be rendered and presented when this object is dropped
    pub fn new(
        vertex_buffers: &'a [wgpu::Buffer],
        index_buffers: &'a [(wgpu::Buffer, usize)],
        textures: &'a [(wgpu::Texture, wgpu::TextureView)],
    ) -> Result<Frame<'a>, FrameError> {
        //TODO: We might be able to delay fetching the frame texture until the actually drawing

        Ok(Frame {
            // pre allocate a bunch of space for storing meshes
            meshes: Vec::with_capacity(100),
            lights: Vec::with_capacity(100),
            camera: Mat4::default(),
            vertex_buffers,
            index_buffers,
            textures,
        })
    }
}

impl<'a> Drop for Frame<'a> {
    // TODO: Break some of this into helper methods
    fn drop(&mut self) {
        // Apply the render pipeline here

        // generate uniform data for rendering meshes
        let aspect_ratio = {
            let surface = surface_config();
            surface.width as f32 / surface.height as f32
        };
        let proj = Mat4::perspective_rh(3.14 / 2.0, aspect_ratio, 0.1, 1_000.0);
        const UBO_SIZE: usize = mem::size_of::<Mat4>() * 3;
        let mut buffer = Vec::with_capacity(self.meshes.len() * UBO_SIZE);
        for mesh in &self.meshes {
            // push transform into buffer
            let transform = proj * self.camera * mesh.1;
            let view = self.camera * mesh.1;
            let norm = Mat4::from_mat3(Mat3::from_mat4(
                (self.camera * mesh.1).inverse().transpose(),
            ));
            buffer.extend_from_slice(bytemuck::bytes_of(&transform));
            buffer.extend_from_slice(bytemuck::bytes_of(&view));
            buffer.extend_from_slice(bytemuck::bytes_of(&norm));
        }

        // Prevent a panic if we are drawing nothing
        if buffer.len() < UBO_SIZE {
            buffer.resize(UBO_SIZE, 0);
        }

        // create a buffer for storing Uniforms
        let ubo = device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: &buffer,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let ubo = device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform Bind"),
            layout: &*transform::LAYOUT,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &ubo,
                    offset: 0,
                    size: NonZeroU64::new(UBO_SIZE as u64),
                }),
            }],
        });

        // generate light buffers
        let mut buffer = Vec::with_capacity((self.lights.len() * 4 * 8) + mem::size_of::<Mat4>());
        buffer.extend_from_slice(bytemuck::cast_slice(bytemuck::bytes_of(&proj.inverse())));
        for light in &self.lights {
            match light {
                // Ambient lights are'nt being draw with this buffer but have to be present to keep
                // the index in sync
                Light::Ambient { color } => buffer.extend_from_slice(&[0.0; 6]),
                Light::Directional { color, direction } => {
                    buffer.extend_from_slice(color);
                    buffer.push(0.0);
                    // direction needs to be moved into view space
                    let dir =
                        self.camera * Vec4::new(direction[0], direction[1], direction[2], 0.0);
                    buffer.extend_from_slice(&dir.to_array());
                    buffer.push(0.0);
                }
                Light::Point { color, position } => {
                    buffer.extend_from_slice(color);
                    buffer.push(0.0);
                    // transform into view space
                    let pos = self.camera * Vec4::new(position[0], position[1], position[2], 1.0);
                    buffer.extend_from_slice(&pos.to_array());
                    buffer.push(0.0);
                }
            }
        }

        let buffer = bytemuck::cast_slice(buffer.as_slice());

        // create gpu buffer
        let light_list = device().create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: &buffer,
            usage: wgpu::BufferUsages::STORAGE,
        });
        let light_list = device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind"),
            layout: &*sun_light::LAYOUT,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &light_list,
                    offset: 0,
                    size: NonZeroU64::new(buffer.len() as u64),
                }),
            }],
        });

        // get next frame
        let frame = surface().get_current_texture().unwrap();
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // build materials
        let mut materials = Vec::with_capacity(self.meshes.len());
        for mesh in self.meshes.iter() {
            materials.push(material::bind(self.textures, &mesh.0));
        }
        // create render pass for drawing to GBuffer
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Geometry pass"),
                color_attachments: &gbuffer::color_attachments(Some(wgpu::Color::BLACK)),
                depth_stencil_attachment: Some(gbuffer::depth_attachment(true)),
            });

            // draw meshes
            rpass.set_pipeline(&*MESH_PIPE);
            rpass.set_bind_group(0, &*sampler::BIND, &[]);

            for (i, (mesh, _)) in self.meshes.iter().enumerate() {
                // issue draw commands for a mesh
                rpass.set_bind_group(1, &ubo, &[(mem::size_of::<Mat4>() * i) as u32]);
                rpass.set_bind_group(2, &materials[i], &[]);
                rpass.set_vertex_buffer(0, self.vertex_buffers[mesh.mesh as usize].slice(..));
                let (idx_buffer, idx_len) = &self.index_buffers[mesh.mesh as usize];
                rpass.set_index_buffer(idx_buffer.slice(..), wgpu::IndexFormat::Uint16);
                rpass.draw_indexed(0..*idx_len as u32, 0, 0..1);
            }
        }
        // Draw to screen
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("HDR pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &*HDR,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // draw lights
            rpass.set_pipeline(&*sun::PIPE_HDR);
            for (i, _) in self.lights.iter().enumerate() {
                rpass.set_bind_group(0, &*sampler::BIND, &[]);
                rpass.set_bind_group(1, &*gbuffer::BIND, &[]);
                rpass.set_bind_group(2, &light_list, &[]);
                rpass.draw(0..7, 0..1);
            }
        }

        // Draw to screen
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Final pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let ambient: f32 = 0.1;

            // ambient
            rpass.set_pipeline(&*ambient::PIPE);
            rpass.set_bind_group(0, &*sampler::BIND, &[]);
            rpass.set_bind_group(1, &*gbuffer::BIND, &[]);
            rpass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::bytes_of(&[ambient; 4]),
            );
            //rpass.draw(0..7, 0..1);

            // draw lights
            rpass.set_pipeline(&*sun::PIPE);
            for (i, _) in self.lights.iter().enumerate() {
                rpass.set_bind_group(0, &*sampler::BIND, &[]);
                rpass.set_bind_group(1, &*gbuffer::BIND, &[]);
                rpass.set_bind_group(2, &light_list, &[]);
                rpass.draw(0..7, 0..1);
            }
        }

        let _ = queue().submit(Some(encoder.finish()));
        frame.present();
    }
}

impl<'a> types::Frame<'a> for Frame<'a> {
    type ID = u32;

    fn draw_mesh(
        &mut self,
        model: &'a types::Model<Self::ID>,
        pos: impl Into<Vector3<f32>>,
        rot: impl Into<Quaternion<f32>>,
        scale: impl Into<Vector3<f32>>,
    ) {
        // create transform
        let mat = Mat4::from_scale_rotation_translation(
            scale.into().into(),
            rot.into().into(),
            pos.into().into(),
        );
        self.meshes.push((model, mat));
    }

    fn draw_light(&mut self, light: types::light::Light) {
        self.lights.push(light);
    }

    fn set_camera(&mut self, cam: impl Into<ColumnMatrix4<f32>>) {
        self.camera = cam.into().into();
    }

    fn set_ui<'b>(&'b mut self, clip_prim: &'b [ClippedPrimitive], textures: TexturesDelta) {
        todo!("egui rendering")
    }
}
