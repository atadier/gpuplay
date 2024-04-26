use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use pico_args::Arguments;
use wgpu::{
    Adapter, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
    FragmentState, Instance, InstanceDescriptor, Limits, MultisampleState,
    PipelineLayoutDescriptor, PowerPreference, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptionsBase, ShaderModule, ShaderModuleDescriptor,
    Surface, SurfaceConfiguration, SurfaceTexture, TextureViewDescriptor, VertexState,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod shader;
mod watch;

const WINDOW_TITLE: &'static str = "GPU Playground";
const FPS_UPDATE_RATE: Duration = Duration::from_millis(200);

struct GraphicsContext<'s> {
    adapter: Adapter,
    device: Device,
    surface: Surface<'s>,
    config: SurfaceConfiguration,
    queue: Queue,
    pipeline: Option<RenderPipeline>,
}

impl<'s> GraphicsContext<'s> {
    pub fn pipeline(&self) -> &Option<RenderPipeline> {
        &self.pipeline
    }

    pub fn get_gpu_id(&self) -> String {
        self.adapter.get_info().name
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn add_shader_module(&mut self, path: &OsStr) {
        let module = shader::load_shader(&path).expect("failed to load shader");
        let shader_module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&String::from_utf8_lossy(path.as_encoded_bytes())),
            source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
        });
        self.build_pipeline_with_shader(&shader_module);
    }

    fn build_pipeline_with_shader(&mut self, module: &ShaderModule) {
        let surface_caps = self.surface.get_capabilities(&self.adapter);
        let format = surface_caps.formats[0];

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        self.pipeline = Some(
            self.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs_main",
                        targets: &[Some(format.into())],
                    }),
                    depth_stencil: None,
                    primitive: PrimitiveState::default(),
                    multisample: MultisampleState::default(),
                    multiview: None,
                }),
        );
    }

    pub fn begin_frame(&mut self) -> (SurfaceTexture, CommandEncoder) {
        let texure = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap chain texture");
        let commands = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        (texure, commands)
    }

    pub fn submit_frame(&mut self, frame: SurfaceTexture, commands: CommandEncoder) {
        self.queue.submit(Some(commands.finish()));
        frame.present();
    }
}

async fn graphics_init(window: &Window) -> GraphicsContext {
    let inner_size = window.inner_size();
    let instance = Instance::new(InstanceDescriptor::default());
    let surface = instance
        .create_surface(window)
        .expect("failed to create surface");

    let adapter = instance
        .request_adapter(&RequestAdapterOptionsBase {
            power_preference: PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("failed to find a graphics adapter");

    let default_config = surface
        .get_default_config(&adapter, inner_size.width, inner_size.height)
        .expect("graphics adapter is not compatible");
    let config = SurfaceConfiguration {
        present_mode: wgpu::PresentMode::AutoVsync,
        ..default_config
    };

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .expect("failed to create graphics device");
    surface.configure(&device, &config);

    GraphicsContext {
        adapter,
        surface,
        config,
        device,
        queue,
        pipeline: None,
    }
}

fn graphics_draw(ctx: &mut GraphicsContext) {
    let (frame, mut commands) = ctx.begin_frame();
    let view = frame.texture.create_view(&TextureViewDescriptor::default());

    {
        let mut render_pass = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Some(pipeline) = ctx.pipeline() {
            render_pass.set_pipeline(pipeline);
        }
        render_pass.draw(0..3, 0..1);
    }

    ctx.submit_frame(frame, commands);
}

fn create_shader_pipeline(ctx: &mut GraphicsContext, path: &OsStr) {
    ctx.add_shader_module(path);
}

async fn run(shader_path: OsString) {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_min_inner_size(PhysicalSize::new(64, 64))
        .build(&event_loop)
        .unwrap();

    let mut graphics_ctx = graphics_init(&window).await;
    create_shader_pipeline(&mut graphics_ctx, &shader_path);

    let (tx, rx) = mpsc::channel();
    let reload_path = shader_path.clone();
    thread::spawn(move || watch::send_reload(reload_path, tx));

    let window_title = format!("{} - {}", WINDOW_TITLE, graphics_ctx.get_gpu_id());
    window.set_title(&window_title);

    let window = &window;
    let mut last_frame = Instant::now();
    let mut last_frame_update = Instant::now();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => graphics_ctx.resize(size),
                WindowEvent::RedrawRequested => {
                    graphics_draw(&mut graphics_ctx);

                    if last_frame_update.elapsed() > FPS_UPDATE_RATE {
                        let fps = 1. / last_frame.elapsed().as_secs_f64();
                        window.set_title(&format!("{} - {:.0} FPS", window_title, fps));
                        last_frame_update = Instant::now();
                    }
                    last_frame = Instant::now();
                }
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();

                match rx.try_recv() {
                    Ok(watch::FileReloadNotification) => {
                        println!("reloading shader module...");
                        graphics_ctx.add_shader_module(&shader_path);
                    }
                    _ => (),
                }
            }
            _ => (),
        })
        .expect("error running application event loop");
}

fn main() {
    let mut args = Arguments::from_env();
    let arg1: OsString = args.free_from_str().unwrap();
    pollster::block_on(run(arg1))
}
