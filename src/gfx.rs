use crate::window::Window;

pub(crate) struct GFX {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
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

        // Create shader module from WGSL source code.
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Handle to pipeline layout.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // type of vertices we want to pass to the vertex shader.
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                // The targets field tells wgpu what color outputs it should set up.
                // Currently, we only need one for the surface.
                targets: &[wgpu::ColorTargetState {
                    format: surface_config.format,                  // Surface's format.
                    blend: Some(wgpu::BlendState::REPLACE), // Replace old with new.
                    write_mask: wgpu::ColorWrites::ALL, // write to all colors: red, blue, green, and alpha.
                }],
            }),
            // The primitive field describes how to interpret our vertices when converting them into triangles.
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // Each three vertices will correspond to one triangle.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // a triangle is facing forward if the vertices are arranged in a counter-clockwise direction.
                cull_mode: Some(wgpu::Face::Back), // Not front-facing triangles are excluded from render (culled).
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState {
                count: 1, // No multisampling.                         
                mask: !0, // Use all samples.                   
                alpha_to_coverage_enabled: false, 
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config: surface_config,
            render_pipeline,
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
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Encodes a series of GPU operations.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // Begins recording of a render pass.

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

            let mut render_pass = encoder.begin_render_pass(&desc);

            render_pass.set_pipeline(&self.render_pipeline);
            // Draw something with 3 vertices and 1 instance.
            // Used in [[builtin(vertex_index)]] in the shader source.
            render_pass.draw(0..3, 0..1);

        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
