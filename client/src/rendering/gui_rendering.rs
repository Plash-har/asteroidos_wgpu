use egui_winit_platform::{Platform, PlatformDescriptor};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use wgpu::TextureFormat;

pub struct GuiPipeline {
    platform: Platform,
    render_pass: RenderPass,
    screen_descriptor: ScreenDescriptor,
    begun_frame: bool,
}

impl GuiPipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, scale_factor: f64, output_format: TextureFormat) -> GuiPipeline {
        let platform = Platform::new(PlatformDescriptor { 
            physical_width: config.width, 
            physical_height: config.height, 
            scale_factor: scale_factor, 
            font_definitions: egui::FontDefinitions::default(), 
            style: Default::default(), 
        });

        let render_pass = RenderPass::new(device, output_format, 1);

        let screen_descriptor = ScreenDescriptor {
            physical_width: config.width,
            physical_height: config.height,
            scale_factor: scale_factor as f32,
        };

        return GuiPipeline { platform, render_pass, screen_descriptor, begun_frame: false };
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: Option<f64>) {
        self.screen_descriptor.physical_width = new_size.width;
        self.screen_descriptor.physical_height = new_size.height;  
        if let Some(scale_factor) = scale_factor {
            self.screen_descriptor.scale_factor = scale_factor as f32;
        }
    }

    pub fn get_context(&mut self) -> egui::Context {
        if self.begun_frame {
            eprintln!("Warning requested context while gui frame has begun");
        }

        self.platform.begin_frame();

        self.begun_frame = true;

        return self.platform.context();
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, window: &winit::window::Window, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.begun_frame {
            let output = self.platform.end_frame(Some(window));
            let paint_jobs = self.platform.context().tessellate(output.shapes);

            let texture_delta = output.textures_delta;

            match self.render_pass.add_textures(device, queue, &texture_delta) {
                Ok(_) => {},
                Err(err) => eprintln!("Error while rendering gui: {:?}", err),
            };

            self.render_pass.update_buffers(device, queue, &paint_jobs, &self.screen_descriptor);

            match self.render_pass.execute(encoder, view, &paint_jobs, &self.screen_descriptor, None) {
                Ok(_) => {},
                Err(err) => eprintln!("Error while executing gui: {:?}", err),
            };

            match self.render_pass.remove_textures(texture_delta) {
                Ok(_) => {},
                Err(err) => eprintln!("Error while removing Texture Delta: {:?}", err),
            }

            self.begun_frame = false;
        }
    }

    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.platform.handle_event(event);
    }
}

