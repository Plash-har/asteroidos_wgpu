use wgpu::include_wgsl;
use wgpu::util::DeviceExt;

use game_logic::Player;

use super::{Basic2DVertex, set_zoom_for_quad, load_textures_to_array};

fn generate_instances(players: &[Player], cam_pos: &cgmath::Point2<f64>) -> Vec<InstanceRaw> {
    let mut output = Vec::with_capacity(players.len());

    for x in players {
        output.push(InstanceRaw::from_player(x, cam_pos));
    }

    return output;
}

pub struct PlayerPipeline {
    render_pipeline: wgpu::RenderPipeline,

    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    overall_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    scr_ratio: f32,
    cam_zoom: f32,

    n_player: usize,
}

impl PlayerPipeline {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration, players: &[Player], zoom: f64) -> PlayerPipeline {
        let zoom = zoom as f32;

        let (player_texture_view, player_texture_sampler) = load_textures_to_array(device, queue, "players", "player");

        let scr_ratio = config.height as f32 / config.width as f32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Vertex Shader"),
            contents: bytemuck::cast_slice(&set_zoom_for_quad(PLAYER_VERTICES)),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Index Shader"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Instance Buffer"),
            contents: bytemuck::cast_slice(&generate_instances(players, &cgmath::Point2 { x: 0., y: 0. })),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        let overall_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Overall Buffer"),
            contents: bytemuck::cast_slice(&[overall_data(scr_ratio, zoom)]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let shader_mod = device.create_shader_module(include_wgsl!("player_shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Player Bind Group Layout"),
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
            label: Some("Player Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(overall_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&player_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&player_texture_sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Player Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Player Render Pipeline"),
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
                conservative: false 
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

        return PlayerPipeline { 
            render_pipeline, 
            index_buffer, 
            vertex_buffer, 
            instance_buffer, 
            overall_buffer, 
            bind_group, 
            scr_ratio, 
            cam_zoom: zoom, 
            n_player: players.len(), 
        };
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, queue: &wgpu::Queue) {
        self.scr_ratio = new_size.height as f32 / new_size.width as f32;
        queue.write_buffer(&self.overall_buffer, 0, bytemuck::cast_slice(&[overall_data(self.scr_ratio, self.cam_zoom)]));
    }

    pub fn update_cam_zoom(&mut self, queue: &wgpu::Queue, new_zoom: f64) {
        self.cam_zoom = new_zoom as f32;
        queue.write_buffer(&self.overall_buffer, 0, bytemuck::cast_slice(&[overall_data(self.scr_ratio, self.cam_zoom)]));
    }

    pub fn update(&mut self, players: &[Player], queue: &wgpu::Queue, device: &wgpu::Device, cam_pos: &cgmath::Point2<f64>) {
        if self.n_player != players.len() {
            self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NEW Player Instance Buffer"),
                contents: bytemuck::cast_slice(&generate_instances(players, cam_pos)),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            });

            self.n_player = players.len();
        } else {
            let data = generate_instances(players, cam_pos);
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&data));
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Player Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Load, 
                        store: true 
                    },
                })
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..self.n_player as u32);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    accent_color_0: [f32; 4],
    accent_color_1: [f32; 4],
    accent_color_2: [f32; 4],
    accent_color_3: [f32; 4],
    accent_flame_color: [f32; 4],
    flame_frame: f32,
    player_idx: i32,
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        return wgpu::VertexBufferLayout { 
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Instance, 
            attributes: &[
                wgpu::VertexAttribute { // Matrice 1
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 2,
                },
                wgpu::VertexAttribute { // Matrice 2
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute { // Matrice 3
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
                wgpu::VertexAttribute { // Matrice 4
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                },
                wgpu::VertexAttribute { // Accent color 0
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute { // Accent color 1
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute { // Accent color 2
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 24]>() as wgpu::BufferAddress,
                    shader_location: 8,
                },
                wgpu::VertexAttribute { // Accent color 3
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 28]>() as wgpu::BufferAddress,
                    shader_location: 9,
                },
                wgpu::VertexAttribute { // Accent Flame Color
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 32]>() as wgpu::BufferAddress,
                    shader_location: 10,
                },
                wgpu::VertexAttribute { // Flame Frame
                    format: wgpu::VertexFormat::Float32,
                    offset: std::mem::size_of::<[f32; 36]>() as wgpu::BufferAddress,
                    shader_location: 11,
                },
                wgpu::VertexAttribute { // Player idx
                    format: wgpu::VertexFormat::Sint32,
                    offset: std::mem::size_of::<[f32; 37]>() as wgpu::BufferAddress,
                    shader_location: 12,
                },
            ],
        };
    }

    fn from_player(player: &Player, cam_pos: &cgmath::Point2<f64>) -> InstanceRaw {
        let pos = cgmath::Vector3 { x: (player.pos.x - cam_pos.x) as f32, y: (player.pos.y - cam_pos.y) as f32, z: 0. };
        let rot = cgmath::Rad(player.rot);
        let model = cgmath::Matrix4::from_translation(pos) * cgmath::Matrix4::from_angle_z(rot);

        return InstanceRaw { 
            model: model.into(), 
            accent_color_0: player.accent_color_0, 
            accent_color_1: player.accent_color_1, 
            accent_color_2: player.accent_color_2, 
            accent_color_3: player.accent_color_3, 
            accent_flame_color: player.accent_flame_color, 
            flame_frame: player.flame_frame, 
            player_idx: player.player_img, 
        };
    }
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1., 0., 0., 0., 
    0., 1., 0., 0., 
    0., 0., 0.5, 0., 
    0., 0., 0.5, 1.
);

