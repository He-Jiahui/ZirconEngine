use super::World;
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent, ColliderComponent,
    JointComponent, PointLight, RigidBodyComponent, SpotLight,
};
use crate::scene::EntityId;

impl World {
    pub fn rigid_body(&self, entity: EntityId) -> Option<&RigidBodyComponent> {
        self.rigid_bodies.get(&entity)
    }

    pub fn collider(&self, entity: EntityId) -> Option<&ColliderComponent> {
        self.colliders.get(&entity)
    }

    pub fn joint(&self, entity: EntityId) -> Option<&JointComponent> {
        self.joints.get(&entity)
    }

    pub fn point_light(&self, entity: EntityId) -> Option<&PointLight> {
        self.point_lights.get(&entity)
    }

    pub fn spot_light(&self, entity: EntityId) -> Option<&SpotLight> {
        self.spot_lights.get(&entity)
    }

    pub fn animation_skeleton(&self, entity: EntityId) -> Option<&AnimationSkeletonComponent> {
        self.animation_skeletons.get(&entity)
    }

    pub fn animation_player(&self, entity: EntityId) -> Option<&AnimationPlayerComponent> {
        self.animation_players.get(&entity)
    }

    pub fn animation_sequence_player(
        &self,
        entity: EntityId,
    ) -> Option<&AnimationSequencePlayerComponent> {
        self.animation_sequence_players.get(&entity)
    }

    pub fn animation_graph_player(
        &self,
        entity: EntityId,
    ) -> Option<&AnimationGraphPlayerComponent> {
        self.animation_graph_players.get(&entity)
    }

    pub fn animation_state_machine_player(
        &self,
        entity: EntityId,
    ) -> Option<&AnimationStateMachinePlayerComponent> {
        self.animation_state_machine_players.get(&entity)
    }

    pub fn set_rigid_body(
        &mut self,
        entity: EntityId,
        rigid_body: Option<RigidBodyComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update rigid body for missing node {entity}"
            ));
        }
        let changed = match rigid_body {
            Some(rigid_body) => {
                if self.rigid_bodies.get(&entity) == Some(&rigid_body) {
                    false
                } else {
                    self.rigid_bodies.insert(entity, rigid_body);
                    true
                }
            }
            None => self.rigid_bodies.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_collider(
        &mut self,
        entity: EntityId,
        collider: Option<ColliderComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!("cannot update collider for missing node {entity}"));
        }
        let changed = match collider {
            Some(collider) => {
                if self.colliders.get(&entity) == Some(&collider) {
                    false
                } else {
                    self.colliders.insert(entity, collider);
                    true
                }
            }
            None => self.colliders.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_joint(
        &mut self,
        entity: EntityId,
        joint: Option<JointComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!("cannot update joint for missing node {entity}"));
        }
        if joint.as_ref().and_then(|joint| joint.connected_entity) == Some(entity) {
            return Err(format!("joint on node {entity} cannot connect to itself"));
        }
        let changed = match joint {
            Some(joint) => {
                if self.joints.get(&entity) == Some(&joint) {
                    false
                } else {
                    self.joints.insert(entity, joint);
                    true
                }
            }
            None => self.joints.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_point_light(
        &mut self,
        entity: EntityId,
        point_light: Option<PointLight>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update point light for missing node {entity}"
            ));
        }
        let changed = match point_light {
            Some(point_light) => {
                if self.point_lights.get(&entity) == Some(&point_light) {
                    false
                } else {
                    self.point_lights.insert(entity, point_light);
                    true
                }
            }
            None => self.point_lights.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_spot_light(
        &mut self,
        entity: EntityId,
        spot_light: Option<SpotLight>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update spot light for missing node {entity}"
            ));
        }
        let changed = match spot_light {
            Some(spot_light) => {
                if self.spot_lights.get(&entity) == Some(&spot_light) {
                    false
                } else {
                    self.spot_lights.insert(entity, spot_light);
                    true
                }
            }
            None => self.spot_lights.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_animation_skeleton(
        &mut self,
        entity: EntityId,
        animation_skeleton: Option<AnimationSkeletonComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update animation skeleton for missing node {entity}"
            ));
        }
        let changed = match animation_skeleton {
            Some(animation_skeleton) => {
                if self.animation_skeletons.get(&entity) == Some(&animation_skeleton) {
                    false
                } else {
                    self.animation_skeletons.insert(entity, animation_skeleton);
                    true
                }
            }
            None => self.animation_skeletons.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_animation_player(
        &mut self,
        entity: EntityId,
        animation_player: Option<AnimationPlayerComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update animation player for missing node {entity}"
            ));
        }
        let changed = match animation_player {
            Some(animation_player) => {
                if self.animation_players.get(&entity) == Some(&animation_player) {
                    false
                } else {
                    self.animation_players.insert(entity, animation_player);
                    true
                }
            }
            None => self.animation_players.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_animation_sequence_player(
        &mut self,
        entity: EntityId,
        animation_sequence_player: Option<AnimationSequencePlayerComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update animation sequence player for missing node {entity}"
            ));
        }
        let changed = match animation_sequence_player {
            Some(animation_sequence_player) => {
                if self.animation_sequence_players.get(&entity) == Some(&animation_sequence_player)
                {
                    false
                } else {
                    self.animation_sequence_players
                        .insert(entity, animation_sequence_player);
                    true
                }
            }
            None => self.animation_sequence_players.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_animation_graph_player(
        &mut self,
        entity: EntityId,
        animation_graph_player: Option<AnimationGraphPlayerComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update animation graph player for missing node {entity}"
            ));
        }
        let changed = match animation_graph_player {
            Some(animation_graph_player) => {
                if self.animation_graph_players.get(&entity) == Some(&animation_graph_player) {
                    false
                } else {
                    self.animation_graph_players
                        .insert(entity, animation_graph_player);
                    true
                }
            }
            None => self.animation_graph_players.remove(&entity).is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }

    pub fn set_animation_state_machine_player(
        &mut self,
        entity: EntityId,
        animation_state_machine_player: Option<AnimationStateMachinePlayerComponent>,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot update animation state machine player for missing node {entity}"
            ));
        }
        let changed = match animation_state_machine_player {
            Some(animation_state_machine_player) => {
                if self.animation_state_machine_players.get(&entity)
                    == Some(&animation_state_machine_player)
                {
                    false
                } else {
                    self.animation_state_machine_players
                        .insert(entity, animation_state_machine_player);
                    true
                }
            }
            None => self
                .animation_state_machine_players
                .remove(&entity)
                .is_some(),
        };
        if changed {
            self.refresh_node_cache();
        }
        Ok(changed)
    }
}
