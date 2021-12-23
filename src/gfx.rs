use crate::window::Window;

pub(crate) struct GFX {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl GFX {
    pub async fn new(window: &Window) -> Self {
        // Instance of wgpu. Its primary use is to create `Adapter`s and `Surface`s.
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // A `Surface` represents a platform-specific surface (e.g. a window)
        // onto which rendered images may be presented.
        // It's the part of the window that we draw to.
        // Created from raw window handle.
        let surface = unsafe { instance.create_surface(window) };

        // Handle to a physical graphics and/or compute device.
        // Adapters can be used to open a connection to the corresponding `Device`
        //on the host system
        let adapter = {
            let options = wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            };

            // Retrieves an `Adapter` which matches the given `RequestAdapterOptions`.
            // If wgpu can't find an adapter with the required permissions,
            // request_adapter will return None
            instance.request_adapter(&options).await.unwrap() // Hard panic for now.
        };

        // Open connection to a graphics and/or compute device
        // and get handle to a command queue on a device.
        let (device, queue) = {
            let desc = wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            };

            // Requests a connection to a physical device, creating a logical device.
            // Returns the Device together with a Queue that executes command buffers.
            adapter.request_device(&desc, None).await.unwrap()
        };

        // Configures a `Surface` for presentation.
        let surface_config = wgpu::SurfaceConfiguration {
            
            // The usage of the swap chain. 
            // The only supported usage is `RENDER_ATTACHMENT`.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            
            // The texture format of the swap chain.
            format: surface.get_preferred_format(&adapter).unwrap(),
            
            // Width and height of the swap chain. 
            // Must be the same size as the surface.
            width: window.width as u32,
            height: window.height as u32,
            
            // Presentation mode of the swap chain. 
            // FIFO is the only guaranteed to be supported.
            // FIFO will cap the display rate at the displays framerate.
            // This is essentially VSync. This is also the most optimal mode on mobile. 
            present_mode: wgpu::PresentMode::Fifo,
        };

        // Initializes `Surface` for presentation.
        surface.configure(&device, &surface_config);

        Self {
            surface,
            device,
            queue,
            config: surface_config,
        }
    }

    // Support window resizing
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Returns the next texture to be presented by the swapchain for drawing.
        let output = self.surface.get_current_texture()?;

        // Creates a view of this texture.
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Encodes a series of GPU operations.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {  // Begins recording of a render pass.

            let color_attachments = &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None, // same as view unless multisampling is used.
                // What operations will be performed on this color attachment.
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }];
            
            let desc = {
                wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: color_attachments,
                    depth_stencil_attachment: None,
                }

            };
            
            let _render_pass = encoder.begin_render_pass(&desc);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
        

    }
}
