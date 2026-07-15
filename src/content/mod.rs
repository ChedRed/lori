use mlua::{Lua, Function, Table};
use crate::utils::lori::{Lfn, keycodes_transformer};

use crate::utils::{ContentCommand, Displacement, LfnCommand, Location, MainCommand};
use crossbeam::channel::{Receiver, Sender, unbounded};
use std::time::{Duration, Instant};
use rapier2d::prelude::*;
pub mod object;
use object::Object;

pub struct Control {
    pub binds: Vec<winit::keyboard::KeyCode>,
    pub state: f32,
    pub clamp: [f32; 2],
    pub index: usize,
}

impl Control {
    pub fn new(binds: Vec<winit::keyboard::KeyCode>, index: usize) -> Self {
        Self {
            binds,
            state: 0.,
            clamp: [0., 1.],
            index: index,
        }
    }
}

#[allow(dead_code)]
pub struct Content {
    lua: Lua,
    lfn: Lfn,
    lori_draw: Function,
    lori_update: Function,
    lori_keypressed: Function,
    lori_keyreleased: Function,
    
    displacement: Displacement,
    objects: Vec<Object>,
    
    gravity: Vec2, // TODO: allow control from Lua
    integration_parameters: IntegrationParameters,
    physics: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    rigidbodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,

    rx: Receiver<ContentCommand>,
    lrx: Receiver<LfnCommand>,
    tx: Sender<MainCommand>,
}

impl Content {
    pub fn create(lua_code: String, rx: Receiver<ContentCommand>, tx: Sender<MainCommand>) -> Self {
        let lua = Lua::new();
        let (ltx, lrx) = unbounded::<LfnCommand>();
        let lfn: Lfn = Lfn::new(&lua, ltx);

        _= lua.load(lua_code).exec().unwrap();

        let lhk: Table = lua.globals().get("lori").unwrap();

        let lori_load: Function = lhk.get("load").unwrap();
        let lori_draw: Function = lhk.get("draw").unwrap();
        let lori_update: Function = lhk.get("update").unwrap();
        let lori_keypressed: Function = lhk.get("keypressed").unwrap();
        let lori_keyreleased: Function = lhk.get("keyreleased").unwrap();

        
        _= lori_load.call::<()>(());
        while let Ok(cmd) = lrx.try_recv() {
            match cmd {
                LfnCommand::SetWindowTitle { text } => {
                    _= tx.send(MainCommand::SetWindowTitle { text });
                },
                LfnCommand::SetWindowSize { w, h } => {
                    _= tx.send(MainCommand::SetWindowSize { w, h });
                }
            }
        }

        
        let objects: Vec<Object> = Vec::new();
        
        let gravity = Vec2 { x: 0., y: 0. };
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1./50.;
        
        let physics = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = BroadPhaseBvh::new();
        let narrow_phase = NarrowPhase::new();
        let rigidbodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let impulse_joints = ImpulseJointSet::new();
        let multibody_joints = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();


        // Create objects here??
        
        let displacement: Displacement = Displacement::new();


        Self {
            lua,
            lfn,
            lori_draw,
            lori_update,
            lori_keypressed,
            lori_keyreleased,
            
            displacement,
            objects,

            gravity,
            integration_parameters,
            physics,
            island_manager,
            broad_phase,
            narrow_phase,
            rigidbodies,
            colliders,
            impulse_joints,
            multibody_joints,
            ccd_solver,

            rx,
            lrx,
            tx,
        }
    }

    pub fn thread_loop(&mut self) {
        let mut loopit: bool = true;
        let interval: Duration = Duration::from_millis(5);
        let mut start = Instant::now() + interval;
        
        while loopit {

            _= self.lori_update.call::<()>(());
            self.update_objects();
                
            while let Ok(cmd) = self.rx.try_recv() {
                match cmd {
                    ContentCommand::Render => {
                        _= self.lori_draw.call::<()>(());
                        
                        let mut new_double_locations: Vec<Vec<Location>> = Vec::new();
                        for i in 0..self.objects.len() {
                            let mut new_locations: Vec<Location> = Vec::new();
                            for j in 0..self.objects[i].rigidhandles.len() {
                                let mut new_location: Location = Location::new();
                                if let Some(body) = self.rigidbodies.get_mut(self.objects[i].rigidhandles[j]) {
                                    new_location.position = [body.translation()[0], body.translation()[1]];
                                    new_location.rotation = [body.rotation().angle(), 0.];
                                }
                                new_locations.push(new_location);
                            }
                            new_double_locations.push(new_locations);
                        }
                        
                        _= self.tx.send(MainCommand::Render { instances: new_double_locations, camera: self.displacement });
                    }

                    ContentCommand::Input { code, state } => {
                        if state {
                            _= self.lori_keypressed.call::<()>((keycodes_transformer(code), state));
                        } else {
                            _= self.lori_keyreleased.call::<()>((keycodes_transformer(code), state));
                        }
                    }
    
                    ContentCommand::Exit => {
                        loopit = false;
                    }
                }
            }

            while let Ok(cmd) = self.lrx.try_recv() {
                match cmd {
                    LfnCommand::SetWindowTitle { text } => {
                        _= self.tx.send(MainCommand::SetWindowTitle { text });
                    },
                    LfnCommand::SetWindowSize { w, h } => {
                        _= self.tx.send(MainCommand::SetWindowSize { w, h });
                    }
                }
            }

            let sleep = Instant::now();
            if sleep > start {
                println!("WARN: Took too long!");
            }
            else { 
                std::thread::sleep(start - sleep);
            }

            start += interval;
        };
    }
    
    fn update_objects(&mut self) {
        // Lua physics handle here
        
        self.physics.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}