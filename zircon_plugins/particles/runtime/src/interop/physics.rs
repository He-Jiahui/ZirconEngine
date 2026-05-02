use zircon_runtime::core::math::{Real, Vec3};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticleOptionalFeatureStatus {
    Available,
    Unavailable { missing_capability: String },
}

impl ParticleOptionalFeatureStatus {
    pub fn from_capabilities<S: AsRef<str>>(required: &str, capabilities: &[S]) -> Self {
        if capabilities
            .iter()
            .any(|capability| capability.as_ref() == required)
        {
            Self::Available
        } else {
            Self::Unavailable {
                missing_capability: required.to_string(),
            }
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticlePhysicsOptions {
    pub external_force: Vec3,
    pub collision_enabled: bool,
    pub bounce: Real,
    pub damping: Real,
}

impl ParticlePhysicsOptions {
    pub fn disabled() -> Self {
        Self {
            external_force: Vec3::ZERO,
            collision_enabled: false,
            bounce: 0.0,
            damping: 0.0,
        }
    }
}

impl Default for ParticlePhysicsOptions {
    fn default() -> Self {
        Self::disabled()
    }
}
