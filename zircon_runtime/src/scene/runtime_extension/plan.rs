use std::collections::BTreeSet;

use crate::scene::World;

use super::{WorldRuntimeExtensionError, WorldRuntimeExtensionRegistration};

#[derive(Clone, Debug, Default)]
/// 插件贡献的 World 注册顺序计划；合并只检查键冲突，应用时每个注册回调依次修改目标 World。
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

    // TODO: [CR-SCENE-WORLD-0001] 确认失败时是否要求原子应用；后续回调报错会保留前序注册，公开调用者重试可能再次遇到已注册项。
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
        let incoming = registrations.into_iter().collect::<Vec<_>>();
        let mut keys = self
            .registrations
            .iter()
            .map(WorldRuntimeExtensionRegistration::key)
            .collect::<BTreeSet<_>>();
        for registration in &incoming {
            if !keys.insert(registration.key()) {
                return Err(WorldRuntimeExtensionError::duplicate_registration(
                    registration.key(),
                ));
            }
        }
        drop(keys);
        self.registrations.extend(incoming);
        Ok(())
    }
}
