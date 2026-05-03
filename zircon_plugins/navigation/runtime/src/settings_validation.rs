use std::collections::HashSet;

use zircon_runtime::asset::NavigationSettingsAsset;
use zircon_runtime::core::framework::navigation::{
    NavigationError, NavigationErrorKind, MAX_NAV_AREAS,
};
use zircon_runtime::core::math::Real;

pub(crate) fn validate_navigation_settings(
    settings: &NavigationSettingsAsset,
) -> Result<(), NavigationError> {
    if settings.agents.is_empty() {
        return Err(invalid_settings(
            "navigation settings must define at least one agent",
        ));
    }
    let mut agent_ids = HashSet::new();
    for agent in &settings.agents {
        if agent.id.trim().is_empty() {
            return Err(invalid_settings("navigation agent id must not be empty"));
        }
        if !agent_ids.insert(agent.id.as_str()) {
            return Err(invalid_settings(format!(
                "navigation settings define duplicate agent id `{}`",
                agent.id
            )));
        }
        validate_positive_real("agent radius", agent.radius)?;
        validate_positive_real("agent height", agent.height)?;
        validate_non_negative_real("agent max climb", agent.max_climb)?;
        validate_non_negative_real("agent max slope", agent.max_slope_degrees)?;
        validate_non_negative_real("agent speed", agent.speed)?;
        validate_non_negative_real("agent acceleration", agent.acceleration)?;
        validate_non_negative_real("agent angular speed", agent.angular_speed_degrees)?;
        validate_non_negative_real("agent stopping distance", agent.stopping_distance)?;
    }

    let mut area_ids = HashSet::new();
    for area in &settings.areas {
        if usize::from(area.id) >= MAX_NAV_AREAS {
            return Err(invalid_settings(format!(
                "navigation area id {} is outside the 0..{} mask range",
                area.id, MAX_NAV_AREAS
            )));
        }
        if !area_ids.insert(area.id) {
            return Err(invalid_settings(format!(
                "navigation settings define duplicate area id {}",
                area.id
            )));
        }
        if area.name.trim().is_empty() {
            return Err(invalid_settings(format!(
                "navigation area {} name must not be empty",
                area.id
            )));
        }
        validate_non_negative_real("area cost", area.cost)?;
    }
    Ok(())
}

fn validate_positive_real(label: &str, value: Real) -> Result<(), NavigationError> {
    if value > 0.0 && value.is_finite() {
        return Ok(());
    }
    Err(invalid_settings(format!(
        "navigation {label} must be positive and finite"
    )))
}

fn validate_non_negative_real(label: &str, value: Real) -> Result<(), NavigationError> {
    if value >= 0.0 && value.is_finite() {
        return Ok(());
    }
    Err(invalid_settings(format!(
        "navigation {label} must be non-negative and finite"
    )))
}

fn invalid_settings(message: impl Into<String>) -> NavigationError {
    NavigationError::new(NavigationErrorKind::InvalidConfiguration, message)
}
