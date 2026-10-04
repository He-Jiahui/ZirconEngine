use std::any::TypeId;
use std::fmt;

use crate::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, RuntimeExtensionRegistryError,
};
use crate::scene::ecs::Event;
use crate::scene::{RuntimeEventMirrorError, RuntimeEventMirrorRegistration, SceneResult, World};
use serde::Serialize;

use super::super::owner::PluginModuleId;
use super::super::validation::validate_plugin_event_catalog_manifest;
use super::super::RuntimeExtensionRegistry;

/// 注册表按 PluginModuleId 保存类型身份、清单和 World 安装动作；克隆项可进入目录合并与逐 World 安装计划。
#[derive(Clone)]
pub struct EventRegistration {
    type_id: TypeId,
    type_name: &'static str,
    manifest: PluginEventManifest,
    apply: EventApply,
}

// 普通事件保存类型注册函数；镜像事件保存带订阅读者计数回调的注册对象，两种动作都在目标 World 安装。
#[derive(Clone)]
enum EventApply {
    Event(fn(&mut World)),
    Mirrored(RuntimeEventMirrorRegistration),
}

impl EventRegistration {
    fn new<E>(manifest: PluginEventManifest) -> Self
    where
        E: Event,
    {
        Self {
            type_id: TypeId::of::<E>(),
            type_name: std::any::type_name::<E>(),
            manifest,
            apply: EventApply::Event(|world| world.register_event::<E>()),
        }
    }

    // 回调被 move 进可克隆的镜像注册记录；Send/Sync 与 'static 约束不让它借用临时状态，但不保证动态库仍驻留。
    fn mirrored<E>(
        manifest: PluginEventManifest,
        reader_count_callback: impl Fn(&mut World, u32, Option<std::time::Instant>) -> SceneResult<()>
            + Send
            + Sync
            + 'static,
    ) -> Self
    where
        E: Event + Serialize,
    {
        Self {
            type_id: TypeId::of::<E>(),
            type_name: std::any::type_name::<E>(),
            apply: EventApply::Mirrored(
                RuntimeEventMirrorRegistration::typed::<E>(
                    manifest.id.clone(),
                    manifest.payload_schema.clone(),
                )
                .with_reader_count_callback_until(reader_count_callback),
            ),
            manifest,
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn manifest(&self) -> &PluginEventManifest {
        &self.manifest
    }

    // 安装计划会在每个目标 World 调用此动作；镜像分支将回调随注册副本存入 World 的镜像表。
    pub(in crate::plugin::extension_registry) fn apply(
        &self,
        world: &mut World,
    ) -> Result<(), RuntimeEventMirrorError> {
        match &self.apply {
            EventApply::Event(apply) => {
                apply(world);
                Ok(())
            }
            EventApply::Mirrored(registration) => {
                world.register_runtime_event_mirror(registration.clone())
            }
        }
    }
}

impl fmt::Debug for EventRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventRegistration")
            .field("type_name", &self.type_name)
            .field("manifest", &self.manifest)
            .finish_non_exhaustive()
    }
}

