use std::{env, fs};

use std::sync::Arc;
use std::cmp::{max, min};
use std::thread::JoinHandle;
use crossbeam::select;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use wgpu::util::DeviceExt;
use crossbeam::channel::{Receiver, Sender, bounded, unbounded};

pub mod utils;
use utils::{Vertex, Location, MainCommand, ContentCommand};

pub mod content;
use content::{Content, object::GPUObject};

use crate::utils::lori::{Lori, keycodes_transformer};
use crate::utils::{ContentLrxCommand, ContentLtxCommand, GPUPrimitives, LoriToMainCall, LoriToMainCommand, MainToLoriCall, MainToLoriCommand, Primitive};



#[repr(C)]
#[derive(Copy, Clone, Debug, serde::Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUView {
    pub time: [f32; 2],
    pub scale: [f32; 2],
    pub position: [f32; 2],
    pub rotation: [f32; 2],
}

impl GPUView {
    pub fn new() -> Self {
        Self {
            time: [0., 0.],
            scale: [1., 1.],
            position: [0., 0.],
            rotation: [0., 0.],
        }
    }
}



struct State {
    init_time: chrono::DateTime<chrono::Utc>,
    last_time: chrono::DateTime<chrono::Utc>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    msaa_view: wgpu::TextureView,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    primitive_pipeline: wgpu::RenderPipeline,

    objects: Vec<GPUObject>,
    primitives: Vec<Primitive>,

    window: Arc<Window>,
    window_scale: [f32; 4],
    gpu_view: GPUView,
    gpu_view_buffer: wgpu::Buffer,
    gpu_view_bind_group: wgpu::BindGroup,

    primitive_buffer: wgpu::Buffer,
    primitive_bind_group: wgpu::BindGroup,
    
    content_tx: Sender<ContentCommand>,
    main_rx: Receiver<MainCommand>,
    lori_call: Sender<MainToLoriCall>,
    lori_back: Receiver<LoriToMainCall>,
    lori_cmd: Receiver<LoriToMainCommand>,
    lori_rtrn: Sender<MainToLoriCommand>,
    lori_handle: Option<JoinHandle<()>>,
    content_handle: Option<JoinHandle<()>>,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let args: Vec<String> = env::args().collect();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            backend_options: wgpu::BackendOptions::default(),
            display: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.unwrap();
        
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let (main_tx, main_rx) = unbounded::<MainCommand>();
        let (content_tx, content_rx) = unbounded::<ContentCommand>();

        let (main_cmd, lori_cmd) = unbounded::<LoriToMainCommand>();
        let (lori_rtrn, main_rtrn) = unbounded::<MainToLoriCommand>();
        let (lori_call, main_call) = bounded::<MainToLoriCall>(0);
        let (main_back, lori_back) = bounded::<LoriToMainCall>(0);

        let (content_ltx, c_lori_lrx) = unbounded::<ContentLtxCommand>();
        let (c_lori_ltx, content_lrx) = unbounded::<ContentLrxCommand>();
        

        let lua_code: String = fs::read_to_string(args[1].clone()).unwrap();
        let mut lori: Lori = Lori::new(lua_code, main_cmd, main_rtrn, main_call, main_back, content_ltx, content_lrx);
        let lori_handle = Some(std::thread::Builder::new()
            .name("lori".to_string())
            .spawn(move || { lori.begin(); }).unwrap());
        
        _= lori_call.send(MainToLoriCall::Load);
        lori_back.recv().unwrap();
        while let Ok(cmd) = lori_cmd.try_recv() {
            match cmd {
                LoriToMainCommand::SetWindowTitle { text } => {
                    _= window.set_title(text.as_str());
                },
                LoriToMainCommand::SetWindowSize { w, h } => {
                    _= window.request_inner_size(PhysicalSize { width: w, height: h });
                },
                LoriToMainCommand::SetWindowResizable { is } => {
                    window.set_resizable(is);
                },
                _ => {}
            }
        }
        println!("Beans");
        
        let mut content = Content::create(main_tx, content_rx, c_lori_ltx, c_lori_lrx);

        let mut objects: Vec<GPUObject> = Vec::new();
        
        let content_handle = Some(std::thread::Builder::new()
            .name("content".to_string())
            .spawn(move || { content.thread_loop(); }).unwrap());
        

        let mut gpu_view: GPUView = GPUView::new();
        let min: f32 = min(size.width, size.height) as f32;
        let max: f32 = max(size.width, size.height) as f32;
        gpu_view.scale = [size.height as f32 / max, size.width as f32 / max];
        
        let window_scale: [f32; 4] = [(size.width as f32 - min) / 2., (size.height as f32 - min) / 2., min, min];
        
