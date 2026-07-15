use crossbeam::channel::Sender;
use mlua::{Lua, Table};

use crate::utils::LfnCommand;

pub struct Lfn {
    pub lori: Table,
}

impl Lfn {
    pub fn new(lua: &Lua, tx: Sender<LfnCommand>) -> Self {
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let lori = lua.create_table().unwrap();
        
        let set = lua.create_table().unwrap();
        let window = lua.create_table().unwrap();
        _= window.set("title", lua.create_function(move |_, text| { // lori.set.window.title
            _= tx2.send(LfnCommand::SetWindowTitle { text });
            Ok(())
        }).unwrap());

        _= window.set("size", lua.create_function(move |_, (w, h)| { // lori.set.window.size
            _= tx3.send(LfnCommand::SetWindowSize { w, h });
            Ok(())
        }).unwrap());
            
        let get = lua.create_table().unwrap();
        let new = lua.create_table().unwrap();
        let draw = lua.create_table().unwrap();
        let push = lua.create_table().unwrap();
        let delete = lua.create_table().unwrap();

        
        _= set.set("window", window);
        _= lori.set("set", set);
        _= lua.globals().set("lori", lori.clone());

        Self {
            lori,
        }
    }
}

pub struct Lhk {
    pub lori: Table
}


pub fn keycodes_transformer(code: winit::keyboard::KeyCode) -> &'static str {
    match code {
        winit::keyboard::KeyCode::KeyA => "a",
        winit::keyboard::KeyCode::KeyB => "b",
        winit::keyboard::KeyCode::KeyC => "c",
        winit::keyboard::KeyCode::KeyD => "d",
        winit::keyboard::KeyCode::KeyE => "e",
        winit::keyboard::KeyCode::KeyF => "f",
        winit::keyboard::KeyCode::KeyG => "g",
        winit::keyboard::KeyCode::KeyH => "h",
        winit::keyboard::KeyCode::KeyI => "i",
        winit::keyboard::KeyCode::KeyJ => "j",
        winit::keyboard::KeyCode::KeyK => "k",
        winit::keyboard::KeyCode::KeyL => "l",
        winit::keyboard::KeyCode::KeyM => "m",
        winit::keyboard::KeyCode::KeyN => "n",
        winit::keyboard::KeyCode::KeyO => "o",
        winit::keyboard::KeyCode::KeyP => "p",
        winit::keyboard::KeyCode::KeyQ => "q",
        winit::keyboard::KeyCode::KeyR => "r",
        winit::keyboard::KeyCode::KeyS => "s",
        winit::keyboard::KeyCode::KeyT => "t",
        winit::keyboard::KeyCode::KeyU => "u",
        winit::keyboard::KeyCode::KeyV => "v",
        winit::keyboard::KeyCode::KeyW => "w",
        winit::keyboard::KeyCode::KeyX => "x",
        winit::keyboard::KeyCode::KeyY => "y",
        winit::keyboard::KeyCode::KeyZ => "z",
        _ => "NONE",
    }
}