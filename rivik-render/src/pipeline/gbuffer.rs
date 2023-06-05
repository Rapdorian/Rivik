/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Utilities for working with the G-buffer
use std::borrow::Cow;

use crate::{context::device, sampler};
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindingType, BlendState,
    ColorTargetState, ColorWrites, CommandEncoder, DepthStencilState, Extent3d, LoadOp,
    RenderBundleDepthStencil, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPipeline, SamplerBindingType, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    VertexBufferLayout,
};

/// The G-buffer
pub struct GBuffer {
    pub(crate) color_view: TextureView,
    pub(crate) pos_view: TextureView,
    pub(crate) norm_view: TextureView,
    pub(crate) lum_view: TextureView,
    pub(crate) depth_view: TextureView,
    pub(crate) hdr_view: TextureView,
    pub(crate) bind_group: BindGroup,
    pub(crate) hdr_bind: BindGroup,
    pub(crate) layout: BindGroupLayout,
}

impl GBuffer {
    /// Creates a new [`GBuffer`].
    ///
    /// # Panics
    ///
    /// Panics if the renderer device hasn't been initialized
    pub fn new(width: u32, height: u32) -> Self {
        let device = device();
        // create textures
        let dimensions = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let color_tex = device.create_texture(&TextureDescriptor {
            label: Some("Diffuse GBuffer"),
            size: dimensions,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let pos_tex = device.create_texture(&TextureDescriptor {
            label: Some("Position GBuffer"),
            size: dimensions,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let norm_tex = device.create_texture(&TextureDescriptor {
            label: Some("Normal GBuffer"),
            size: dimensions,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let lum_tex = device.create_texture(&TextureDescriptor {
            label: Some("Luminance GBuffer"),
            size: dimensions,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let hdr_tex = device.create_texture(&TextureDescriptor {
            label: Some("HDR GBuffer"),
            size: dimensions,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let depth_tex = device.create_texture(&TextureDescriptor {
            label: Some("Depth GBuffer"),
            size: dimensions,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        });

        let layout = device.create_bind_group_layout(&Self::layout());

        let color_view = color_tex.create_view(&TextureViewDescriptor::default());
        let pos_view = pos_tex.create_view(&TextureViewDescriptor::default());
        let norm_view = norm_tex.create_view(&TextureViewDescriptor::default());
        let lum_view = lum_tex.create_view(&TextureViewDescriptor::default());
        let hdr_view = hdr_tex.create_view(&TextureViewDescriptor::default());
        let depth_view = depth_tex.create_view(&TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GBuffer"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler::DEFAULT),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&pos_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&norm_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&lum_view),
                },
            ],
        });

        let hdr_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("HDR Bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let hdr_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GBuffer HDR binding"),
            layout: &hdr_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler::DEFAULT),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
            ],
        });
        Self {
            hdr_bind,
            bind_group,
            layout,
            color_view,
            pos_view,
            hdr_view,
            depth_view,
            lum_view,
            norm_view,
        }
    }

    /// Formats of the g-buffer's input buffers
    pub fn color_formats() -> &'static [Option<TextureFormat>] {
        &[
            Some(TextureFormat::Rgba16Float),
            Some(TextureFormat::Rgba16Float),
            Some(TextureFormat::Rgba16Float),
            Some(TextureFormat::Rgba16Float),
        ]
    }

    /// The format of the HDR buffer
    pub fn hdr_format() -> TextureFormat {
        TextureFormat::Rgba16Float
    }

    /// The depth format used by the g-buffer
    pub fn depth_format() -> Option<RenderBundleDepthStencil> {
        Some(RenderBundleDepthStencil {
            format: TextureFormat::Depth24Plus,
            depth_read_only: false,
            stencil_read_only: false,
        })
    }

    /// The g-buffer's render target
    pub const TARGETS: &[Option<ColorTargetState>] = &[
        Some(ColorTargetState {
            format: TextureFormat::Rgba16Float,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        }),
        Some(ColorTargetState {
            format: TextureFormat::Rgba16Float,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        }),
        Some(ColorTargetState {
            format: TextureFormat::Rgba16Float,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        }),
        Some(ColorTargetState {
            format: TextureFormat::Rgba16Float,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        }),
    ];

    /// Create a pipeline for rendering to the g-buffer without setting the depth buffer
    ///
    /// useful for rendering skyboxes and other objects that are infinitely far from the camera
    pub fn geom_no_depth_pipeline(
        shader: &str,
        bind_groups: &[&BindGroupLayout],
        vertex: VertexBufferLayout,
    ) -> RenderPipeline {
        let shader = device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader)),
        });

        let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: bind_groups,
            push_constant_ranges: &[],
        });

        device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex],
            },
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: GBuffer::TARGETS,
            }),
            multiview: None,
        })
    }

    /// Create a pipeline for rendering geometry to the g-buffer
    pub fn geom_pipeline(
        shader: &str,
        bind_groups: &[&BindGroupLayout],
        vertex: VertexBufferLayout,
    ) -> RenderPipeline {
        let shader = device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader)),
        });

        let layout = device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: bind_groups,
            push_constant_ranges: &[],
        });

        device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex],
            },
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: GBuffer::TARGETS,
            }),
            multiview: None,
        })
    }

    pub(crate) fn rpass<'buffer, 'frame>(
        &'buffer self,
        label: Option<&'frame str>,
        encoder: &'frame mut CommandEncoder,
        clear: Option<wgpu::Color>,
    ) -> RenderPass<'frame>
    where
        'buffer: 'frame,
    {
        let load = if let Some(color) = clear {
            LoadOp::Clear(color)
        } else {
            LoadOp::Load
        };

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label,
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &self.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: true },
                }),
                Some(RenderPassColorAttachment {
                    view: &self.pos_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: true },
                }),
                Some(RenderPassColorAttachment {
                    view: &self.norm_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: true },
                }),
                Some(RenderPassColorAttachment {
                    view: &self.lum_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: true },
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: if clear.is_some() {
                        LoadOp::Clear(1.0)
                    } else {
                        LoadOp::Load
                    },
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }

    fn layout() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("GBuffer"),
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // color
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // position
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // normal
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // luminance
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        }
    }
}
