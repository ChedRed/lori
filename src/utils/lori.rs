use crossbeam::channel::{Receiver, Sender};

use crate::utils::{LoriToMainCall, LoriToMainCommand, MainToLoriCall, MainToLoriCommand};

pub struct Lori {
    lua: mlua::Lua,
    main_call: Receiver<MainToLoriCall>,
    main_back: Sender<LoriToMainCall>,

    lori_load: mlua::Function,
    lori_keypressed: mlua::Function,
    lori_keyreleased: mlua::Function,
    lori_mousepressed: mlua::Function,
    lori_mousereleased: mlua::Function,
    lori_mousemoved: mlua::Function,
    lori_mousescrolled: mlua::Function,
    lori_update: mlua::Function,
    lori_render: mlua::Function,
}

impl Lori {
    pub fn new(code: String, main_cmd: Sender<LoriToMainCommand>, main_rtrn: Receiver<MainToLoriCommand>, main_call: Receiver<MainToLoriCall>, main_back: Sender<LoriToMainCall>) -> Self {
        let lua = mlua::Lua::new();

        let tx = main_cmd.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        let tx5 = tx.clone();
        let tx6 = tx.clone();
        let tx7 = tx.clone();
        let tx8 = tx.clone();
        
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
        let push = lua.create_table().unwrap();
        let delete = lua.create_table().unwrap();

        
        _= set.set("window", set_window);
        _= get.set("key", get_key);
        _= get.set("window", get_window);
        _= lori.set("set", set);
        _= lori.set("get", get);
        _= lori.set("draw", draw);
        _= lua.globals().set("lori", lori.clone());
        lua.load(code).exec().unwrap();
        let lhk: mlua::Table = lua.globals().get("lori").unwrap();

        let lori_load: mlua::Function = lhk.get("load").unwrap();
        let lori_keypressed: mlua::Function = lhk.get("keypressed").unwrap();
        let lori_keyreleased: mlua::Function = lhk.get("keyreleased").unwrap();
        let lori_mousepressed: mlua::Function = lhk.get("mousepressed").unwrap();
        let lori_mousereleased: mlua::Function = lhk.get("mousereleased").unwrap();
        let lori_mousemoved: mlua::Function = lhk.get("mousemoved").unwrap();
        let lori_mousescrolled: mlua::Function = lhk.get("mousescrolled").unwrap();
        let lori_update: mlua::Function = lhk.get("update").unwrap();
        let lori_render: mlua::Function = lhk.get("render").unwrap();
        
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
                    _= self.lori_load.call::<()>(());
                    _= self.main_back.send(LoriToMainCall::Load);
                }
                MainToLoriCall::Keypressed { code } => {
                    _= self.lori_keypressed.call::<()>(code);
                    _= self.main_back.send(LoriToMainCall::Keypressed);
                }
                MainToLoriCall::Keyreleased { code } => {
                    _= self.lori_keyreleased.call::<()>(code);
                    _= self.main_back.send(LoriToMainCall::Keyreleased);
                }
                MainToLoriCall::Mousepressed { x, y, button } => {
                    _= self.lori_mousepressed.call::<()>((x, y, button));
                    _= self.main_back.send(LoriToMainCall::Mousepressed);
                }
                MainToLoriCall::Mousereleased { x, y, button } => {
                    _= self.lori_mousereleased.call::<()>((x, y, button));
                    _= self.main_back.send(LoriToMainCall::Mousereleased);
                }
                MainToLoriCall::MouseMoved { motion } => {
                    _= self.lori_mousemoved.call::<()>((motion.0, motion.1));
                    _= self.main_back.send(LoriToMainCall::MouseMoved);
                }
                MainToLoriCall::MouseScrolled { motion } => {
                    _= self.lori_mousescrolled.call::<()>((motion.0, motion.1));
                    _= self.main_back.send(LoriToMainCall::MouseScrolled);
                }
                MainToLoriCall::Update { delta } => {
                    _= self.lori_update.call::<()>(delta);
                    _= self.main_back.send(LoriToMainCall::Draw);
                }
                MainToLoriCall::Render => {
                    _= self.lori_render.call::<()>(());
                    _= self.main_back.send(LoriToMainCall::Render);
                }
                MainToLoriCall::Exit => {
                    break;
                }
            }
        }
    }
}