use super::{SceneError, SceneResult, World};
use crate::scene::EntityId;
use crate::scene::components::{
    AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, ColliderComponent, JointComponent, PointLight, RectLight,
    RigidBodyComponent, SpotLight,
};

impl World {
    pub fn rigid_body(&self, entity: EntityId) -> Option<&RigidBodyComponent> {
        self.get::<RigidBodyComponent>(entity)
    }

    pub fn collider(&self, entity: EntityId) -> Option<&ColliderComponent> {
        self.get::<ColliderComponent>(entity)
    }

    pub fn joint(&self, entity: EntityId) -> Option<&JointComponent> {
        self.get::<JointComponent>(entity)
    }

    pub fn ambient_light(&self, entity: EntityId) -> Option<&AmbientLight> {
        self.get::<AmbientLight>(entity)
    }

    pub fn point_light(&self, entity: EntityId) -> Option<&PointLight> {
        self.get::<PointLight>(entity)
    }

    pub fn rect_light(&self, entity: EntityId) -> Option<&RectLight> {
        self.get::<RectLight>(entity)
    }

    pub fn spot_light(&self, entity: EntityId) -> Option<&SpotLight> {
        self.get::<SpotLight>(entity)
    }

    pub fn animation_skeleton(&self, entity: EntityId) -> Option<&AnimationSkeletonComponent> {
        self.get::<AnimationSkeletonComponent>(entity)
    }

    pub fn animation_player(&self, entity: EntityId) -> Option<&AnimationPlayerComponent> {
        self.get::<AnimationPlayerComponent>(entity)
    }

    pub fn animation_sequence_player(
        &self,
        entity: EntityId,
    ) -> Option<&AnimationSequencePlayerComponent> {
        self.get::<AnimationSequencePlayerComponent>(entity)
    }

    pub fn animation_graph_player(
        &self,
        entity: EntityId,
    ) -> Option<&AnimationGraphPlayerComponent> {
        self.get::<AnimationGraphPlayerComponent>(entity)
    }

    pub fn animation_state_machine_player(
        &self,
        entity: EntityId,
    ) -> Option<&AnimationStateMachinePlayerComponent> {
        self.get::<AnimationStateMachinePlayerComponent>(entity)
    }

