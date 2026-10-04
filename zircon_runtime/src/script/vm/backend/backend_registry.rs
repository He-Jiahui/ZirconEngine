//! 后端选择器由管理器在装载前解析：限定名前缀锁定家族，旧式裸名允许家族逐个解析；注册表只持有后端工厂，不持有运行中的插件实例。
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use super::{VmBackend, VmBackendFamily, VmError};

#[derive(Default)]
pub struct VmBackendRegistry {
    families: Mutex<BTreeMap<String, Arc<dyn VmBackendFamily>>>,
}

impl fmt::Debug for VmBackendRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VmBackendRegistry")
            .field("families", &self.names())
            .finish()
    }
}

impl VmBackendRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock_families(&self) -> MutexGuard<'_, BTreeMap<String, Arc<dyn VmBackendFamily>>> {
        self.families
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 同名家族替换后续解析结果；已装载实例仍由原协调器持有，不会在这里切换。
    pub fn register_family(&self, family: Arc<dyn VmBackendFamily>) -> String {
        let name = family.family_name().to_string();
        self.lock_families().insert(name.clone(), family);
        name
    }

    pub fn resolve(&self, selector: &str) -> Result<Arc<dyn VmBackend>, VmError> {
        if let Some((family_name, _)) = selector.split_once(':') {
            let family = self.lock_families().get(family_name).cloned();
            if let Some(family) = family {
                return family.resolve(selector);
            }
        }

        let families = self.lock_families().values().cloned().collect::<Vec<_>>();
        for family in families {
            if let Ok(backend) = family.resolve(selector) {
                return Ok(backend);
            }
        }

        Err(VmError::UnknownBackend(selector.to_string()))
    }

    pub fn contains(&self, selector: &str) -> bool {
        self.resolve(selector).is_ok()
    }

    pub fn names(&self) -> Vec<String> {
        let families = self.lock_families();
        let mut selectors = Vec::new();
        for family in families.values() {
            family.visit_selectors(&mut |selector| selectors.push(selector.to_owned()));
        }
        selectors.sort();
        selectors.dedup();
        selectors
    }
}

#[cfg(test)]
#[path = "tests/backend_registry.rs"]
mod tests;

#[cfg(test)]
#[path = "backend_registry/tests/qualified_lookup_tests.rs"]
mod qualified_lookup_tests;
