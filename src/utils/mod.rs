pub mod transform;
pub mod lori;

use transform::Vector2;

pub enum LfnCommand {
    CreateObject {
        x: f32,
        y: f32,
        rotation: f32,
    },
}

#[derive(Clone)]
pub enum ContentCommand {
    Render,
    Exit,
}

#[derive(Clone)]
pub enum MainCommand {
    Render {
        instances: Vec<Vec<Location>>,
        camera: Displacement,
    },
    CreateObject {
        x: f32,
        y: f32,
        rotation: f32,
    },
}

pub struct Xy {
    pub position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, serde::Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Location {
    pub position: [f32; 2],
    pub rotation: [f32; 2],
}

impl Location {
    pub fn new() -> Self {
        Self {
            position: [0., 0.],
            rotation: [0., 0.],
        }
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Location>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}


#[derive(Clone, Copy)]
pub struct Displacement {
    pub position: Vector2,
    pub velocity: Vector2,
    pub rotation: f32,
}

impl Displacement {
    pub fn new() -> Self {
        Self {
            position: Vector2::new(),
            velocity: Vector2::new(),
            rotation: 0.,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Wheel {
    pub position: [f32; 2],
    pub rotation: f32,
    pub angle: f32,
    pub base: f32,
    pub diameter: f32,
    pub friction: f32,
    pub limits: [f32; 2],
    pub power: f32,
}

pub fn keycodes_transformer(code: winit::keyboard::KeyCode) -> &'static str {
    match code {
        winit::keyboard::KeyCode::KeyA => "a",
        _ => "NONE",
    }
}