    pub fn set_rigid_body(
        &mut self,
        entity: EntityId,
        rigid_body: Option<RigidBodyComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update rigid body for", entity));
        }
        let changed = match rigid_body {
            Some(rigid_body) => {
                if self.get::<RigidBodyComponent>(entity) == Some(&rigid_body) {
                    false
                } else {
                    self.insert(entity, rigid_body)?;
                    true
                }
            }
            None => self.remove::<RigidBodyComponent>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_collider(
        &mut self,
        entity: EntityId,
        collider: Option<ColliderComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update collider for", entity));
        }
        let changed = match collider {
            Some(collider) => {
                if self.get::<ColliderComponent>(entity) == Some(&collider) {
                    false
                } else {
                    self.insert(entity, collider)?;
                    true
                }
            }
            None => self.remove::<ColliderComponent>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_joint(
        &mut self,
        entity: EntityId,
        joint: Option<JointComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update joint for", entity));
        }
        let joint_connects_to_self = match &joint {
            Some(joint) => joint.connected_entity == Some(entity),
            None => false,
        };
        if joint_connects_to_self {
            return Err(SceneError::JointConnectsToSelf { entity });
        }
        let changed = match joint {
            Some(joint) => {
                if self.get::<JointComponent>(entity) == Some(&joint) {
                    false
                } else {
                    self.insert(entity, joint)?;
                    true
                }
            }
            None => self.remove::<JointComponent>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_point_light(
        &mut self,
        entity: EntityId,
        point_light: Option<PointLight>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update point light for", entity));
        }
        let changed = match point_light {
            Some(point_light) => {
                if self.get::<PointLight>(entity) == Some(&point_light) {
                    false
                } else {
                    self.insert(entity, point_light)?;
                    true
                }
            }
            None => self.remove::<PointLight>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_ambient_light(
        &mut self,
        entity: EntityId,
        ambient_light: Option<AmbientLight>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "update ambient light for",
                entity,
            ));
        }
        let changed = match ambient_light {
            Some(ambient_light) => {
                if self.get::<AmbientLight>(entity) == Some(&ambient_light) {
                    false
                } else {
                    self.insert(entity, ambient_light)?;
                    true
                }
            }
            None => self.remove::<AmbientLight>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_rect_light(
        &mut self,
        entity: EntityId,
        rect_light: Option<RectLight>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update rect light for", entity));
        }
        let changed = match rect_light {
            Some(rect_light) => {
                if self.get::<RectLight>(entity) == Some(&rect_light) {
                    false
                } else {
                    self.insert(entity, rect_light)?;
                    true
                }
            }
            None => self.remove::<RectLight>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_spot_light(
        &mut self,
        entity: EntityId,
        spot_light: Option<SpotLight>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update spot light for", entity));
        }
        let changed = match spot_light {
            Some(spot_light) => {
                if self.get::<SpotLight>(entity) == Some(&spot_light) {
                    false
                } else {
                    self.insert(entity, spot_light)?;
                    true
                }
            }
            None => self.remove::<SpotLight>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_animation_skeleton(
        &mut self,
        entity: EntityId,
        animation_skeleton: Option<AnimationSkeletonComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "update animation skeleton for",
                entity,
            ));
        }
        let changed = match animation_skeleton {
            Some(animation_skeleton) => {
                if self.get::<AnimationSkeletonComponent>(entity) == Some(&animation_skeleton) {
                    false
                } else {
                    self.insert(entity, animation_skeleton)?;
                    true
                }
            }
            None => self.remove::<AnimationSkeletonComponent>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_animation_player(
        &mut self,
        entity: EntityId,
        animation_player: Option<AnimationPlayerComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "update animation player for",
                entity,
            ));
        }
        let changed = match animation_player {
            Some(animation_player) => {
                if self.get::<AnimationPlayerComponent>(entity) == Some(&animation_player) {
                    false
                } else {
                    self.insert(entity, animation_player)?;
                    true
                }
            }
            None => self.remove::<AnimationPlayerComponent>(entity)?.is_some(),
        };
        Ok(changed)
    }

    pub fn set_animation_sequence_player(
        &mut self,
        entity: EntityId,
        animation_sequence_player: Option<AnimationSequencePlayerComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "update animation sequence player for",
                entity,
            ));
        }
        let changed = match animation_sequence_player {
            Some(animation_sequence_player) => {
                if self.get::<AnimationSequencePlayerComponent>(entity)
                    == Some(&animation_sequence_player)
                {
                    false
                } else {
                    self.insert(entity, animation_sequence_player)?;
                    true
                }
            }
            None => self
                .remove::<AnimationSequencePlayerComponent>(entity)?
                .is_some(),
        };
        Ok(changed)
    }

    pub fn set_animation_graph_player(
        &mut self,
        entity: EntityId,
        animation_graph_player: Option<AnimationGraphPlayerComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "update animation graph player for",
                entity,
            ));
        }
        let changed = match animation_graph_player {
            Some(animation_graph_player) => {
                if self.get::<AnimationGraphPlayerComponent>(entity)
                    == Some(&animation_graph_player)
                {
                    false
                } else {
                    self.insert(entity, animation_graph_player)?;
                    true
                }
            }
            None => self
                .remove::<AnimationGraphPlayerComponent>(entity)?
                .is_some(),
        };
        Ok(changed)
    }

    pub fn set_animation_state_machine_player(
        &mut self,
        entity: EntityId,
        animation_state_machine_player: Option<AnimationStateMachinePlayerComponent>,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "update animation state machine player for",
                entity,
            ));
        }
        let changed = match animation_state_machine_player {
            Some(animation_state_machine_player) => {
                if self.get::<AnimationStateMachinePlayerComponent>(entity)
                    == Some(&animation_state_machine_player)
                {
                    false
                } else {
                    self.insert(entity, animation_state_machine_player)?;
                    true
                }
            }
            None => self
                .remove::<AnimationStateMachinePlayerComponent>(entity)?
                .is_some(),
        };
        Ok(changed)
    }
}
