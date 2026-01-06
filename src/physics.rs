use crate::shared::*;

pub type PhysicsDiff = (Vec<(RigidBodyHandle, RigidBody)>, Vec<(ColliderHandle, Collider)>);

pub struct Physics {
    pub state: PhysicsState,
    pipeline: PhysicsPipeline,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            state: PhysicsState::new(),
            pipeline: PhysicsPipeline::new(),
        }
    }

    pub fn step(&mut self) {
        self.pipeline.step(
            &self.state.gravity,
            &self.state.integration_parameters,
            &mut self.state.island_manager,
            &mut self.state.broad_phase,
            &mut self.state.narrow_phase,
            &mut self.state.rigid_body_set,
            &mut self.state.collider_set,
            &mut self.state.impulse_joint_set,
            &mut self.state.multibody_joint_set,
            &mut self.state.ccd_solver,
            &(),
            &(),
        );
    }

    pub fn spawn_cube(&mut self, pos: Vec3, size: Vec3) -> (RigidBodyHandle, ColliderHandle) {
        let size = size / 2.0; // half extents

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(conv_vec_1(pos))
            .build();
        let collider = ColliderBuilder::cuboid(size.x, size.y, size.z).restitution(0.7).build();

        let rigid_body_handle = self.state.rigid_body_set.insert(rigid_body);
        let collider_handle = self.state.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut self.state.rigid_body_set
        );

        (rigid_body_handle, collider_handle)
    }

    pub fn get_physics_diff(&self) -> PhysicsDiff {
        // use rapier3d::data::HasModifiedFlag;

        // TODO create proper diff
        // TODO move to server?

        (
            self.state.rigid_body_set.iter()
                // .filter(|(_, rig)| rig.has_modified_flag())
                .map(|(handler, rig)| (handler, rig.clone()))
                .collect::<Vec<_>>(),
            self.state.collider_set.iter()
                // .filter(|(_, col)| col.has_modified_flag())
                .map(|(handler, col)| (handler, col.clone()))
                .collect::<Vec<_>>()
        )
    }

    pub fn get_rig(&self, handle: RigidBodyHandle) -> &RigidBody {
        self.state.rigid_body_set.get(handle).expect("invalid rigid body handle")
    }

    pub fn get_rig_mut(&mut self, handle: RigidBodyHandle) -> &mut RigidBody {
        self.state.rigid_body_set.get_mut(handle).expect("invalid rigid body handle")
    }

    pub fn get_col(&self, handle: ColliderHandle) -> &Collider {
        self.state.collider_set.get(handle).expect("invalid collider handle")
    }

    pub fn get_col_mut(&mut self, handle: ColliderHandle) -> &mut Collider {
        self.state.collider_set.get_mut(handle).expect("invalid collider handle")
    }
}

#[derive(Serialize, Deserialize)]
pub struct PhysicsState {
    valid: bool, // is the state initialized or null?
    gravity: Vector<f32>,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            valid: false,
            gravity: Vector::new(0.0, -9.81, 0.0),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn set_valid(&mut self) {
        self.valid = true;
    }
}
