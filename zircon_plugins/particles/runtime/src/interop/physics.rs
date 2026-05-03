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

    pub fn with_external_force(mut self, external_force: Vec3) -> Self {
        self.external_force = external_force;
        self
    }

    pub fn with_collision(mut self, bounce: Real, damping: Real) -> Self {
        self.collision_enabled = true;
        self.bounce = bounce.max(0.0);
        self.damping = damping.clamp(0.0, 1.0);
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.external_force != Vec3::ZERO
            || self.collision_enabled
            || self.bounce > 0.0
            || self.damping > 0.0
    }
}

impl Default for ParticlePhysicsOptions {
    fn default() -> Self {
        Self::disabled()
    }
}
