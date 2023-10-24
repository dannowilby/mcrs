#![allow(dead_code)]
#![allow(unused_variables)]

use rapier3d::prelude::*;
use std::collections::HashMap;

use crate::{
    engine::{input::Input, renderer::Renderer},
    world::{Event, GameData},
};

pub struct PhysicsEngine {
    rigidbody_set: RigidBodySet,
    collider_set: ColliderSet,

    colliders_handles: HashMap<String, ColliderHandle>,
    rigidbody_handles: HashMap<String, RigidBodyHandle>,

    gravity: Vector<Real>,

    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,

    pipeline: PhysicsPipeline,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            rigidbody_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),

            rigidbody_handles: HashMap::new(),
            colliders_handles: HashMap::new(),

            gravity: vector![0.0, -9.81, 0.0],

            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),

            pipeline: PhysicsPipeline::new(),
        }
    }
    
    pub fn insert_entity(&mut self, id: &str, rigidbody: RigidBody, collider: Collider) {
        let rigid_body_handle = self.rigidbody_set.insert(rigidbody);
        self.rigidbody_handles.insert(String::from(id), rigid_body_handle);
        let collider_handle = self.collider_set.insert_with_parent(collider, rigid_body_handle, &mut self.rigidbody_set);
        self.colliders_handles.insert(String::from(id), collider_handle);
    }

    pub fn insert_collider(&mut self, id: String, collider: Collider) {
        let handle = self.collider_set.insert(collider);
        self.colliders_handles.insert(id, handle);
    }

    pub fn get_collider(&self, id: String) -> Option<&Collider> {
        let handle = self.colliders_handles.get(&id)?;
        self.collider_set.get(handle.clone())
    }

    pub fn remove_collider(&mut self, id: &str) {
        if let Some(handle) = self.colliders_handles.get(id) {
            self.collider_set.remove(
                handle.clone(),
                &mut self.island_manager,
                &mut self.rigidbody_set,
                false,
            );
        }
    }

    pub fn insert_rigid_body(&mut self, id: String, rigidbody: RigidBody) {
        let handle = self.rigidbody_set.insert(rigidbody);
        self.rigidbody_handles.insert(id, handle);
    }

    pub fn get_rigid_body(&self, id: String) -> Option<&RigidBody> {
        let handle = self.rigidbody_handles.get(&id)?;
        self.rigidbody_set.get(handle.clone())
    }

    pub fn get_mut_rigid_body<'a>(&'a mut self, id: String) -> Option<&'a mut RigidBody> {
        let handle = self.rigidbody_handles.get(&id)?;
        self.rigidbody_set.get_mut(handle.clone())
    }
    
    pub fn remove_rigid_body(&mut self, id: &str) {
        if let Some(handle) = self.rigidbody_handles.get(id) {
            self.rigidbody_set.remove(
                handle.clone(),
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }

    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbody_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
    }
}

pub fn simulate_physics(
    _renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    delta: f64,
) {
    data.physics_engine.step();
}
