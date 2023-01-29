use wgpu::util::DeviceExt;

use game_logic::Asteroid;
use logger::warn;

use super::{Basic2DVertex, set_zoom_for_quad, load_textures_to_array};

fn generate_instances(asteroids: &[Asteroid], cam_pos: cgmath::Point2<f64>) -> Vec<InstanceRaw> {
    let start = std::time::Instant::now();

    let mut output = Vec::<InstanceRaw>::with_capacity(asteroids.len());
    
    for x in asteroids {
        output.push(InstanceRaw::from_asteroid(x, cam_pos));
    }

    let elapsed = start.elapsed().as_secs_f32();
    if elapsed > 0.01 {
        warn(1, 
            format!("Time taken to convert asteroids to InstanceRaw {} for len {} speed {}", elapsed, output.len(), elapsed / output.len() as f32)
            );
    }
    
    return output;
}

pub struct AsteroidPipeline {
    render_pipeline: wgpu::RenderPipeline,

    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    overall_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    scr_ratio: f32,
    cam_zoom: f32,

    n_ast: usize,
}

impl AsteroidPipeline {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration, asteroids: &[Asteroid], zoom: f64) -> AsteroidPipeline {
        let (asteroid_texture_views, asteroid_texture_sampler) = load_textures_to_array(device, queue, "asteroids", "asteroid");

        let scr_ratio = config.height as f32 / config.width as f32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Asteroid Vertex Buffer"),
            contents: bytemuck::cast_slice(&set_zoom_for_quad(QUAD)),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Asteroid Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let n_ast = asteroids.len();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Asteroid Instance Buffer"),
            contents: bytemuck::cast_slice(&generate_instances(asteroids, cgmath::Point2::new(0., 0.))),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let overall_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Overall Bufffer Asteroids"),
            contents: bytemuck::cast_slice(&[overall_data(scr_ratio, zoom as f32)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shader_mod = device.create_shader_module(wgpu::include_wgsl!("ast_shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Asteroids Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None, 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: false }, 
                        view_dimension: wgpu::TextureViewDimension::D2Array, 
                        multisampled: false 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Asteroids Unfirom Bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(overall_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&asteroid_texture_views),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&asteroid_texture_sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Asteroid Render Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout
                ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Asteroid Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { 
                module: &shader_mod, 
                entry_point: "vs_main", 
                buffers: &[
                    Basic2DVertex::desc(),
                    InstanceRaw::desc(),
                ] 
            },
            primitive: wgpu::PrimitiveState { 
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None, 
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: None, 
                unclipped_depth: false, 
                polygon_mode: wgpu::PolygonMode::Fill, 
                conservative: false, 
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { 
                count: 1, 
                mask: !0, 
                alpha_to_coverage_enabled: false 
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_mod,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })
                ],
            }),
            multiview: None,
        });

        return AsteroidPipeline { 
            render_pipeline, 
            index_buffer, 
            vertex_buffer, 
            instance_buffer,
            n_ast,
            overall_buffer,
            bind_group, 
            scr_ratio,
            cam_zoom: zoom as f32,
        };
    }

    pub fn update_cam_zoom(&mut self, queue: &wgpu::Queue, new_zoom: f64) {
        let new_zoom = new_zoom as f32;
        queue.write_buffer(&self.overall_buffer, 0, bytemuck::cast_slice(&[overall_data(self.scr_ratio, new_zoom)]));
        self.cam_zoom = new_zoom;
    }

    pub fn update(&mut self, asteroids: &[Asteroid], queue: &wgpu::Queue, device: &wgpu::Device, camera_pos: cgmath::Point2<f64>) {        
        if self.n_ast != asteroids.len() {
            self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("New Asteroid Instance Buffer"),
                contents: bytemuck::cast_slice(&generate_instances(asteroids, camera_pos)),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            });
            self.n_ast = asteroids.len();
        } else {
            let data = generate_instances(asteroids, camera_pos);
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&data));
        }

    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, queue: &wgpu::Queue) {
        
        self.scr_ratio = new_size.height as f32 / new_size.width as f32;
        queue.write_buffer(&self.overall_buffer, 0, bytemuck::cast_slice(&[overall_data(self.scr_ratio, self.cam_zoom)]));
        
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Asteroid Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Load, 
                        store: true 
                    },
                }),
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..self.n_ast as u32);
    }
}



const QUAD: &[Basic2DVertex] = &[
    Basic2DVertex { position: [-1., 1.], tex_coords: [0., 0.] },
    Basic2DVertex { position: [-1., -1.], tex_coords: [0., 1.] },
    Basic2DVertex { position: [1., -1.], tex_coords: [1., 1.] },
    Basic2DVertex { position: [1., 1.], tex_coords: [1., 0.] },
];

const QUAD_INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1., 0., 0., 0., 
    0., 1., 0., 0., 
    0., 0., 0.5, 0., 
    0., 0., 0.5, 1.
);

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub img_idx: i32,
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        return wgpu::VertexBufferLayout { 
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Instance, 
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32,
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 6,
                }
            ],
        };
    }
    
    fn from_asteroid(asteroid: &Asteroid, cam_pos: cgmath::Point2<f64>) -> InstanceRaw {
        let rot = cgmath::Rad(asteroid.rot);
        let pos = cgmath::Vector3 { x: (asteroid.pos.x - cam_pos.x) as f32, y: (asteroid.pos.y - cam_pos.y) as f32, z: 0.};
        let model = cgmath::Matrix4::from_translation(pos) * cgmath::Matrix4::<f32>::from_angle_z(rot);

        return InstanceRaw {
            model: model.into(),
            img_idx: asteroid.img_idx,
        };
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OveralBuffer {
    view_proj: [[f32; 4]; 4],
}

fn overall_data(scr_ratio: f32, zoom: f32) -> OveralBuffer {
    const N_UNIT: f32 = 1.;

    let unit_w;
    let unit_h;
    if scr_ratio > 1. {
        unit_w = 1. / N_UNIT;
        unit_h = unit_w / scr_ratio;
    } else {
        unit_h = 1. / N_UNIT;
        unit_w = unit_h * scr_ratio;
    }

    let deform = cgmath::Matrix4::from_nonuniform_scale(unit_w * zoom, unit_h * zoom, 1.);
    let mat = OPENGL_TO_WGPU_MATRIX * deform;

    return OveralBuffer { view_proj: mat.into() };
}