use rapier2d::{dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderSet}, math::Vec2};

use wgpu::util::DeviceExt;
use crate::utils::{Location, Vertex};

pub struct GPUObject {
    pub vertex_buffer: wgpu::Buffer,
    pub location_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub vertices: Box<[Vertex]>,
    pub indices: Box<[u32]>,
    pub locations: Vec<Location>,
}

impl GPUObject {
    pub fn new(device: &wgpu::Device, vertices: Box<[Vertex]>, indices: Box<[u32]>, locations: Vec<Location>) -> Self {
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let location_buffer = device.create_buffer(&wgpu::BufferDescriptor { // TODO: Remember to EXPAND for more objects!!!! Use for loop to dynamically make larger pls :)
            label: Some("Location Buffer"),
            size: (size_of::<Location>() * 200) as u64, // Max 200 instances per car
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            location_buffer,
            vertices,
            indices,
            locations,
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, rotation: f32) {
        self.locations.push(Location {position: [x, y], rotation: [rotation, 0.]})
    }
}

pub struct Object {
    pub vertices: Box<[Vertex]>,
    pub indices: Box<[u32]>,

    pub rigidhandles: Vec<RigidBodyHandle>,
    pub hull: Option<ColliderBuilder>,
}

impl Object {
    pub fn new(vertices: Box<[Vertex]>, indices: Box<[u32]>, physical: bool) -> Self {
        let rigidhandles: Vec<RigidBodyHandle> = Vec::new();

        let mut points: Vec<Vec2> = Vec::new();
        for vertex in vertices.iter() {
            points.push(Vec2{
                x: vertex.position[0],
                y: vertex.position[1],
            })
        }

        let mut hull: Option<ColliderBuilder> = None;
        if physical {
            hull = Some(ColliderBuilder::convex_hull(&points.clone().into_boxed_slice()).unwrap()
                .restitution(0.2)
                .friction(0.2)); // TODO: Make it accessible via Lua
        }
        
        Self {
            vertices,
            indices,
            rigidhandles,
            hull,
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, rotation: f32, rigidbodies: &mut RigidBodySet, colliders: &mut ColliderSet) {
        let rb = RigidBodyBuilder::dynamic()
            .translation(Vec2 { x, y })
            .rotation(rotation)
            .build();
        let rb_handle = rigidbodies.insert(rb);
        self.rigidhandles.push(rb_handle);

        if let Some(hullshape) = self.hull.as_mut() {
            colliders.insert_with_parent(hullshape.clone(), rb_handle, rigidbodies);
        }
    }
}