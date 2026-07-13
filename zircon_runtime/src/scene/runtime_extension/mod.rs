use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use crate::scene::World;

type WorldRuntimeExtensionApply =
    Arc<dyn Fn(&mut World) -> Result<(), WorldRuntimeExtensionError> + Send + Sync>;

#[derive(Clone)]
pub struct WorldRuntimeExtensionRegistration {
    key: String,
    apply: WorldRuntimeExtensionApply,
}

impl WorldRuntimeExtensionRegistration {
    pub fn new(
        key: impl Into<String>,
        apply: impl Fn(&mut World) -> Result<(), WorldRuntimeExtensionError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            apply: Arc::new(apply),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    fn apply(&self, world: &mut World) -> Result<(), WorldRuntimeExtensionError> {
        (self.apply)(world)
    }
}

impl fmt::Debug for WorldRuntimeExtensionRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorldRuntimeExtensionRegistration")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorldRuntimeExtensionPlan {
    registrations: Vec<WorldRuntimeExtensionRegistration>,
}

impl WorldRuntimeExtensionPlan {
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = WorldRuntimeExtensionRegistration>,
    ) -> Result<Self, WorldRuntimeExtensionError> {
        let mut plan = Self::default();
        plan.append_unique(registrations)?;
        Ok(plan)
    }

    pub fn try_merge(&self, contribution: Self) -> Result<Self, WorldRuntimeExtensionError> {
        let mut candidate = self.clone();
        candidate.append_unique(contribution.registrations)?;
        Ok(candidate)
    }

    pub fn registration_count(&self) -> usize {
        self.registrations.len()
    }

    pub(crate) fn apply_to_world(
        &self,
        world: &mut World,
    ) -> Result<(), WorldRuntimeExtensionError> {
        for registration in &self.registrations {
            registration.apply(world)?;
        }
        Ok(())
    }

    fn append_unique(
        &mut self,
        registrations: impl IntoIterator<Item = WorldRuntimeExtensionRegistration>,
    ) -> Result<(), WorldRuntimeExtensionError> {
        let mut keys = self
            .registrations
            .iter()
            .map(|registration| registration.key.clone())
            .collect::<BTreeSet<_>>();
        let incoming = registrations.into_iter().collect::<Vec<_>>();
        for registration in &incoming {
            if !keys.insert(registration.key.clone()) {
                return Err(WorldRuntimeExtensionError::duplicate_registration(
                    &registration.key,
                ));
            }
        }
        self.registrations.extend(incoming);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldRuntimeExtensionError {
    diagnostic: String,
}

impl WorldRuntimeExtensionError {
    pub fn duplicate_registration(key: &str) -> Self {
        Self::new(format!("duplicate world runtime extension `{key}`"))
    }

    pub fn registration_failed(key: &str, diagnostic: impl fmt::Display) -> Self {
        Self::new(format!(
            "world runtime extension `{key}` failed: {diagnostic}"
        ))
    }

    pub fn new(diagnostic: impl Into<String>) -> Self {
        Self {
            diagnostic: diagnostic.into(),
        }
    }

    pub fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
}

impl fmt::Display for WorldRuntimeExtensionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic)
    }
}

impl std::error::Error for WorldRuntimeExtensionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_merge_preserves_the_previous_plan() {
        let base = WorldRuntimeExtensionPlan::from_registrations([registration("scene.tick")])
            .expect("base plan");
        let contribution =
            WorldRuntimeExtensionPlan::from_registrations([registration("scene.tick")])
                .expect("standalone contribution");

        assert!(base.try_merge(contribution).is_err());
        assert_eq!(base.registration_count(), 1);
    }

    fn registration(key: &str) -> WorldRuntimeExtensionRegistration {
        WorldRuntimeExtensionRegistration::new(key, |_| Ok(()))
    }
}
