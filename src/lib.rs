mod interactive;

use std::{borrow::Cow, time::{Instant, Duration}};

use interactive::camera_controller::CameraController;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, Fullscreen},
};

pub async fn main(event_loop: EventLoop<()>, window: Window) {
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
                features: wgpu::Features::SHADER_FLOAT64,
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

    let mut camera_controller = CameraController::new(0.02, size.width, size.height);
    let mut f11_state_prev = ElementState::Released;
    let mut esc_state_prev = ElementState::Released;
    let mut frame_time = Duration::new(1, 0);

    let properties_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera controller buffer"),
            contents: bytemuck::cast_slice(&[camera_controller.properties()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let properties_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        let start = Instant::now();

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                camera_controller.update_window_size(size.width, size.height);
                queue.write_buffer(&properties_buffer, 0, bytemuck::cast_slice(&[camera_controller.properties()]));
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::F11),
                        state, .. }, .. }, ..
            } => {
                // toggle fullscreen with F11
                if state != f11_state_prev && state == ElementState::Pressed {
                    window.set_fullscreen(
                        if window.fullscreen() == None {
                            Some(Fullscreen::Borderless(None))
                        }
                        else {
                            None
                        });
                }
                f11_state_prev = state;
            },
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        state, .. }, .. }, ..
            } => {
                // exit fullscreen with Esc
                if state != esc_state_prev
                    && state == ElementState::Pressed
                    && window.fullscreen() != None
                {
                    window.set_fullscreen(None);
                }
                esc_state_prev = state;
            }
            Event::WindowEvent { event, .. } => {
                let changed = camera_controller.process_events(&event);
                if changed {
                    window.request_redraw();
                }

                window.set_title(&format!("Mandelbrotov fraktal | koordinate: ({}, {}) | zoom: {}x | čas sličice: {} ms ({} FPS)",
                    camera_controller.properties().center[0],
                    camera_controller.properties().center[1],
                    1.0 / camera_controller.properties().zoom,
                    frame_time.as_millis(),
                    1_000_000 / frame_time.as_micros()));
            }
            Event::RedrawRequested(_) => {
                camera_controller.update_window_size(config.width, config.height);
                queue.write_buffer(&properties_buffer, 0, bytemuck::cast_slice(&[camera_controller.properties()]));
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
                    rpass.set_bind_group(0, &properties_bind_group, &[]);
                    rpass.draw(0..6, 0..1);

                    frame_time = start.elapsed();
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {}
        }
    });
}
