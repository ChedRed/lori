use crate::utils::{ContentCommand, ContentLrxCommand, ContentLtxCommand, Displacement, Location, MainCommand};
use crossbeam::channel::{Receiver, Sender};
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


pub struct Content {
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

    mrx: Receiver<ContentCommand>,
    mtx: Sender<MainCommand>,
    crx: Receiver<ContentLtxCommand>,
    ctx: Sender<ContentLrxCommand>,
}

impl Content {
    pub fn create(mtx: Sender<MainCommand>, mrx: Receiver<ContentCommand>, ctx: Sender<ContentLrxCommand>, crx: Receiver<ContentLtxCommand>) -> Self {
        while let Ok(cmd) = crx.try_recv() {
            match cmd {
                // _ => {}
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

            mrx,
            crx,
            mtx,
            ctx,
        }
    }

    pub fn thread_loop(&mut self) {
        let mut loopit: bool = true;
        let interval: Duration = Duration::from_millis(5);
        let mut start = Instant::now() + interval;
        
        while loopit {

            _= self.ctx.send(ContentLrxCommand::Update);
            self.update_objects();
                
            while let Ok(cmd) = self.mrx.try_recv() {
                match cmd {
                    ContentCommand::Render => {
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
                        
                        _= self.mtx.send(MainCommand::Render { instances: new_double_locations, camera: self.displacement });
                    }
    
                    ContentCommand::Exit => {
                        loopit = false;
                    },
                }
            }

            while let Ok(cmd) = self.crx.try_recv() {
                match cmd {
                    // _ => {}
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