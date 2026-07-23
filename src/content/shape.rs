use crossbeam::channel::Sender;
use mlua::{UserData, UserDataMethods};

use crate::utils::{LoriToMainCommand, Vertex};

#[derive(Clone)]
pub struct LoriShape {
    pub uid: u64,
    pub tx: Sender<LoriToMainCommand>,
}

impl UserData for LoriShape {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("test", |_, this, text| {
            _= this.tx.send(LoriToMainCommand::ShapeTest { uid: this.uid, text });
            Ok(())
        });
    }
}

pub struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Shape {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
        }
    }
}