impl RuntimeExtensionRegistry {
    /// 以 owner 模块名首段派生插件事件命名空间，校验 manifest 后按 Rust TypeId 拒绝重复项，并登记目录与可执行事件。
    /// 链接式 RuntimePlugin::register 经注册报告调用此入口；独立 package event catalog 只提供元数据，不绑定 Rust 类型。
    /// 这是宿主进程内的 Rust Event 注册入口；当前 Native Host ABI 的 event emit/drain FFI 返回 UnsupportedVersion，不会转到此处。
    /// 安装函数会复制进运行时目录和 World 安装计划；若 E 来自可卸载模块，须先释放这些副本及其事件使用者再卸载代码。owner 撤销只移除注册表中的贡献。
    pub fn register_event<E>(
        &mut self,
        owner: PluginModuleId,
        manifest: PluginEventManifest,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        E: Event,
    {
        let namespace = self
            .plugin_modules
            .name(owner)
            .and_then(plugin_event_catalog_namespace_from_module)
            .ok_or_else(|| {
                RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                    "unknown plugin module owner {}",
                    owner.raw()
                ))
            })?;
        validate_event_manifest(&namespace, &manifest)?;
        let registration = EventRegistration::new::<E>(manifest.clone());
        if self.plugin_events.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                registration.type_name().to_string(),
            ));
        }
        self.push_derived_event_catalog_entry(namespace, manifest)?;
        self.register_event_registration(owner, registration)
    }

    /// 在同一 owner 与 manifest 校验路径上登记可序列化事件，并把回调保存到由 Arc 共享持有的镜像注册中。
    /// 回调在 World 镜像订阅读者数变化时执行，且会随注册副本留在已应用的 World；owner 撤销不会删除这些 World 副本。
    /// 若回调包含可卸载模块的代码，调用方必须等所有注册表副本、安装计划和 World 镜像记录释放后再卸载。
    pub fn register_mirrored_event<E>(
        &mut self,
        owner: PluginModuleId,
        manifest: PluginEventManifest,
        reader_count_callback: impl Fn(&mut World, u32) -> SceneResult<()> + Send + Sync + 'static,
    ) -> Result<(), RuntimeExtensionRegistryError>
    where
        E: Event + Serialize,
    {
        let namespace = self
            .plugin_modules
            .name(owner)
            .and_then(plugin_event_catalog_namespace_from_module)
            .ok_or_else(|| {
                RuntimeExtensionRegistryError::InvalidPluginModule(format!(
                    "unknown plugin module owner {}",
                    owner.raw()
                ))
            })?;
        validate_event_manifest(&namespace, &manifest)?;
        // 内部镜像回调也接收回收期限；此公共 API 只承诺 reader-count 通知，因此不向调用者转发期限。
        let registration =
            EventRegistration::mirrored::<E>(manifest.clone(), move |world, count, _deadline| {
                reader_count_callback(world, count)
            });
        if self.plugin_events.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                registration.type_name().to_string(),
            ));
        }
        self.push_derived_event_catalog_entry(namespace, manifest)?;
        self.register_event_registration(owner, registration)
    }

    // 目录合并会把来源表的注册项绑定到目标表 owner；重复检查使下方 expect 只表达已校验的不变量。
    pub(crate) fn register_event_registration(
        &mut self,
        owner: PluginModuleId,
        registration: EventRegistration,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        if self.plugin_events.contains_key(&registration.type_id) {
            return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                registration.type_name().to_string(),
            ));
        }
        self.plugin_events
            .register(owner, registration.type_id, registration)
            .expect("plugin event duplicate was prechecked");
        Ok(())
    }

    /// 连同 owner 暴露已注册事件，供运行时目录按模块筛选/重绑定，并供所有权撤销与 World 计划读取。
    pub fn plugin_events(&self) -> impl Iterator<Item = (PluginModuleId, &EventRegistration)> {
        self.plugin_events
            .iter()
            .map(|(owner, _key, registration)| (owner, registration))
    }

    // 找到同 namespace 的 catalog 时追加唯一 ID；否则驻留包级 runtime owner，并从当前 typed 注册派生 version 1 catalog。
    fn push_derived_event_catalog_entry(
        &mut self,
        namespace: String,
        manifest: PluginEventManifest,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        for catalog in self.plugin_event_catalogs.values_mut() {
            if catalog.namespace == namespace {
                if catalog.events.iter().any(|event| event.id == manifest.id) {
                    return Err(RuntimeExtensionRegistryError::DuplicatePluginEvent(
                        manifest.id,
                    ));
                }
                catalog.events.push(manifest);
                return Ok(());
            }
        }

        let owner = self.intern_owner_from_namespaced_key(&namespace)?;
        self.plugin_event_catalogs
            .register(
                owner,
                namespace.clone(),
                PluginEventCatalogManifest {
                    namespace,
                    version: 1,
                    events: vec![manifest],
                },
            )
            .expect("derived event catalog duplicate was prechecked");
        Ok(())
    }
}

// 模块名可包含子模块后缀；这里只取首个插件 token，让同包事件汇入同一事件目录。
fn plugin_event_catalog_namespace_from_module(module_name: &str) -> Option<String> {
    let plugin_id = module_name.split('.').next()?;
    if plugin_id.is_empty() {
        return None;
    }
    let capacity = plugin_id.len() + ".events".len();
    let mut namespace = String::with_capacity(capacity);
    namespace.push_str(plugin_id);
    namespace.push_str(".events");
    Some(namespace)
}

// 与 package catalog 共用校验器，使单个 typed 注册采用相同的 namespace、版本和 payload 约束。
fn validate_event_manifest(
    namespace: &str,
    manifest: &PluginEventManifest,
) -> Result<(), RuntimeExtensionRegistryError> {
    validate_plugin_event_catalog_manifest(&PluginEventCatalogManifest {
        namespace: namespace.to_string(),
        version: 1,
        events: vec![manifest.clone()],
    })
}

#[cfg(test)]
#[path = "tests/event_registration.rs"]
mod tests;
