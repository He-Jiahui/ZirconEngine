use std::collections::HashMap;
use std::sync::Arc;

use crate::plugin::RuntimeExtensionRegistryError;

/// 单个注册表生命周期内的模块身份；raw 值只在其来源注册表内有意义。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PluginModuleId(u32);

impl PluginModuleId {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

// 名称驻留器让同名模块在重复登记与目录合并期间取得一致 owner，克隆后共享名称存储。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::plugin::extension_registry) struct PluginModuleInterner {
    names: Vec<Arc<str>>,
    ids_by_name: HashMap<Arc<str>, PluginModuleId>,
}

impl PluginModuleInterner {
    pub(in crate::plugin::extension_registry) fn intern(
        &mut self,
        name: impl Into<String>,
    ) -> Result<PluginModuleId, RuntimeExtensionRegistryError> {
        let name = name.into();
        validate_plugin_module_name(&name)?;
        if let Some(id) = self.ids_by_name.get(name.as_str()).copied() {
            return Ok(id);
        }

        let id = PluginModuleId::from_raw(self.names.len() as u32);
        let name: Arc<str> = name.into();
        self.names.push(Arc::clone(&name));
        self.ids_by_name.insert(name, id);
        Ok(id)
    }

    pub(in crate::plugin::extension_registry) fn name(&self, id: PluginModuleId) -> Option<&str> {
        self.names.get(id.index()).map(AsRef::as_ref)
    }
}

// 这里只验证基础模块形状；需要 `.runtime` 归属的扩展家族由各自注册入口再检查。
fn validate_plugin_module_name(name: &str) -> Result<(), RuntimeExtensionRegistryError> {
    if name.trim().is_empty() || name.trim() != name || !name.contains('.') {
        return Err(RuntimeExtensionRegistryError::InvalidPluginModule(
            name.to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/owner.rs"]
mod tests;
