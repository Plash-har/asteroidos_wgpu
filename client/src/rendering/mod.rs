mod asteroid_rendering;
mod player_rendering;
mod gui_rendering;

use game_logic::World;

use asteroid_rendering::AsteroidPipeline;
use player_rendering::PlayerPipeline;
use gui_rendering::GuiPipeline;

pub struct MainRenderer {
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,
    pub size: winit::dpi::PhysicalSize<u32>,
    queue: wgpu::Queue,

    asteroid_pipeline: AsteroidPipeline,
    player_pipeline: PlayerPipeline,
    gui_pipeline: GuiPipeline,
    
    cam_pos: cgmath::Point2<f64>,
    zoom: f64,
}

impl MainRenderer {
    const DEFAULT_PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::Fifo;

    pub async fn new(window: &winit::window::Window, zoom: f64, cam_pos: cgmath::Point2<f64>) -> MainRenderer {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: Self::DEFAULT_PRESENT_MODE,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                features: wgpu::Features::empty() | wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits::default(),
            }, None).await.unwrap();

        surface.configure(&device, &config);

        let asteroid_pipeline = AsteroidPipeline::new(&device, &queue, &config, &[], zoom);

        let player_pipeline = PlayerPipeline::new(&device, &queue, &config, &[], zoom);

        let gui_pipeline = GuiPipeline::new(&device, &config, window.scale_factor(), surface.get_supported_formats(&adapter)[0]);

        return MainRenderer {
            device,
            config,
            surface,
            size,
            queue,
            asteroid_pipeline,
            cam_pos,
            zoom,
            player_pipeline,
            gui_pipeline,
        };
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f64>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.asteroid_pipeline.resize(new_size, &self.queue);
            self.player_pipeline.resize(new_size, &self.queue);
            self.gui_pipeline.resize(new_size, scale_factor);
        }
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main command Encoder"),
        });

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Basic Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { 
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0., g: 0., b: 0., a: 1. }), 
                            store: true 
                        },
                    }),
                ],
                depth_stencil_attachment: None,
            });
        }

        self.asteroid_pipeline.render(&mut encoder, &view);

        self.player_pipeline.render(&mut encoder, &view);

        self.gui_pipeline.render(&mut encoder, &view, window, &self.device, &self.queue);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        return Ok(());
    }
    
    pub fn update(&mut self, game: &World) {
        
        self.asteroid_pipeline.update(&game.asteroids, &self.queue, &self.device, self.cam_pos);
        self.player_pipeline.update(&game.players, &self.queue, &self.device, &self.cam_pos);
        
    }

    pub fn set_cam_pos(&mut self, new_pos: cgmath::Point2<f64>) {
        self.cam_pos = new_pos;
    }

    pub fn set_zoom(&mut self, new_zoom: f64) {
        self.zoom = new_zoom;
        self.asteroid_pipeline.update_cam_zoom(&self.queue, self.zoom);
        self.player_pipeline.update_cam_zoom(&self.queue, new_zoom);
    }

    pub fn get_gui_context(&mut self) -> egui::Context {
        return self.gui_pipeline.get_context();
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.gui_pipeline.handle_event(event);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Basic2DVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Basic2DVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        return wgpu::VertexBufferLayout { 
            array_stride: std::mem::size_of::<Basic2DVertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex, 
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                }
            ] 
        };
    }
}

fn set_zoom_for_quad(quad: &[Basic2DVertex]) -> Vec<Basic2DVertex> {
    const N_WIDTH: f32 = 20.; // The number of asteroids you can put in the width of the screen

    let width = 1. / N_WIDTH;
    let heigth = width;

    let mut output = Vec::from(quad);

    for (idx, x) in output.iter_mut().enumerate() {
        match idx % 4 {
            0 => {
                x.position = [width * -0.5, heigth * 0.5];
            },
            1 => {
                x.position = [width * -0.5, heigth * -0.5];
            },
            2 => {
                x.position = [width * 0.5, heigth * -0.5];
            },
            3 => {
                x.position = [width * 0.5, heigth * 0.5];
            },
            _ => {panic!("Computer stupid !")}
        }
    }

    return output;
}

/// Loads all the images in a assets/{path}/ into an array of textures
/// the item argument is for debug purposes, to know of which item we are talking
fn load_textures_to_array(device: &wgpu::Device, queue: &wgpu::Queue, path: &str, item: &str) -> (wgpu::TextureView, wgpu::Sampler) {
    use std::fs;
    use image::GenericImageView;

    let files_dir = match fs::read_dir(format!("assets/{}/", path)) {
        Ok(val) => val,
        Err(err) => panic!("Unable to open {} texture dir: {}", item, err),
    };

    let mut imgs= Vec::new();
    for file in files_dir.into_iter() {
        match file {
            Ok(file) => {
                let img = match image::open(file.path()) {
                    Ok(val) => val,
                    Err(image::ImageError::Decoding(err)) => panic!("Error while decoding {} texture: {}", item, err),
                    Err(err) => panic!("Error while handling {} texture {}", item, err),
                };

                imgs.push(img);
            },
            Err(err) => panic!("Unable to open {} texture file: {}", item, err)
        }
    }
        let img_dimesion = imgs[0].dimensions();

        let texture_array_size = wgpu::Extent3d {
            width: img_dimesion.0,
            height: img_dimesion.1,
            depth_or_array_layers: imgs.len() as u32,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("{} texture", item)),
            size: texture_array_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let texture_size = wgpu::Extent3d {
            width: img_dimesion.0,
            height: img_dimesion.1,
            depth_or_array_layers: 1,
        };

        for (z, img) in imgs.iter().enumerate() {
            let img_rgba = img.to_rgba8();
            let img_dimesion = img.dimensions();

            queue.write_texture(
                wgpu::ImageCopyTextureBase { texture: &texture, mip_level: 0, origin: wgpu::Origin3d { 
                    x: 0, 
                    y: 0, 
                    z: z as u32 }, 
                    aspect: wgpu::TextureAspect::All }, 
                &img_rgba, 
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * img_dimesion.0),
                    rows_per_image: std::num::NonZeroU32::new(img_dimesion.1),
                }, 
                texture_size
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&format!("THE {} texture view", item)),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("The {} sampler", item)),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    return (view, sampler);
}