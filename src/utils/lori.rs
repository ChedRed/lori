use crossbeam::channel::Sender;
use mlua::{Lua, Table};

use crate::utils::LfnCommand;

pub struct Lfn {
    pub lori: Table,
}

impl Lfn {
    pub fn new(lua: &Lua, tx: Sender<LfnCommand>) -> Self {
        let lori = lua.create_table().unwrap();
        let create = lua.create_table().unwrap();

        _= create.set("object", lua.create_function(move |_, ()| { // lori.create.object()
            tx.clone().send(LfnCommand::CreateObject { x: 0., y: 0., rotation: 0. }).map_err(mlua::Error::external).unwrap();
            Ok(())
        }).unwrap());
        
        _= lori.set("create", create);
        _= lua.globals().set("lori", lori.clone());

        Self {
            lori,
        }
    }
}

pub struct Lhk {
    pub lori: Table
}