const PLAYER_VERTICES: &[Basic2DVertex] = &[
    Basic2DVertex { position: [-1., -1.], tex_coords: [0.5001, 0.] }, // Flames
    Basic2DVertex { position: [-1., 1.], tex_coords: [0.5001, 0.25] },
    Basic2DVertex { position: [1., 1.], tex_coords: [1., 0.25] },
    Basic2DVertex { position: [1., -1.], tex_coords: [1., 0.] },

    Basic2DVertex { position: [-1., -1.], tex_coords: [0.0, 0.] }, // Layer 0
    Basic2DVertex { position: [-1., 1.], tex_coords: [0.0, 0.25] },
    Basic2DVertex { position: [1., 1.], tex_coords: [0.5, 0.25] },
    Basic2DVertex { position: [1., -1.], tex_coords: [0.5, 0.] },

    Basic2DVertex { position: [-1., -1.], tex_coords: [0.0, 0.25] }, // Layer 1
    Basic2DVertex { position: [-1., 1.], tex_coords: [0.0, 0.5] },
    Basic2DVertex { position: [1., 1.], tex_coords: [0.5, 0.5] },
    Basic2DVertex { position: [1., -1.], tex_coords: [0.5, 0.25] },

    Basic2DVertex { position: [-1., -1.], tex_coords: [0.0, 0.5] }, // Layer 2
    Basic2DVertex { position: [-1., 1.], tex_coords: [0.0, 0.75] },
    Basic2DVertex { position: [1., 1.], tex_coords: [0.5, 0.75] },
    Basic2DVertex { position: [1., -1.], tex_coords: [0.5, 0.5] },

    
    Basic2DVertex { position: [-1., -1.], tex_coords: [0.0, 0.75] }, // Layer 3
    Basic2DVertex { position: [-1., 1.], tex_coords: [0.0, 1.] },
    Basic2DVertex { position: [1., 1.], tex_coords: [0.5, 1.] },
    Basic2DVertex { position: [1., -1.], tex_coords: [0.5, 0.75] },
];

const QUAD_INDICES: &[u16] = &[
    3, 0, 1,
    3, 1, 2,

    7, 4, 5,
    7, 5, 6,

    11, 8, 9,
    11, 9, 10,

    15, 12, 13,
    15, 13, 14,

    19, 16, 17,
    19, 17, 18,
];

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