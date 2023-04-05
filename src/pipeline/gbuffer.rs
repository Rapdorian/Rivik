use crate::{context::device, frame::Frame};
use wgpu::{
    BindGroupLayoutDescriptor, BindingType, CommandEncoder, Extent3d, LoadOp,
    RenderBundleDepthStencil, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, SamplerBindingType, SamplerDescriptor, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

pub struct GBuffer {
    pub(crate) color_view: TextureView,
    pub(crate) pos_view: TextureView,
    pub(crate) norm_view: TextureView,
    pub(crate) depth_view: TextureView,
    pub(crate) hdr_view: TextureView,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) layout: wgpu::BindGroupLayout,
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
        let hdr_view = hdr_tex.create_view(&TextureViewDescriptor::default());
        let depth_view = depth_tex.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GBuffer"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
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
            ],
        });
        Self {
            bind_group,
            layout,
            color_view,
            pos_view,
            hdr_view,
            depth_view,
            norm_view,
        }
    }

    pub fn color_formats() -> &'static [Option<TextureFormat>] {
        &[
            Some(TextureFormat::Rgba16Float),
            Some(TextureFormat::Rgba16Float),
            Some(TextureFormat::Rgba16Float),
        ]
    }

    pub fn hdr_format() -> TextureFormat {
        TextureFormat::Rgba16Float
    }

    pub fn depth_format() -> Option<RenderBundleDepthStencil> {
        Some(RenderBundleDepthStencil {
            format: TextureFormat::Depth24Plus,
            depth_read_only: false,
            stencil_read_only: false,
        })
    }

    pub(crate) fn rpass<'a>(
        &'a self,
        encoder: &'a mut CommandEncoder,
        clear: Option<wgpu::Color>,
    ) -> RenderPass<'a> {
        let load = if let Some(color) = clear {
            LoadOp::Clear(color)
        } else {
            LoadOp::Load
        };

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GBuffer Render pass"),
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
                // position
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
            ],
        }
    }
}
