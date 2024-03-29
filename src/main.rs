#![allow(non_snake_case)]

#[cfg(feature = "hot-reload")]
use tpaint::prelude::dioxus_hot_reload;
use tpaint::DomEventLoop;
use tpaint_wgpu::{Renderer, ScreenDescriptor};
use winit::event::WindowEvent;

mod app;
mod asset_browser;
mod console;
mod drawer;
mod inspector;
mod scene_viewer;

type UserEvent = ();

#[derive(Clone)]
struct RootContext {
    window_id: u64,
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "hot-reload")]
    dioxus_hot_reload::hot_reload_init!();

    // #[cfg(feature = "tracy")]
    // let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    // #[cfg(feature = "tracy")]
    // use tracing_subscriber::layer::SubscriberExt;
    // #[cfg(feature = "tracy")]
    // tracing::subscriber::set_global_default(tracing_subscriber::registry().with(chrome_layer))
    //     .expect("set up the subscriber");

    let event_loop = winit::event_loop::EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(true)
        .with_title("roth")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: 1920,
            height: 1080,
        })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let size = window.inner_size();

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

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

    let mut renderer = Renderer::new(&device, swapchain_format, None, 1);

    let mut app = DomEventLoop::spawn(
        app::app,
        window.inner_size(),
        window.scale_factor() as f32,
        event_loop.create_proxy(),
        (),
        RootContext {
            window_id: window.id().into(),
        },
    );

    event_loop
        .run(move |event, target| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.
            let _ = (&instance, &adapter);

            let mut redraw = || {
                target.set_control_flow(winit::event_loop::ControlFlow::Wait);
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                let (primitives, delta, screen_descriptor) = app.get_paint_info();

                for (id, texture) in delta.set {
                    renderer.update_texture(&device, &queue, id, &texture);
                }

                for id in delta.free {
                    renderer.free_texture(&id);
                }

                let screen = &ScreenDescriptor {
                    size_in_pixels: screen_descriptor.size.into(),
                    pixels_per_point: screen_descriptor.pixels_per_point,
                };
                renderer.update_buffers(&device, &queue, &mut encoder, &primitives, screen);

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    renderer.render(&mut rpass, &primitives, screen)
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            };

            match event {
                // winit::event::Event::RedrawRequested if !cfg!(target_os = "windows") => redraw(),
                winit::event::Event::WindowEvent {
                    event: ref window_event,
                    ..
                } => {
                    match window_event {
                        WindowEvent::Resized(size) => {
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);
                            window.request_redraw();
                        }

                        WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                            target.exit();
                        }

                        WindowEvent::RedrawRequested => {
                            redraw();
                        }

                        _ => {}
                    }

                    let repaint = app.on_window_event(window_event);
                    if repaint {
                        window.request_redraw();
                    }
                }

                winit::event::Event::UserEvent(_) => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
