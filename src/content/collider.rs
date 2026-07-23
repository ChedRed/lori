use crossbeam::channel::Sender;
use mlua::UserData;

use crate::utils::{LoriToMainCommand, Vertex};

#[derive(Clone)]
pub struct LoriColliderRef {
    pub uid: u64,
    pub tx: Sender<LoriToMainCommand>,
}

impl UserData for LoriColliderRef {}

#[derive(Clone)]
pub struct LoriCollider {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub collision: String,
}

impl LoriCollider {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, collision: String) -> Self {
        Self {
            vertices,
            indices,
            collision,
        }
    }
}