use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use buffer::{to_slice, BufferUniforms};
use pico_args::Arguments;
use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, FragmentState,
    Instance, InstanceDescriptor, Limits, MultisampleState, PipelineLayoutDescriptor,
    PowerPreference, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptionsBase, ShaderModule, ShaderModuleDescriptor, ShaderStages, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureViewDescriptor, VertexState,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod buffer;
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
    vs_module: ShaderModule,
    uniform: Buffer,
    state: Option<GraphicsState>,
}

struct GraphicsState {
    pipeline: RenderPipeline,
    uniform_bind: BindGroup,
}

impl<'s> GraphicsContext<'s> {
    pub fn state(&self) -> &Option<GraphicsState> {
        &self.state
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

        let uniform_bind_layout =
            self.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let uniform_bind = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: self.uniform.as_entire_binding(),
            }],
        });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&uniform_bind_layout],
                push_constant_ranges: &[],
            });

        let pipeline = self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &self.vs_module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: "main",
                    targets: &[Some(format.into())],
                }),
                depth_stencil: None,
                primitive: PrimitiveState::default(),
                multisample: MultisampleState::default(),
                multiview: None,
            });

        self.state = Some(GraphicsState {
            pipeline,
            uniform_bind,
        });
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

    pub fn write_uniforms(&mut self, uniforms: &BufferUniforms) {
        self.queue
            .write_buffer(&self.uniform, 0, unsafe { to_slice(uniforms) });
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

    let vs_module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("vs_module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/vert.wgsl").into()),
    });

    let uniform = device.create_buffer(&BufferDescriptor {
        label: Some("uniform"),
        size: std::mem::size_of::<BufferUniforms>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    GraphicsContext {
        adapter,
        surface,
        config,
        device,
        queue,
        vs_module,
        uniform,
        state: None,
    }
}

fn graphics_draw(ctx: &mut GraphicsContext, uniforms: &BufferUniforms) {
    let (frame, mut commands) = ctx.begin_frame();
    let view = frame.texture.create_view(&TextureViewDescriptor::default());
    ctx.write_uniforms(uniforms);

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

        if let Some(state) = ctx.state() {
            render_pass.set_pipeline(&state.pipeline);
            render_pass.set_bind_group(0, &state.uniform_bind, &[]);
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
    let window_size = window.inner_size();
    let mut uniforms = BufferUniforms::default();
    uniforms.resolution = [window_size.width as f32, window_size.height as f32, 1.];

    let mut start_time = Instant::now();
    let mut last_frame = Instant::now();
    let mut last_frame_update = Instant::now();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => {
                    uniforms.resolution = [size.width as f32, size.height as f32, 1.];
                    graphics_ctx.resize(size);
                }
                WindowEvent::RedrawRequested => {
                    uniforms.time = start_time.elapsed().as_secs_f32();
                    uniforms.delta_time = last_frame.elapsed().as_secs_f32();
                    uniforms.frame += 1;

                    graphics_draw(&mut graphics_ctx, &uniforms);

                    if last_frame_update.elapsed() > FPS_UPDATE_RATE {
                        let fps = 1. / last_frame.elapsed().as_secs_f64();
                        window.set_title(&format!("{} - {:.0} FPS", window_title, fps));
                        last_frame_update = Instant::now();
                    }
                    last_frame = Instant::now();
                }
                WindowEvent::KeyboardInput { event, .. } if event.state.is_pressed() => {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => elwt.exit(),
                        _ => (),
                    }
                }
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();

                match rx.try_recv() {
                    Ok(watch::FileReloadNotification) => {
                        println!("reloading shader module...");
                        graphics_ctx.add_shader_module(&shader_path);
                        start_time = Instant::now();
                        uniforms.frame = 0;
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
