use crossbeam::{channel::{Receiver, Sender, bounded, unbounded}, select};
use std::{cmp::{max, min}, env, fs, process::exit, sync::Arc, thread::JoinHandle};
use rapier2d::prelude::*;
use winit::{application::ApplicationHandler, event::MouseScrollDelta};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, MouseButton, MouseScrollDelta::{LineDelta, PixelDelta}, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::SmolStr;
use winit::window::{Window, WindowId};
use wgpu::util::DeviceExt;

pub mod content;
use content::{object::GPUObject, object::Object};
pub mod utils;
use crate::utils::{GPUPrimitives, Location, lori::Lori, LoriToMainCall, LoriToMainCommand, MainToLoriCall, MainToLoriCommand, Primitive, Vertex};





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
    current_time: chrono::DateTime<chrono::Utc>,
    last_time: chrono::DateTime<chrono::Utc>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    msaa_view: wgpu::TextureView,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    mouse: (f32, f32),
    keys: Vec<String>,
    render_pipeline: wgpu::RenderPipeline,
    primitive_pipeline: wgpu::RenderPipeline,

    gpu_objects: Vec<GPUObject>,
    objects: Vec<Object>,
    primitives: Vec<Primitive>,

    window: Arc<Window>,
    window_scale: [f32; 4],
    gpu_view: GPUView,
    gpu_view_buffer: wgpu::Buffer,
    gpu_view_bind_group: wgpu::BindGroup,

    primitive_buffer: wgpu::Buffer,
    primitive_bind_group: wgpu::BindGroup,

    gravity: Vec2, // TODO: allow control from Lua
    integration_parameters: IntegrationParameters,
    physics: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    rigidbodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,

    lori_call: Sender<MainToLoriCall>,
    lori_back: Receiver<LoriToMainCall>,
    lori_cmd: Receiver<LoriToMainCommand>,
    lori_rtrn: Sender<MainToLoriCommand>,
    lori_handle: Option<JoinHandle<()>>,
}

