use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, Queue, RequestAdapterOptionsBase, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct GraphicsContext<'s> {
    device: Device,
    surface: Surface<'s>,
    config: SurfaceConfiguration,
    queue: Queue,
}

impl<'s> GraphicsContext<'s> {
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
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
    let config = surface
        .get_default_config(&adapter, inner_size.width, inner_size.height)
        .expect("graphics adapter is not compatible");

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
        surface,
        config,
        device,
        queue,
    }
}

fn graphics_draw(ctx: &mut GraphicsContext) {
    let (frame, mut commands) = ctx.begin_frame();
    let view = frame.texture.create_view(&TextureViewDescriptor::default());

    commands.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 1.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    ctx.submit_frame(frame, commands);
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("GPU Playground")
        .with_min_inner_size(PhysicalSize::new(64, 64))
        .build(&event_loop)
        .unwrap();

    let mut graphics_ctx = pollster::block_on(graphics_init(&window));

    let window = &window;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => graphics_ctx.resize(size),
                WindowEvent::RedrawRequested => graphics_draw(&mut graphics_ctx),
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => (),
        })
        .expect("error running application event loop");
}
