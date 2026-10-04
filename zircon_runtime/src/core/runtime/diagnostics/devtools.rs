use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};

use crate::core::{CoreHandle, LifecycleState, ServiceKind, StartupMode};

/// 核心注册表的工具视图：模块、服务和目录信息供调试界面查询，不触发服务启动。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDevtoolsSnapshot {
    pub modules: Vec<RuntimeDevtoolsModuleSnapshot>,
    pub services: Vec<RuntimeDevtoolsServiceSnapshot>,
    pub plugin_catalog: Vec<RuntimeDevtoolsPluginCatalogEntry>,
    pub native_backend_status: RuntimeDevtoolsBackendStatus,
    pub vm_backend_status: RuntimeDevtoolsBackendStatus,
    pub diagnostics_summary: RuntimeDevtoolsDiagnosticsSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsModuleSnapshot {
    pub name: String,
    pub description: String,
    pub lifecycle: LifecycleState,
    pub service_count: usize,
    pub driver_count: usize,
    pub manager_count: usize,
    pub plugin_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsServiceSnapshot {
    pub name: String,
    pub owner_module: String,
    pub kind: ServiceKind,
    pub startup_mode: StartupMode,
    pub lifecycle: LifecycleState,
    pub dependencies: Vec<String>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsPluginCatalogEntry {
    pub package_id: String,
    pub display_name: String,
    pub crate_name: String,
    pub capabilities: Vec<String>,
    pub target_modes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsBackendStatus {
    pub backend: String,
    pub available: bool,
    pub loaded_plugin_count: usize,
}

impl Default for RuntimeDevtoolsBackendStatus {
    fn default() -> Self {
        Self {
            backend: String::new(),
            available: false,
            loaded_plugin_count: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDevtoolsDiagnosticsSummary {
    pub series_count: usize,
    pub tagged_subsystems: Vec<String>,
}

/// 从 CoreHandle 的注册表和诊断存储投影只读视图；调用者先采集诊断快照。
pub(crate) fn project_runtime_devtools_snapshot(
    core: &CoreHandle,
    diagnostics: &super::RuntimeDiagnosticsSnapshot,
) -> RuntimeDevtoolsSnapshot {
    RuntimeDevtoolsSnapshot {
        modules: collect_module_snapshots(core),
        services: collect_service_snapshots(core),
        plugin_catalog: collect_plugin_catalog_entries(core),
        // TODO: [CR-RUNTIME-DIAGNOSTICS-0001] 确认后端可用性与已加载数量的真实来源；当前常量投影尚未对应加载状态。
        native_backend_status: RuntimeDevtoolsBackendStatus {
            backend: "native_dynamic".to_string(),
            available: true,
            loaded_plugin_count: 0,
        },
        vm_backend_status: RuntimeDevtoolsBackendStatus {
            backend: "vm".to_string(),
            available: false,
            loaded_plugin_count: 0,
        },
        diagnostics_summary: RuntimeDevtoolsDiagnosticsSummary {
            series_count: diagnostics.store.series.len(),
            tagged_subsystems: tagged_subsystems(&diagnostics.store),
        },
    }
}

fn collect_module_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsModuleSnapshot> {
    let modules = lock_poison_recovered(&core.inner.modules);
    let mut snapshots = modules
        .values()
        .map(|entry| {
            let descriptor = entry.descriptor();
            RuntimeDevtoolsModuleSnapshot {
                name: descriptor.name.clone(),
                description: descriptor.description.clone(),
                lifecycle: entry.lifecycle,
                service_count: entry.service_names.len(),
                driver_count: descriptor.drivers.len(),
                manager_count: descriptor.managers.len(),
                plugin_count: descriptor.plugins.len(),
            }
        })
        .collect::<Vec<_>>();
    // 排序留在注册表锁之外，避免工具侧展示拖慢模块注册和生命周期推进。
    drop(modules);
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_service_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsServiceSnapshot> {
    let services = lock_poison_recovered(&core.inner.services);
    let mut snapshots = services
        .iter()
        .map(|(name, entry)| RuntimeDevtoolsServiceSnapshot {
            name: name.to_string(),
            owner_module: name.module_name().to_string(),
            kind: name.service_kind(),
            startup_mode: entry.startup_mode,
            lifecycle: entry.lifecycle,
            dependencies: entry
                .dependencies
                .iter()
                .map(|dependency| dependency.to_string())
                .collect(),
            active: entry.instance.is_some(),
        })
        .collect::<Vec<_>>();
    drop(services);
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_plugin_catalog_entries(core: &CoreHandle) -> Vec<RuntimeDevtoolsPluginCatalogEntry> {
    let mut entries = lock_poison_recovered(&core.inner.devtools_plugin_catalog_entries).clone();
    entries.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    entries
}

// 标签仅用于面板筛选摘要；借用存储快照中的文本并输出稳定顺序。
fn tagged_subsystems(store: &super::DiagnosticStoreSnapshot) -> Vec<String> {
    let mut unique_tags = HashSet::<&str>::new();
    for series in &store.series {
        unique_tags.extend(series.subsystem_tags.iter().map(String::as_str));
    }
    let mut tags = unique_tags
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    tags.sort_unstable();
    tags
}

fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[path = "tests/devtools.rs"]
mod tests;
