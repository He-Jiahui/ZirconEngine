use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::constants::{
    NavAreaId, AREA_JUMP, AREA_NOT_WALKABLE, AREA_WALKABLE, DEFAULT_AGENT_TYPE,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationAgentSettings {
    pub id: String,
    pub display_name: String,
    pub radius: Real,
    pub height: Real,
    pub max_climb: Real,
    pub max_slope_degrees: Real,
    pub speed: Real,
    pub acceleration: Real,
    pub angular_speed_degrees: Real,
    pub stopping_distance: Real,
}

impl NavigationAgentSettings {
    pub fn humanoid() -> Self {
        Self {
            id: DEFAULT_AGENT_TYPE.to_string(),
            display_name: "Humanoid".to_string(),
            radius: 0.5,
            height: 2.0,
            max_climb: 0.4,
            max_slope_degrees: 45.0,
            speed: 3.5,
            acceleration: 8.0,
            angular_speed_degrees: 360.0,
            stopping_distance: 0.1,
        }
    }
}

impl Default for NavigationAgentSettings {
    fn default() -> Self {
        Self::humanoid()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationAreaSettings {
    pub id: NavAreaId,
    pub name: String,
    pub cost: Real,
    pub walkable: bool,
}

impl NavigationAreaSettings {
    pub fn not_walkable() -> Self {
        Self {
            id: AREA_NOT_WALKABLE,
            name: "not_walkable".to_string(),
            cost: 0.0,
            walkable: false,
        }
    }

    pub fn walkable() -> Self {
        Self {
            id: AREA_WALKABLE,
            name: "walkable".to_string(),
            cost: 1.0,
            walkable: true,
        }
    }

    pub fn jump() -> Self {
        Self {
            id: AREA_JUMP,
            name: "jump".to_string(),
            cost: 2.0,
            walkable: true,
        }
    }
}

pub fn default_navigation_areas() -> Vec<NavigationAreaSettings> {
    vec![
        NavigationAreaSettings::not_walkable(),
        NavigationAreaSettings::walkable(),
        NavigationAreaSettings::jump(),
    ]
}
