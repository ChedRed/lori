use std::process::exit;

use crossbeam::channel::{Sender, Receiver};
use mlua::{UserData, UserDataMethods};
use rapier2d::{dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet}, geometry::{ColliderBuilder, ColliderSet}, math::Vec2, utils::{PoseOps, RotationOps}};

use wgpu::{Queue, naga::{BuiltIn::Vertices, FastHashMap}, util::DeviceExt};
use crate::{content::{collider::LoriCollider, shape::LoriShape}, utils::{Location, LoriToMainCommand, MainToLoriCommand}};


#[derive(Clone)]
pub struct LoriSpawnerRef {
    pub uid: u64,
    pub tx: Sender<LoriToMainCommand>,
    pub rx: Receiver<MainToLoriCommand>,
}

impl UserData for LoriSpawnerRef {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("spawn", |_, this, (x, y, r)| {
            _= this.tx.send(LoriToMainCommand::SpawnerSpawn { uid: this.uid, x, y, r });
            let mut real_object: Option<LoriObjectRef> = None;
            while let Ok(cmd) = this.rx.recv() {
                match cmd {
                    MainToLoriCommand::ReturnNewObject { object } => {
                        real_object = Some(object);
                        break;
                    }
                    _ => {}
                }
            }
            Ok(real_object)
        });
    }
}

#[derive(Clone)]
pub struct LoriObjectRef {
    pub puid: u64,
    pub uid: u64,
    pub tx: Sender<LoriToMainCommand>,
}

impl UserData for LoriObjectRef {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("move", |_, this, (x, y)| {
            _= this.tx.send(LoriToMainCommand::ObjectMove { puid: this.puid, uid: this.uid, x, y });
            Ok(())
        });
        methods.add_method("push", |_, this, (x, y)| {
            _= this.tx.send(LoriToMainCommand::ObjectPush { puid: this.puid, uid: this.uid, x, y });
            Ok(())
        });
        methods.add_method("pull", |_, this, (x1, y1, x2, y2)| {
            _= this.tx.send(LoriToMainCommand::ObjectPull { puid: this.puid, uid: this.uid, x1, y1, x2, y2 });
            Ok(())
        });
    }
}

pub struct LoriSpawner {
    count: u64,
    
    pub indices: u32,
    pub locations: FastHashMap<u64, Location>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub location_buffer: Option<wgpu::Buffer>,
    render: bool,
    
    pub hull: Option<ColliderBuilder>,
    pub rigidhandles: FastHashMap<u64, RigidBodyHandle>,
    collision: String,
    collide: bool,
}

impl LoriSpawner {
    pub fn new(device: &wgpu::Device, shape: Option<LoriShape>, collider: Option<LoriCollider>) -> Self {
        let count: u64 = 0;
        
        let mut points: Vec<Vec2> = Vec::new();
        let mut hull: Option<ColliderBuilder> = None;
        let mut center: Option<Vec2> = None;
        let rigidhandles: FastHashMap<u64, RigidBodyHandle> = FastHashMap::default();
        let mut collide: bool = false;
        let mut collision: String = "static".to_string();
        if let Some(real_collider) = collider {
            collision = real_collider.collision;
            collide = true;
            for vertex in real_collider.vertices.iter() {
                points.push(Vec2{
                    x: vertex.position[0],
                    y: vertex.position[1],
                })
            }

            hull = Some(ColliderBuilder::convex_hull(&points.clone().into_boxed_slice()).unwrap()
                .restitution(0.2)
                .friction(0.2)
                .density(0.001)); // TODO: Make it accessible via Lua
            center = Some(hull.clone().unwrap().build().mass_properties().local_com);
        }

        let mut indices: u32 = 0;
        let locations: FastHashMap<u64, Location> = FastHashMap::default();
        let mut index_buffer: Option<wgpu::Buffer> = None;
        let mut vertex_buffer: Option<wgpu::Buffer> = None;
        let mut location_buffer: Option<wgpu::Buffer> = None;
        let mut render: bool = false;
        if let Some(real_shape) = shape {
            let mut modified_shape = real_shape.clone();
            
            render = true;
            indices = real_shape.indices.len() as u32;
            index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&real_shape.indices),
                usage: wgpu::BufferUsages::INDEX,
            }));

            for vertex in modified_shape.vertices.iter_mut() {
                vertex.position[0] -= center.unwrap().x;
                vertex.position[1] -= center.unwrap().y;
            }
            
            vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&modified_shape.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }));
            
            location_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor { // TODO: Replace 200 with a reasonable number
                label: Some("Location Buffer"),
                size: (size_of::<Location>() * 200) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        
        Self {
            count,
            
            indices,
            locations,
            vertex_buffer,
            index_buffer,
            location_buffer,
            render,
            
            hull,
            rigidhandles,
            collision,
            collide,
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, rotation: f32, rigidbodies: &mut RigidBodySet, colliders: &mut ColliderSet) -> u64 {
        if self.render {
            self.locations.insert(self.count, Location {position: [x, y], rotation: [rotation, 0.]});
        }
        if self.collide {
            let rb: RigidBody;
            if self.collision == "static" {
                rb = RigidBodyBuilder::fixed()
                    .translation(Vec2 { x, y })
                    .rotation(rotation)
                    .build();
            } else if self.collision == "diaxial" {
                rb = RigidBodyBuilder::dynamic()
                    .translation(Vec2 { x, y })
                    .rotation(rotation)
                    .lock_rotations()
                    .build();
            } else {
                rb = RigidBodyBuilder::dynamic()
                    .translation(Vec2 { x, y })
                    .rotation(rotation)
                    .build();
            }
            let rb_handle = rigidbodies.insert(rb);
            self.rigidhandles.insert(self.count, rb_handle);
    
            if let Some(hullshape) = self.hull.as_mut() {
                colliders.insert_with_parent(hullshape.clone(), rb_handle, rigidbodies);
            }
        }
        self.count += 1;
        return self.count - 1;
    }

    pub fn renderable(&self) -> bool {
        self.render
    }

    pub fn collidable(&self) -> bool {
        self.collide
    }
}