impl State {
    async fn new(window: Arc<Window>) -> State {
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 {
            eprintln!("lori: 'lori <path to .lua>'");
            exit(1);
        }

        let lua_code: String;
        match fs::read_to_string(args[1].clone()) {
            Ok(file) => {
                lua_code = file;
            }
            Err(e) => {
                eprintln!("lori (EROR): {}", e); // EROR, INFO, DBUG, VBOS
                exit(e.raw_os_error().unwrap_or_default());
            }
        }
        
        let mouse: (f32, f32) = (0., 0.);
        let keys: Vec<String> = Vec::new();

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

        let (main_cmd, lori_cmd) = unbounded::<LoriToMainCommand>();
        let (lori_rtrn, main_rtrn) = unbounded::<MainToLoriCommand>();
        let (lori_call, main_call) = bounded::<MainToLoriCall>(0);
        let (main_back, lori_back) = bounded::<LoriToMainCall>(0);


        let mut lori: Lori = Lori::new(lua_code, main_cmd, main_rtrn, main_call, main_back);
        let lori_handle = Some(std::thread::Builder::new()
            .name("lori".to_string())
            .spawn(move || { lori.begin(); }).unwrap());
        
        let gpu_objects: Vec<GPUObject> = Vec::new();
        let objects: Vec<Object> = Vec::new();
        

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

        let gravity = Vec2 { x: 0., y: 0. };
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1./60.;
        
        let physics = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhaseBvh::new();
        let narrow_phase = NarrowPhase::new();
        let rigidbodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let impulse_joints = ImpulseJointSet::new();
        let multibody_joints = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();

        let mut state = State {
            current_time: chrono::Utc::now(),
            last_time: chrono::Utc::now(),
            surface,
            surface_format,
            msaa_view,
            device,
            queue,
            size,
            mouse,
            keys,
            render_pipeline,
            primitive_pipeline,

            gpu_objects,
            objects,
            primitives,

            window,
            window_scale,
            gpu_view,
            gpu_view_buffer,
            gpu_view_bind_group,

            primitive_buffer,
            primitive_bind_group,

            gravity,
            integration_parameters,
            physics,
            island_manager,
            broad_phase,
            narrow_phase,
            rigidbodies,
            colliders,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            
            lori_call,
            lori_back,
            lori_cmd,
            lori_rtrn,
            lori_handle,
        };

        _= state.lori_call.send(MainToLoriCall::Load);
        state.handle_lori_loop();
        
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
            present_mode: wgpu::PresentMode::AutoNoVsync,
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
    
    fn keyboard_inputs(&mut self, key: String, state: bool) {
        if state {
            self.keys.push(key.clone());
            _= self.lori_call.send(MainToLoriCall::Keypressed { code: key });
        } else {
            self.keys.retain(|k| k != &key);
            _= self.lori_call.send(MainToLoriCall::Keyreleased { code: key });
        }
        self.handle_lori_loop();
    }

    fn mouse_button_inputs(&mut self, button: MouseButton, state: bool) {
        let numerical_button: u32;
        match button {
            MouseButton::Left => {
                numerical_button = 1;
            }
            MouseButton::Right => {
                numerical_button = 2;
            }
            MouseButton::Middle => {
                numerical_button = 3;
            }
            MouseButton::Back => {
                numerical_button = 4;
            }
            MouseButton::Forward => {
                numerical_button = 5;
            }
            MouseButton::Other(num) => {
                numerical_button = (6+num) as u32;
            }
        }
        if state {
            _= self.lori_call.send(MainToLoriCall::Mousepressed { x: self.mouse.0, y: self.mouse.1, button: numerical_button });
        } else {
            _= self.lori_call.send(MainToLoriCall::Mousereleased { x: self.mouse.0, y: self.mouse.1, button: numerical_button });
        }
        self.handle_lori_loop();
    }

    fn mouse_movement_inputs(&mut self, motion: (f64, f64)) {
        let simple_motion: (f32, f32) = (motion.0 as f32, motion.1 as f32);
        _= self.lori_call.send(MainToLoriCall::MouseMoved { motion: simple_motion });
        self.handle_lori_loop();
    }

    fn mouse_scroll_inputs(&mut self, delta: MouseScrollDelta) {
        let simple_motion: (f32, f32);
        match delta {
            PixelDelta(position) => {
                simple_motion = (position.x as f32, position.y as f32);
            }
            LineDelta(x, y) => {
                simple_motion = (x, y);
            }
        }
        _= self.lori_call.send(MainToLoriCall::MouseScrolled { motion: simple_motion });
        self.handle_lori_loop();
    }

    fn render(&mut self) {
        self.current_time = chrono::Utc::now();
        self.integration_parameters.dt = self.current_time.signed_duration_since(self.last_time).as_seconds_f32();
        
        self.physics.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );


        _= self.lori_call.send(MainToLoriCall::Update { delta: self.integration_parameters.dt });
        self.handle_lori_loop();
    


        
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
        
        self.gpu_view.time[0] = chrono::Utc::now().signed_duration_since(self.current_time).as_seconds_f32();
        self.gpu_view.time[1] = chrono::Utc::now().signed_duration_since(self.last_time).as_seconds_f32();
        self.queue.write_buffer(&self.gpu_view_buffer, 0, bytemuck::bytes_of(&[self.gpu_view]));

        renderpass.set_bind_group(0, &self.gpu_view_bind_group, &[]);
        
        for i in 0..self.gpu_objects.len() { // Draw each shape
            renderpass.set_vertex_buffer(0, self.gpu_objects[i].vertex_buffer.slice(..));
            renderpass.set_vertex_buffer(1, self.gpu_objects[i].location_buffer.slice(..));
            renderpass.set_index_buffer(self.gpu_objects[i].index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            renderpass.draw_indexed(0..self.gpu_objects[i].indices.len() as u32, 0, 0..self.gpu_objects[i].locations.len() as _);
        }

        
        renderpass.set_pipeline(&self.primitive_pipeline);

        _= self.lori_call.send(MainToLoriCall::Render);
        self.handle_lori_loop();
        
        
        let mut primitive_box: GPUPrimitives = GPUPrimitives::from_vec(self.primitives.len() as u32, &self.primitives);
        primitive_box.scale = [self.size.width as f32, self.size.height as f32];
        self.primitives.clear();

        self.queue.write_buffer(&self.primitive_buffer, 0, &bytemuck::bytes_of(&[primitive_box]));
        renderpass.set_bind_group(0, &self.primitive_bind_group, &[]);
        renderpass.draw(0..3, 0..1); // TODO: Only do if any primitive drawing function is called
        
        drop(renderpass);

        
        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(pretexture_view);

        self.last_time = self.current_time;
    }

    fn handle_lori_commands(&mut self, v: LoriToMainCommand) {
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
                _= self.lori_rtrn.send(MainToLoriCommand::ReturnWindowSize { w: self.size.width, h: self.size.height });
            },
            LoriToMainCommand::GetKeyPressed { key } => {
                _= self.lori_rtrn.send(MainToLoriCommand::ReturnKeyPressed { key: self.keys.contains(&key) });
            },
            LoriToMainCommand::DrawPrimitive { x, y, w, h, r, color, label } => {
                self.primitives.push(Primitive { xywh: [x, y, w, h], angle: r, label, _pad0: 0, _pad1: 0, color });
            }
        }
    }
    
    fn handle_lori_loop(&mut self) {
        loop {
            select! {
                recv(self.lori_cmd) -> cmd => {
                    if let Ok(v) = cmd {
                        self.handle_lori_commands(v);
                    }
                }
                recv(self.lori_back) -> _ => {
                    while let Ok(cmd) = self.lori_cmd.try_recv() {
                        self.handle_lori_commands(cmd);
                    }
                    
                    break;
                }
            }
        }
    }

    fn exit(&mut self) {
        _= self.lori_call.send(MainToLoriCall::Exit);
        if let Some(join_handle) = self.lori_handle.take() {
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
            WindowEvent::KeyboardInput { event, .. } => {
                let newtext: String = event.text.unwrap_or_else(|| SmolStr::new("NONE")).to_string();
                superstate.keyboard_inputs(newtext, event.state.is_pressed());
            }
            WindowEvent::MouseInput { state, button, .. } => {
                superstate.mouse_button_inputs(button, state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                superstate.mouse = (position.x as f32, position.y as f32);
            }
            _ => (),
        }
    }
    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        let superstate = self.state.as_mut().unwrap();

        match event {
            DeviceEvent::MouseMotion { delta } => {
                superstate.mouse_movement_inputs(delta);
            }
            DeviceEvent::MouseWheel { delta } => {
                superstate.mouse_scroll_inputs(delta);
            }
            _ => {}
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
