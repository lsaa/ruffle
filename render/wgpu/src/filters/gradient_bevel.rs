use crate::backend::RenderTargetMode;
use crate::buffer_pool::TexturePool;
use crate::descriptors::Descriptors;
use crate::filters::blur::BlurFilter;
use crate::filters::{
    FilterSource, FilterVertexWithDoubleBlur, VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_DOUBLE_BLUR,
};
use crate::surface::target::CommandTarget;
use crate::utils::SampleCountMap;
use bytemuck::{Pod, Zeroable};
use std::sync::OnceLock;
use swf::GradientFilter as GradientBevelFilterArgs;
use wgpu::util::{DeviceExt, StagingBelt};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
struct GradientBevelUniform {
    strength: f32,
    bevel_type: u32, // 0 outer, 1 inner, 2 full
    knockout: u32,
    composite_source: u32,
}

pub struct GradientBevelFilter {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    vertices_size: wgpu::BufferSize,
    uniform_size: wgpu::BufferSize,
    pipeline: SampleCountMap<OnceLock<wgpu::RenderPipeline>>,
}

impl GradientBevelFilter {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<GradientBevelUniform>() as u64;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: create_debug_label!("Gradient bevel filter binds").as_deref(),
        });

        let vertices_size = std::mem::size_of::<[FilterVertexWithDoubleBlur; 4]>() as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertices_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        Self {
            pipeline: Default::default(),
            pipeline_layout,
            vertex_buffer,
            uniform_buffer,
            bind_group_layout,
            uniform_size: wgpu::BufferSize::new(uniform_size).expect("Definitely not zero."),
            vertices_size: wgpu::BufferSize::new(vertices_size).expect("Definitely not zero."),
        }
    }

    fn pipeline(&self, descriptors: &Descriptors, msaa_sample_count: u32) -> &wgpu::RenderPipeline {
        self.pipeline.get_or_init(msaa_sample_count, || {
            let label = create_debug_label!("Gradient Bevel Filter ({} msaa)", msaa_sample_count);
            descriptors
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: label.as_deref(),
                    layout: Some(&self.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &descriptors.shaders.gradient_bevel_filter,
                        entry_point: Some("main_vertex"),
                        buffers: &VERTEX_BUFFERS_DESCRIPTION_FILTERS_WITH_DOUBLE_BLUR,
                        compilation_options: Default::default(),
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::default(),
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: msaa_sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &descriptors.shaders.gradient_bevel_filter,
                        entry_point: Some("main_fragment"),
                        targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
                        compilation_options: Default::default(),
                    }),
                    multiview_mask: None,
                    cache: None,
                })
        })
    }

    #[expect(clippy::too_many_arguments)]
    pub fn apply(
        &self,
        descriptors: &Descriptors,
        texture_pool: &mut TexturePool,
        draw_encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut StagingBelt,
        source: &FilterSource,
        filter: &GradientBevelFilterArgs,
        blur_filter: &BlurFilter,
    ) -> CommandTarget {
        let sample_count = source.texture.sample_count();
        let format = source.texture.format();
        let pipeline = self.pipeline(descriptors, sample_count);
        let blurred = blur_filter.apply(
            descriptors,
            texture_pool,
            draw_encoder,
            staging_belt,
            source,
            &filter.inner_blur_filter(),
        );
        let blurred_texture = if let Some(blurred) = &blurred {
            blurred.ensure_cleared(draw_encoder);
            blurred.color_texture()
        } else {
            source.texture
        };
        let source_view = source.texture.create_view(&Default::default());
        let blurred_view = blurred_texture.create_view(&Default::default());
        let distance = filter.distance.to_f32();
        let angle = filter.angle.to_f32();
        let blur_offset = (angle.cos() * distance, angle.sin() * distance);

        let target = CommandTarget::new(
            descriptors,
            texture_pool,
            wgpu::Extent3d {
                width: source.size.0,
                height: source.size.1,
                depth_or_array_layers: 1,
            },
            format,
            sample_count,
            RenderTargetMode::FreshWithColor(wgpu::Color::TRANSPARENT),
            draw_encoder,
        );
        let gradient = descriptors.device.create_texture_with_data(
            &descriptors.queue,
            &wgpu::TextureDescriptor {
                label: create_debug_label!("Gradient bevel lookup").as_deref(),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &gradient_colors(filter),
        );
        let gradient_view = gradient.create_view(&Default::default());
        staging_belt
            .write_buffer(draw_encoder, &self.uniform_buffer, 0, self.uniform_size)
            .copy_from_slice(bytemuck::cast_slice(&[GradientBevelUniform {
                strength: filter.strength.to_f32(),
                bevel_type: if filter.is_on_top() {
                    2
                } else if filter.is_inner() {
                    1
                } else {
                    0
                },
                knockout: if filter.is_knockout() { 1 } else { 0 },
                composite_source: u32::from(
                    filter
                        .flags
                        .contains(swf::GradientFilterFlags::COMPOSITE_SOURCE),
                ),
            }]));
        let mut vertices = source.vertices_with_highlight_and_shadow(blur_offset);
        if let Some(blurred) = &blurred {
            // A blurred subrectangle occupies its own texture, starting at (0, 0).
            let blur_vertices = FilterSource::for_entire_texture(blurred.color_texture())
                .vertices_with_highlight_and_shadow(blur_offset);
            for (vertex, blur_vertex) in vertices.iter_mut().zip(blur_vertices) {
                vertex.blur_uv_left = blur_vertex.blur_uv_left;
                vertex.blur_uv_right = blur_vertex.blur_uv_right;
            }
        }
        staging_belt
            .write_buffer(draw_encoder, &self.vertex_buffer, 0, self.vertices_size)
            .copy_from_slice(bytemuck::cast_slice(&vertices));
        let filter_group = descriptors
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: create_debug_label!("Filter group").as_deref(),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&source_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            descriptors.bitmap_samplers.get_sampler(false, true),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&blurred_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&gradient_view),
                    },
                ],
            });
        let mut render_pass = draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: create_debug_label!("Gradient bevel filter").as_deref(),
            color_attachments: &[target.color_attachments()],
            ..Default::default()
        });
        render_pass.set_pipeline(pipeline);

        render_pass.set_bind_group(0, &filter_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            descriptors.quad.indices.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..6, 0, 0..1);
        target
    }
}

/// The filter indexes a 256-entry gradient by the difference of two alpha samples.
fn gradient_colors(filter: &GradientBevelFilterArgs) -> [u8; 256 * 4] {
    let mut colors = [0; 256 * 4];
    if filter.colors.is_empty() {
        return colors;
    }
    let mut index = 0;
    for (ratio, rgba) in colors.chunks_exact_mut(4).enumerate() {
        if ratio > usize::from(filter.colors[0].ratio)
            && index + 1 < filter.colors.len()
            && ratio > usize::from(filter.colors[index + 1].ratio)
        {
            index += 1;
        }
        let left = &filter.colors[index];
        let right = &filter.colors[(index + 1).min(filter.colors.len() - 1)];
        let t = if ratio <= usize::from(left.ratio) || right.ratio <= left.ratio {
            0.0
        } else {
            ((ratio - usize::from(left.ratio)) as f32 / f32::from(right.ratio - left.ratio))
                .min(1.0)
        };
        for (out, (a, b)) in rgba.iter_mut().zip(
            [left.color.r, left.color.g, left.color.b, left.color.a]
                .into_iter()
                .zip([right.color.r, right.color.g, right.color.b, right.color.a]),
        ) {
            *out = (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8;
        }
    }
    colors
}
