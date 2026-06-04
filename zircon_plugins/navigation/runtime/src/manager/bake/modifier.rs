use crate::component_json::parse_component;
use zircon_runtime::core::framework::navigation::{
    NavMeshModifierDescriptor, NAV_MESH_MODIFIER_COMPONENT_TYPE,
};
use zircon_runtime::scene::World;

pub(super) fn effective_modifier(
    world: &World,
    entity: u64,
    agent_type: &str,
) -> Option<NavMeshModifierDescriptor> {
    if let Some(modifier) = direct_modifier(world, entity, agent_type) {
        return Some(modifier);
    }

    let mut current = world.parent_of(entity);
    while let Some(parent) = current {
        if let Some(modifier) = direct_modifier(world, parent, agent_type) {
            if modifier.apply_to_children {
                return Some(modifier);
            }
        }
        current = world.parent_of(parent);
    }
    None
}

pub(super) fn direct_modifier(
    world: &World,
    entity: u64,
    agent_type: &str,
) -> Option<NavMeshModifierDescriptor> {
    let value = world.dynamic_component(entity, NAV_MESH_MODIFIER_COMPONENT_TYPE)?;
    let modifier = parse_component::<NavMeshModifierDescriptor>(value);
    modifier_affects_agent(&modifier, agent_type).then_some(modifier)
}

fn modifier_affects_agent(modifier: &NavMeshModifierDescriptor, agent_type: &str) -> bool {
    modifier.affected_agents.is_empty()
        || modifier
            .affected_agents
            .iter()
            .any(|affected| affected == agent_type)
}
