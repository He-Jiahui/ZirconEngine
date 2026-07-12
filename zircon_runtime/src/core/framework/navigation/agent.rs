use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::constants::{NavAreaMask, DEFAULT_AREA_MASK};
use super::handle::NavMeshHandle;
use super::settings::NavigationAgentSettings;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavAvoidanceQuality {
    None,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavAgentWritebackMode {
    #[default]
    Transform,
    DesiredVelocity,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavDesiredVelocity {
    pub linear: [Real; 3],
}

impl Default for NavAvoidanceQuality {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NavMeshAgentDescriptor {
    pub nav_mesh: Option<NavMeshHandle>,
    pub agent_type: String,
    pub radius: Real,
    pub height: Real,
    pub base_offset: Real,
    pub speed: Real,
    pub angular_speed: Real,
    pub acceleration: Real,
    pub stopping_distance: Real,
    pub auto_braking: bool,
    pub avoidance_quality: NavAvoidanceQuality,
    pub priority: u8,
    pub area_mask: NavAreaMask,
    pub auto_repath: bool,
    pub auto_traverse_links: bool,
    pub update_position: bool,
    pub update_rotation: bool,
    pub writeback_mode: NavAgentWritebackMode,
    pub destination: Option<[Real; 3]>,
}

impl Default for NavMeshAgentDescriptor {
    fn default() -> Self {
        let agent = NavigationAgentSettings::humanoid();
        Self {
            nav_mesh: None,
            agent_type: agent.id,
            radius: agent.radius,
            height: agent.height,
            base_offset: 0.0,
            speed: agent.speed,
            angular_speed: agent.angular_speed_degrees,
            acceleration: agent.acceleration,
            stopping_distance: agent.stopping_distance,
            auto_braking: true,
            avoidance_quality: NavAvoidanceQuality::Medium,
            priority: 50,
            area_mask: DEFAULT_AREA_MASK,
            auto_repath: true,
            auto_traverse_links: true,
            update_position: true,
            update_rotation: true,
            writeback_mode: NavAgentWritebackMode::Transform,
            destination: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavAgentTickReport {
    pub scanned_agents: usize,
    pub moved_agents: usize,
    pub blocked_agents: usize,
    pub traversing_agents: usize,
    pub queued_link_agents: usize,
    pub off_mesh_events: Vec<super::off_mesh_link::OffMeshTraverseEvent>,
    pub diagnostics: Vec<String>,
}
