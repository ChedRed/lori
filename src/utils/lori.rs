use std::process::exit;

use crossbeam::channel::{Receiver, Sender};
use mlua::Function;

use crate::utils::{LoriToMainCall, LoriToMainCommand, MainToLoriCall, MainToLoriCommand};

pub struct Lori {
    lua: mlua::Lua,
    main_call: Receiver<MainToLoriCall>,
    main_back: Sender<LoriToMainCall>,

    lori_load: Option<mlua::Function>,
    lori_keypressed: Option<mlua::Function>,
    lori_keyreleased: Option<mlua::Function>,
    lori_mousepressed: Option<mlua::Function>,
    lori_mousereleased: Option<mlua::Function>,
    lori_mousemoved: Option<mlua::Function>,
    lori_mousescrolled: Option<mlua::Function>,
    lori_update: Option<mlua::Function>,
    lori_render: Option<mlua::Function>,
}

impl Lori {
    pub fn new(code: String, verbose: bool, main_cmd: Sender<LoriToMainCommand>, main_rtrn: Receiver<MainToLoriCommand>, main_call: Receiver<MainToLoriCall>, main_back: Sender<LoriToMainCall>) -> Self {
        let lua = mlua::Lua::new();

        let tx = main_cmd.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        let tx5 = tx.clone();
        let tx6 = tx.clone();
        let tx7 = tx.clone();
        let tx8 = tx.clone();
        let tx9 = tx.clone();
        
        let rx = main_rtrn.clone();
        let rx2 = rx.clone();
        let rx3 = rx.clone();
        let lori = lua.create_table().unwrap();
        
        let set = lua.create_table().unwrap();
        let set_window = lua.create_table().unwrap();
        _= set_window.set("title", lua.create_function(move |_, text| { // lori.set.window.title
            _= tx2.send(LoriToMainCommand::SetWindowTitle { text });
            Ok(())
        }).unwrap());

        _= set_window.set("size", lua.create_function(move |_, (w, h)| { // lori.set.window.size
            _= tx3.send(LoriToMainCommand::SetWindowSize { w, h });
            Ok(())
        }).unwrap());

        _= set_window.set("resizable", lua.create_function(move |_, is| { // lori.set.window.size
            _= tx4.send(LoriToMainCommand::SetWindowResizable { is });
            Ok(())
        }).unwrap());

            
        let get = lua.create_table().unwrap();
        let get_window = lua.create_table().unwrap();
        _= get_window.set("size", lua.create_function(move |_, ()| {
            let mut nw: u32 = 0;
            let mut nh: u32 = 0;
            _= tx5.try_send(LoriToMainCommand::GetWindowSize);
            while let Ok(cmd) = rx2.recv() {
                match cmd {
                    MainToLoriCommand::ReturnWindowSize { w, h } => {
                        nw = w;
                        nh = h;
                        break;
                    }
                    _ => {}
                }
            }
            Ok((nw, nh))
        }).unwrap());
            
        let get_key = lua.create_table().unwrap();
        _= get_key.set("state", lua.create_function(move |_, key| {
            let mut pressed: bool = false;
            _= tx8.try_send(LoriToMainCommand::GetKeyPressed { key });
            while let Ok(cmd) = rx3.recv() {
                match cmd {
                    MainToLoriCommand::ReturnKeyPressed { key } => {
                        pressed = key;
                        break;
                    }
                    _ => {}
                }
            }
            Ok(pressed)
        }).unwrap());
        
        let new = lua.create_table().unwrap();
        let draw = lua.create_table().unwrap();
        _= draw.set("rect", lua.create_function(move |_, (x, y, w, h, r, color)| {
            _= tx6.send(LoriToMainCommand::DrawPrimitive { x, y, w, h, r, color, label: 0 });
            Ok(())
        }).unwrap());
        _= draw.set("circle", lua.create_function(move |_, (x, y, r, color)| {
            _= tx7.send(LoriToMainCommand::DrawPrimitive { x, y, w: 0., h: 0., r, color, label: 1 });
            Ok(())
        }).unwrap());
        _= draw.set("line", lua.create_function(move |_, (x1, y1, x2, y2, r, color)| {
            _= tx9.send(LoriToMainCommand::DrawPrimitive { x: x1, y: y1, w: x2, h: y2, r, color, label: 2 });
            Ok(())
        }).unwrap());
        let push = lua.create_table().unwrap();
        let delete = lua.create_table().unwrap();

        
        _= set.set("window", set_window);
        _= get.set("key", get_key);
        _= get.set("window", get_window);
        _= lori.set("set", set);
        _= lori.set("get", get);
        _= lori.set("draw", draw);
        _= lua.globals().set("lori", lori.clone());
        match lua.load(code).exec() {
            Ok(()) => {
                if verbose {
                    println!("lori (VBOS): Successfully loaded code");
                }
            }
            Err(e)=> {
                eprintln!("lori (EROR): {}", e);
                exit(3);
            }
        }
        let lhk: mlua::Table = lua.globals().get("lori").unwrap();

        let mut lori_load: Option<mlua::Function> = None;
        let mut lori_keypressed: Option<mlua::Function> = None; 
        let mut lori_keyreleased: Option<mlua::Function> = None; 
        let mut lori_mousepressed: Option<mlua::Function> = None; 
        let mut lori_mousereleased: Option<mlua::Function> = None; 
        let mut lori_mousemoved: Option<mlua::Function> = None; 
        let mut lori_mousescrolled: Option<mlua::Function> = None; 
        let mut lori_update: Option<mlua::Function> = None; 
        let mut lori_render: Option<mlua::Function> = None; 
        
        match lhk.get("load") {
            Ok(func) => {
                lori_load = func;
                if verbose {
                    println!("lori (VBOS): Loaded function 'Load'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("keypressed") {
            Ok(func) => {
                lori_keypressed = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'KeyPressed'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("keyreleased") {
            Ok(func) => {
                lori_keyreleased = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'KeyReleased'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("mousepressed") {
            Ok(func) => {
                lori_mousepressed = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'MousePressed'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("mousereleased") {
            Ok(func) => {
                lori_mousereleased = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'MouseReleased'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("mousemoved") {
            Ok(func) => {
                lori_mousemoved = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'MouseMoved'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("mousescrolled") {
            Ok(func) => {
                lori_mousescrolled = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'MouseScrolled'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("update") {
            Ok(func) => {
                lori_update = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'Update'");
                }
            }
            _ => {}
        }
        match lhk.get::<Function>("render") {
            Ok(func) => {
                lori_render = Some(func);
                if verbose {
                    println!("lori (VBOS): Loaded function 'Render'");
                }
            }
            _ => {}
        }

        
        Self {
            lua,
            main_call,
            main_back,

            lori_load,
            lori_keypressed,
            lori_keyreleased,
            lori_mousepressed,
            lori_mousereleased,
            lori_mousemoved,
            lori_mousescrolled,
            lori_update,
            lori_render,
        }
    }

    pub fn begin(&mut self) {
        while let Ok(cmd) = self.main_call.recv() {
            match cmd {
                MainToLoriCall::Load => {
                    match &self.lori_load {
                        Some(func) => {
                            _= func.call::<()>(());
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Load);
                }
                MainToLoriCall::Keypressed { code } => {
                    match &self.lori_keypressed {
                        Some(func) => {
                            _= func.call::<()>(code);
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Keypressed);
                }
                MainToLoriCall::Keyreleased { code } => {
                    match &self.lori_keyreleased {
                        Some(func) => {
                            _= func.call::<()>(code);
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Keyreleased);
                }
                MainToLoriCall::Mousepressed { x, y, button } => {
                    match &self.lori_mousepressed {
                        Some(func) => {
                            _= func.call::<()>((x, y, button));
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Mousepressed);
                }
                MainToLoriCall::Mousereleased { x, y, button } => {
                    match &self.lori_mousereleased {
                        Some(func) => {
                            _= func.call::<()>((x, y, button));
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Mousereleased);
                }
                MainToLoriCall::MouseMoved { motion } => {
                    match &self.lori_mousemoved {
                        Some(func) => {
                            _= func.call::<()>((motion.0, motion.1));
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::MouseMoved);
                }
                MainToLoriCall::MouseScrolled { motion } => {
                    match &self.lori_mousescrolled {
                        Some(func) => {
                            _= func.call::<()>((motion.0, motion.1));
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::MouseScrolled);
                }
                MainToLoriCall::Update { delta } => {
                    match &self.lori_update {
                        Some(func) => {
                            _= func.call::<()>(delta);
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Draw);
                }
                MainToLoriCall::Render => {
                    match &self.lori_render {
                        Some(func) => {
                            _= func.call::<()>(());
                        }
                        _ => {}
                    }
                    _= self.main_back.send(LoriToMainCall::Render);
                }
                MainToLoriCall::Exit => {
                    break;
                }
            }
        }
    }
}