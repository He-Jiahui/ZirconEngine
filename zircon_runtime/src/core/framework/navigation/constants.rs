pub const NAV_MESH_SURFACE_COMPONENT_TYPE: &str = "navigation.Component.NavMeshSurface";
pub const NAV_MESH_MODIFIER_COMPONENT_TYPE: &str = "navigation.Component.NavMeshModifier";
pub const NAV_MESH_AGENT_COMPONENT_TYPE: &str = "navigation.Component.NavMeshAgent";
pub const NAV_DESIRED_VELOCITY_COMPONENT_TYPE: &str = "navigation.Component.NavDesiredVelocity";
pub const NAV_MESH_OBSTACLE_COMPONENT_TYPE: &str = "navigation.Component.NavMeshObstacle";
pub const NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE: &str = "navigation.Component.NavMeshOffMeshLink";
pub const NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE: &str =
    "navigation.Component.NavMeshOffMeshBridge";

pub const DEFAULT_AGENT_TYPE: &str = "humanoid";
pub const AREA_NOT_WALKABLE: NavAreaId = 0;
pub const AREA_WALKABLE: NavAreaId = 1;
pub const AREA_JUMP: NavAreaId = 2;
pub const CUSTOM_AREA_START: NavAreaId = 3;
pub const MAX_NAV_AREAS: usize = 64;
pub const DEFAULT_AREA_MASK: NavAreaMask = u64::MAX;
pub const MAX_OFF_MESH_BRIDGE_LANES: u32 = 32;

pub type NavAreaId = u8;
pub type NavAreaMask = u64;