        let gpu_view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Viewport Buffer"),
            contents: bytemuck::cast_slice(&[gpu_view]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let gpu_view_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Viewport Bind Group Layout"),
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
        });

        let gpu_view_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Viewport Bind Group"),
            layout: &gpu_view_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_view_buffer.as_entire_binding(),
                },
            ],
        });

        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa color texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        
        
        let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let raster_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/main.wgsl").into());
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layout for Primary Render Pipeline"),
            bind_group_layouts: &[Some(gpu_view_bind_layout).as_ref()],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Primary Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &raster_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc()), Some(Location::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &raster_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let primitives: Vec<Primitive> = Vec::with_capacity(200);

        let primitive_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Primitive Buffer"),
            size: ((12304)) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let primitive_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Primitives Bind Group Layout"),
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
            ]
        });
        
        let primitive_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Primitives Bind Group"),
            layout: &primitive_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: primitive_buffer.as_entire_binding(),
            }]
        });
        
        let primitive_shader = device.create_shader_module(wgpu::include_wgsl!("./shaders/prim.wgsl").into());
        let primitive_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Layout for Primary Render Pipeline"),
            bind_group_layouts: &[Some(primitive_bind_layout).as_ref()],
            immediate_size: 0,
        });

        let primitive_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Primary Render Pipeline"),
            layout: Some(&primitive_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &primitive_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &primitive_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint32),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },

            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let mut state = State {
            init_time: chrono::Utc::now(),
            last_time: chrono::Utc::now(),
            surface,
            surface_format,
            msaa_view,
            device,
            queue,
            size,
            render_pipeline,
            primitive_pipeline,

            objects,
            primitives,

            window,
            window_scale,
            gpu_view,
            gpu_view_buffer,
            gpu_view_bind_group,

            primitive_buffer,
            primitive_bind_group,
            
            content_tx,
            main_rx,
            lori_call,
            lori_back,
            lori_cmd,
            lori_rtrn,
            lori_handle,
            content_handle,
        };

        state.configure_surface();
        state
    }

    fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&mut self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            color_space: wgpu::SurfaceColorSpace::default(),
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);


        let msaa_texture = &self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Texture"),
            size: wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        
        self.msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
        let size: [u32; 2] = [self.size.width, self.size.height];
        let min: f32 = min(size[0], size[1]) as f32;
        let max: f32 = max(size[0], size[1]) as f32;
        
        self.gpu_view.scale = [size[1] as f32 / max, size[0] as f32 / max];
        self.window_scale = [(size[0] as f32 - min) / 2., (size[1] as f32 - min) / 2., min, min];
    }
    
    fn keyboard_inputs(&mut self, code: winit::keyboard::KeyCode, state: bool) {
        if state {
            _= self.lori_call.send(MainToLoriCall::Keypressed { code: keycodes_transformer(code) });
            loop {
                select! {
                    recv(self.lori_cmd) -> cmd => {
                        if let Ok(v) = cmd {
                            match v {
                                LoriToMainCommand::GetWindowSize => {
                                    _= self.lori_rtrn.send(MainToLoriCommand::GetWindowSize { w: self.size.width, h: self.size.height });
                                }
                                _ => {}
                            }
                        }
                    }
                    recv(self.lori_back) -> _ => {
                        break
                    }
                }
            }
        } else {
            _= self.lori_call.send(MainToLoriCall::Keyreleased { code: keycodes_transformer(code) });
            loop {
                select! {
                    recv(self.lori_cmd) -> cmd => {
                        if let Ok(v) = cmd {
                            match v {
                                LoriToMainCommand::GetWindowSize => {
                                    _= self.lori_rtrn.send(MainToLoriCommand::GetWindowSize { w: self.size.width, h: self.size.height });
                                }
                                _ => {}
                            }
                        }
                    }
                    recv(self.lori_back) -> _ => {
                        break
                    }
                }
            }
        }
    }

    fn render(&mut self) {
        let surface_texture = self.surface.get_current_texture();
        
        let pretexture_view = match surface_texture {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => return
        };
        let texture_view = pretexture_view.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.surface_format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.msaa_view,
                depth_slice: None,
                resolve_target: Some(&texture_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        renderpass.set_pipeline(&self.render_pipeline);
        
        _= self.content_tx.send(ContentCommand::Render);
        if let Ok(cmd) = self.main_rx.recv() {
            match cmd {
                MainCommand::Render { instances, camera } => {
                    for i in 0..self.objects.len() {
                        self.queue.write_buffer(&self.objects[i].location_buffer, 0, bytemuck::cast_slice(&instances[i]));
                    }

                    self.gpu_view.position[0] = camera.position.x;
                    self.gpu_view.position[1] = camera.position.y;
                    self.gpu_view.rotation[0] = camera.rotation;
                },
            }
        }
        
        self.gpu_view.time[0] = chrono::Utc::now().signed_duration_since(self.init_time).as_seconds_f32();
        self.gpu_view.time[1] = chrono::Utc::now().signed_duration_since(self.last_time).as_seconds_f32();
        self.queue.write_buffer(&self.gpu_view_buffer, 0, bytemuck::bytes_of(&[self.gpu_view]));

        renderpass.set_bind_group(0, &self.gpu_view_bind_group, &[]);
        
        for i in 0..self.objects.len() { // Draw each shape
            renderpass.set_vertex_buffer(0, self.objects[i].vertex_buffer.slice(..));
            renderpass.set_vertex_buffer(1, self.objects[i].location_buffer.slice(..));
            renderpass.set_index_buffer(self.objects[i].index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            renderpass.draw_indexed(0..self.objects[i].indices.len() as u32, 0, 0..self.objects[i].locations.len() as _);
        }

        
        renderpass.set_pipeline(&self.primitive_pipeline);

        _= self.lori_call.send(MainToLoriCall::Render);
        loop {
            select! {
                recv(self.lori_cmd) -> cmd => {
                    if let Ok(v) = cmd {
                        match v {
                            LoriToMainCommand::SetWindowTitle { text } => {
                                self.window.set_title(text.as_str());
                            },
                            LoriToMainCommand::SetWindowSize { w, h } => {
                                // if let Some(_) = self.window.request_inner_size(PhysicalSize { width: w, height: h }) {
                                //     self.resize(PhysicalSize { width: w, height: h });
                                // } // TODO: Wait...
                                _= self.window.request_inner_size(PhysicalSize { width: w, height: h });
                            },
                            LoriToMainCommand::SetWindowResizable { is } => {
                                _= self.window.set_resizable(is);
                            },
                            LoriToMainCommand::GetWindowSize => {
                                let size: PhysicalSize<u32> = self.window.inner_size();
                                _= self.lori_rtrn.send(MainToLoriCommand::GetWindowSize { w: size.width, h: size.height });
                            },
                            LoriToMainCommand::DrawRect { x, y, w, h, r, color } => {
                                self.primitives.push(Primitive { xywh: [x, y, w, h], angle: r, label: 0, _pad0: 0, _pad1: 0, color });
                            }
                            _ => {}
                        }
                    }
                }
                recv(self.lori_back) -> _ => {
                    while let Ok(cmd) = self.lori_cmd.try_recv() {
                        match cmd {
                            LoriToMainCommand::SetWindowTitle { text } => {
                                self.window.set_title(text.as_str());
                            },
                            LoriToMainCommand::SetWindowSize { w, h } => {
                                // if let Some(_) = self.window.request_inner_size(PhysicalSize { width: w, height: h }) {
                                //     self.resize(PhysicalSize { width: w, height: h });
                                // } // TODO: Wait...
                                _= self.window.request_inner_size(PhysicalSize { width: w, height: h });
                            },
                            LoriToMainCommand::SetWindowResizable { is } => {
                                _= self.window.set_resizable(is);
                            },
                            LoriToMainCommand::GetWindowSize => {
                                let size: PhysicalSize<u32> = self.window.inner_size();
                                _= self.lori_rtrn.send(MainToLoriCommand::GetWindowSize { w: size.width, h: size.height });
                            },
                            LoriToMainCommand::DrawRect { x, y, w, h, r, color } => {
                                self.primitives.push(Primitive { xywh: [x, y, w, h], angle: r, label: 0, _pad0: 0, _pad1: 0, color });
                            }
                            _ => {}
                        }
                    }
                    
                    break;
                }
            }
        }
        
        
        let mut primitive_box: GPUPrimitives = GPUPrimitives::from_vec(self.primitives.len() as u32, &self.primitives);
        primitive_box.scale = [self.size.width as f32, self.size.height as f32];
        self.primitives.clear();

        self.queue.write_buffer(&self.primitive_buffer, 0, &bytemuck::bytes_of(&[primitive_box]));
        renderpass.set_bind_group(0, &self.primitive_bind_group, &[]);
        renderpass.draw(0..3, 0..1); // TODO: Only do if any primitive drawing function is called
        
        self.last_time = chrono::Utc::now();
        drop(renderpass);

        
        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(pretexture_view);
    }

    fn exit(&mut self) {
        _= self.lori_call.send(MainToLoriCall::Exit);
        _= self.content_tx.send(ContentCommand::Exit);
        if let Some(join_handle) = self.lori_handle.take() {
            _= join_handle.join();
        };

        if let Some(join_handle) = self.content_handle.take() {
            _= join_handle.join();
        };
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let superstate = self.state.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested! Exiting application...");
                superstate.exit();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                superstate.render();
                superstate.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                superstate.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => superstate.keyboard_inputs(code, key_state.is_pressed()),
            _ => (),
            
        }
    }
}

fn main() {
    let events = EventLoop::new().unwrap();
    events.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    match events.run_app(&mut app) {
        Ok(()) => println!("Exited successfully."),
        Err(error) => eprintln!("Exited with an error:\n {error:?}"),
    }
}
