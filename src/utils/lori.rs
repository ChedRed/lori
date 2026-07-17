use crossbeam::{channel::{Receiver, Sender}, select};

use crate::utils::{MainLtxCommand, MainLrxCommand, ContentLtxCommand, ContentLrxCommand};

pub struct Lori {
    lua: mlua::Lua,
    main_rx: Receiver<MainLrxCommand>,
    content_rx: Receiver<ContentLrxCommand>,

    lori_load: mlua::Function,
    lori_render: mlua::Function,
    lori_update: mlua::Function,
    lori_keypressed: mlua::Function,
    lori_keyreleased: mlua::Function,
}

impl Lori {
    pub fn new(code: String, main_tx: Sender<MainLtxCommand>, main_rx: Receiver<MainLrxCommand>, content_tx: Sender<ContentLtxCommand>, content_rx: Receiver<ContentLrxCommand>) -> Self {
        let lua = mlua::Lua::new();

        let tx = main_tx.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        let tx5 = tx.clone();
        let tx6 = tx.clone();
        
        let rx = main_rx.clone();
        let rx2 = rx.clone();
        let lori = lua.create_table().unwrap();
        
        let set = lua.create_table().unwrap();
        let set_window = lua.create_table().unwrap();
        _= set_window.set("title", lua.create_function(move |_, text| { // lori.set.window.title
            _= tx2.send(MainLtxCommand::SetWindowTitle { text });
            Ok(())
        }).unwrap());

        _= set_window.set("size", lua.create_function(move |_, (w, h)| { // lori.set.window.size
            _= tx3.send(MainLtxCommand::SetWindowSize { w, h });
            Ok(())
        }).unwrap());

        _= set_window.set("resizable", lua.create_function(move |_, is| { // lori.set.window.size
            _= tx4.send(MainLtxCommand::SetWindowResizable { is });
            Ok(())
        }).unwrap());

            
        let get = lua.create_table().unwrap();
        let get_window = lua.create_table().unwrap();
        _= get_window.set("size", lua.create_function(move |_, ()| {
            let mut nw: u32 = 0;
            let mut nh: u32 = 0;
            _= tx5.try_send(MainLtxCommand::GetWindowSize);
            while let Ok(cmd) = rx2.recv() {
                match cmd {
                    MainLrxCommand::GetWindowSize { w, h } => {
                        nw = w;
                        nh = h;
                        break;
                    }
                    _ => {} // TODO: Not that
                }
            }
            Ok((nw, nh))
        }).unwrap());
        
        let new = lua.create_table().unwrap();
        let draw = lua.create_table().unwrap();
        _= draw.set("rect", lua.create_function(move |_, (x, y, w, h, r, color)| {
            _= tx6.send(MainLtxCommand::DrawRect { x, y, w, h, r, color });
            Ok(())
        }).unwrap());
        let push = lua.create_table().unwrap();
        let delete = lua.create_table().unwrap();

        
        _= set.set("window", set_window);
        _= get.set("window", get_window);
        _= lori.set("set", set);
        _= lori.set("get", get);
        _= lori.set("draw", draw);
        _= lua.globals().set("lori", lori.clone());
        lua.load(code).exec().unwrap();
        let lhk: mlua::Table = lua.globals().get("lori").unwrap();

        let lori_load: mlua::Function = lhk.get("load").unwrap();
        let lori_render: mlua::Function = lhk.get("render").unwrap();
        let lori_update: mlua::Function = lhk.get("update").unwrap();
        let lori_keypressed: mlua::Function = lhk.get("keypressed").unwrap();
        let lori_keyreleased: mlua::Function = lhk.get("keyreleased").unwrap();
        
        Self {
            lua,
            // main_tx,
            main_rx,
            // content_tx,
            content_rx,

            lori_load,
            lori_render,
            lori_update,
            lori_keypressed,
            lori_keyreleased,
        }
    }

    pub fn begin(&mut self) {
        _= self.lori_load.call::<()>(());

        loop {
            select! {
                recv(self.main_rx) -> cmd => {
                    if let Ok(v) = cmd {
                        match v {
                            MainLrxCommand::Keypressed { code } => {
                                _= self.lori_keypressed.call::<()>(code);
                            },
                            MainLrxCommand::Keyreleased { code } => {
                                _= self.lori_keyreleased.call::<()>(code);
                            },
                            MainLrxCommand::Render => {
                                _= self.lori_render.call::<()>(());
                            },
                            MainLrxCommand::Exit => {
                                break;
                            },
                            _ => {}
                        }
                    }
                },
                recv(self.content_rx) -> cmd => {
                    if let Ok(v) = cmd {
                        match v {
                            ContentLrxCommand::Update => {
                                _= self.lori_update.call::<()>(());
                            }
                        }
                    }
                }
            }
        }
    }
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