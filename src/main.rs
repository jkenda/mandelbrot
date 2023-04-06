use std::borrow::Cow;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, Theme}, platform::unix::WindowBuilderExtUnix,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let mut aspect = size.width as f32 / size.height as f32;
    let mut camera_controller = CameraController::new(0.02);

    let aspect_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Aspect ratio buffer"),
            contents: bytemuck::cast_slice(&[aspect]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let properties_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera controller buffer"),
            contents: bytemuck::cast_slice(&[camera_controller.properties]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let aspect_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            }
        ],
        label: Some("aspect_bind_group_layout"),
    });

    let properties_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ],
        label: Some("aspect_bind_group_layout"),
    });

    let aspect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &aspect_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: aspect_buffer.as_entire_binding(),
            }
        ],
        label: Some("aspect_bind_group"),
    });

    let properties_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &properties_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: properties_buffer.as_entire_binding(),
            }
        ],
        label: Some("aspect_bind_group"),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &aspect_bind_group_layout,
            &properties_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                aspect = size.width as f32 / size.height as f32;
                queue.write_buffer(&aspect_buffer, 0, bytemuck::cast_slice(&[aspect]));
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                camera_controller.process_events(&event);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                camera_controller.update_camera();
                queue.write_buffer(&properties_buffer, 0, bytemuck::cast_slice(&[camera_controller.properties]));
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &aspect_bind_group, &[]);
                    rpass.set_bind_group(1, &properties_bind_group, &[]);
                    rpass.draw(0..6, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {}
        }
    });
}
// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Properties {
    center: [f32; 2],
    zoom: f32,
    _padding: u32,
}

struct CameraController {
    properties: Properties,
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_zoom_in_pressed: bool,
    is_zoom_out_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            properties: Properties {
                _padding: 0,
                center: [-0.25, 0.0],
                zoom: 2.4,
            },
            is_up_pressed: false,
            is_down_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_zoom_in_pressed: false,
            is_zoom_out_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::K | VirtualKeyCode::Up => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::H | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::J | VirtualKeyCode::Down => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::L | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::PageUp => {
                        self.is_zoom_in_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::PageDown => {
                        self.is_zoom_out_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera(&mut self) {
        if self.is_up_pressed {
            self.properties.center[1] += self.speed * self.properties.zoom;
        }
        if self.is_down_pressed {
            self.properties.center[1] -= self.speed * self.properties.zoom;
        }
        if self.is_left_pressed {
            self.properties.center[0] -= self.speed * self.properties.zoom;
        }
        if self.is_right_pressed {
            self.properties.center[0] += self.speed * self.properties.zoom;
        }
        if self.is_zoom_in_pressed {
            self.properties.zoom -= self.speed * self.properties.zoom;
        }
        if self.is_zoom_out_pressed {
            self.properties.zoom += self.speed * self.properties.zoom;
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Mandelbrotov fraktal")
        .with_gtk_theme_variant("dark".to_string())
        .with_wayland_csd_theme(Theme::Dark)
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
