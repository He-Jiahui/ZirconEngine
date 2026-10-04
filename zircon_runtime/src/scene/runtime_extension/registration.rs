use std::fmt;
use std::sync::Arc;

use crate::scene::World;

use super::WorldRuntimeExtensionError;

type WorldRuntimeExtensionApply =
    Arc<dyn Fn(&mut World) -> Result<(), WorldRuntimeExtensionError> + Send + Sync>;

#[derive(Clone)]
/// 一个稳定键与一个 World 安装回调；插件注册表生成这些项，WorldDriver 在世界初始化时应用。
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

    pub(super) fn apply(&self, world: &mut World) -> Result<(), WorldRuntimeExtensionError> {
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
