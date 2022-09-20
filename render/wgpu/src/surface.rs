use crate::commands::CommandRenderer;
use crate::frame::Frame;
use crate::mesh::Mesh;
use crate::uniform_buffer::BufferStorage;
use crate::{Descriptors, Globals, Pipelines, Transforms, UniformBuffer};
use ruffle_render::commands::CommandList;
use std::sync::Arc;

#[derive(Debug)]
pub struct FrameBuffer {
    view: wgpu::TextureView,
}

impl FrameBuffer {
    pub fn new(
        device: &wgpu::Device,
        msaa_sample_count: u32,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: create_debug_label!("Framebuffer texture").as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&Default::default());
        Self { view }
    }
}

#[derive(Debug)]
pub struct DepthTexture {
    view: wgpu::TextureView,
}

impl DepthTexture {
    pub fn new(device: &wgpu::Device, msaa_sample_count: u32, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: create_debug_label!("Depth texture").as_deref(),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: msaa_sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let view = texture.create_view(&Default::default());
        Self { view }
    }
}

#[derive(Debug)]
pub enum Surface {
    Direct {
        depth: DepthTexture,
        pipelines: Arc<Pipelines>,
    },
    Resolve {
        frame_buffer: FrameBuffer,
        depth: DepthTexture,
        pipelines: Arc<Pipelines>,
    },
}

impl Surface {
    pub fn new(
        descriptors: &Descriptors,
        msaa_sample_count: u32,
        width: u32,
        height: u32,
        frame_buffer_format: wgpu::TextureFormat,
    ) -> Self {
        let frame_buffer = if msaa_sample_count > 1 {
            Some(FrameBuffer::new(
                &descriptors.device,
                msaa_sample_count,
                width,
                height,
                frame_buffer_format,
            ))
        } else {
            None
        };

        let depth = DepthTexture::new(&descriptors.device, msaa_sample_count, width, height);
        let pipelines = descriptors.pipelines(msaa_sample_count, frame_buffer_format);

        match frame_buffer {
            Some(frame_buffer) => Surface::Resolve {
                frame_buffer,
                depth,
                pipelines,
            },
            None => Surface::Direct { depth, pipelines },
        }
    }

    pub fn view<'a>(&'a self, frame: &'a wgpu::TextureView) -> &wgpu::TextureView {
        match self {
            Surface::Direct { .. } => frame,
            Surface::Resolve { frame_buffer, .. } => &frame_buffer.view,
        }
    }

    pub fn resolve_target<'a>(
        &'a self,
        frame: &'a wgpu::TextureView,
    ) -> Option<&wgpu::TextureView> {
        match self {
            Surface::Direct { .. } => None,
            Surface::Resolve { .. } => Some(&frame),
        }
    }

    pub fn depth(&self) -> &wgpu::TextureView {
        match self {
            Surface::Direct { depth, .. } => &depth.view,
            Surface::Resolve { depth, .. } => &depth.view,
        }
    }

    pub fn pipelines(&self) -> &Pipelines {
        match self {
            Surface::Direct { pipelines, .. } => pipelines,
            Surface::Resolve { pipelines, .. } => pipelines,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_commands(
        &self,
        frame_view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
        descriptors: &Descriptors,
        globals: &mut Globals,
        uniform_buffers_storage: &mut BufferStorage<Transforms>,
        meshes: &Vec<Mesh>,
        commands: CommandList,
    ) -> Vec<wgpu::CommandBuffer> {
        let label = create_debug_label!("Draw encoder");
        let mut draw_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: label.as_deref(),
                });

        let uniform_encoder_label = create_debug_label!("Uniform upload command encoder");
        let mut uniform_encoder =
            descriptors
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: uniform_encoder_label.as_deref(),
                });

        globals.update_uniform(&descriptors.device, &mut draw_encoder);

        let load = match clear_color {
            Some(color) => wgpu::LoadOp::Clear(color),
            None => wgpu::LoadOp::Load,
        };

        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.view(frame_view),
                ops: wgpu::Operations { load, store: true },
                resolve_target: self.resolve_target(frame_view),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: false,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
            label: None,
        });
        render_pass.set_bind_group(0, globals.bind_group(), &[]);

        uniform_buffers_storage.recall();
        let mut frame = Frame::new(
            &self.pipelines(),
            &descriptors,
            UniformBuffer::new(uniform_buffers_storage),
            render_pass,
            &mut uniform_encoder,
        );
        commands.execute(&mut CommandRenderer::new(
            &mut frame,
            meshes,
            descriptors.quad.vertices.slice(..),
            descriptors.quad.indices.slice(..),
        ));
        frame.finish();

        vec![uniform_encoder.finish(), draw_encoder.finish()]
    }
}
