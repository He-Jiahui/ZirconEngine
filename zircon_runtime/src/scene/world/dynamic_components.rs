//! World 内动态组件的 JSON 存储、描述符发布与 VM 模式同步。
//!
//! 组件类型路径用于持久名称；ECS 的组件编号与反射字段槽位各由所在目录解析。
//! 借用读取受 World 借用期约束，实例投影则复制 JSON 与可用的描述符。
//! 注册及 VM 同步在各自预检边界后发布；字段写入是否校验完整模式取决于 VM 动态标记。

use std::collections::HashMap;
use std::fmt::Write as _;

use serde_json::{Map, Number, Value};

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::{reflect::RuntimeTypeRegistration, EntityId};
use zircon_runtime_interface::reflect::ReflectError;
use zircon_runtime_interface::reflect::ReflectTypeRegistration;

use super::error::{SceneError, SceneResult};
use super::World;
use crate::scene::reflect::VmTypeBacking;

#[cfg(test)]
#[path = "dynamic_components/tests/registration_tests.rs"]
mod registration_tests;

#[path = "dynamic_components/creation.rs"]
mod creation;
pub use creation::{
    DynamicComponentCreationError, DynamicComponentCreationSpec, DynamicComponentCreationTemplate,
};

/// 某实体上一份动态组件的拥有式快照。
///
/// 由 [`World::dynamic_components_for_entity`] 复制生成，离开 World 借用后仍可持有；
/// `descriptor` 可以为空，例如空目录兼容路径下尚未声明的组件。
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicComponentInstance {
    /// 完整组件类型路径；快照排序也使用此名称。
    pub component_id: String,
    /// 复制出的 JSON 值；修改快照不会写回 World。
    pub value: Value,
    /// 生成快照时目录中的描述符副本，后续注册变化不会更新它。
    pub descriptor: Option<ComponentTypeDescriptor>,
}

impl World {
    /// 注册一个 VM 动态类型，并在发布前校验该路径已有的 JSON 实例。
    ///
    /// # 参数与返回
    /// `registration` 携带类型路径及完整字段声明，`backing` 选择 Runtime 存储适配器；
    /// 重名、描述符或存量实例不合法时返回错误，成功返回 `Ok(())`。
    ///
    /// # 发布边界
    /// 两个候选目录均在克隆上准备，校验通过后才替换当前目录并记录 VM 动态标记。
    /// 此入口不把路径加入目录托管集合；目录全量替换由内部 VM 同步入口负责。
    pub fn register_vm_type(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> SceneResult<()> {
        let type_path = registration.type_path.type_path().to_string();
        if self.type_registry.contains(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path }.into());
        }

        let mut type_registry = self.type_registry.clone();
        let descriptor = type_registry.register_vm_type_with_descriptor(registration, backing)?;
        let mut component_types = self.component_types.clone();
        component_types.register(descriptor)?;

        let registration = type_registry.registration(&type_path)?;
        self.validate_retained_vm_payloads(std::slice::from_ref(registration))?;

        self.component_types = component_types;
        self.type_registry = type_registry;
        self.vm_dynamic_type_paths.insert(type_path);
        Ok(())
    }

    /// 对当前 World 的完整 VM 候选目录作预检，不发布克隆目录或修改现有实例。
    ///
    /// `registrations` 借用仅用于本次检查；成功表示此刻的目录和存量值可接纳候选，
    /// 不预留以后提交的状态。跨 World 的加锁与再次校验由 LevelManager 协调。
    pub(crate) fn validate_vm_type_sync(
        &self,
        registrations: &[ReflectTypeRegistration],
    ) -> SceneResult<()> {
        self.prepare_vm_type_sync(registrations).map(|_| ())
    }

    /// 用完整 VM 候选集合更新此 World 的托管目录和动态类型标记。
    ///
    /// `registrations` 中不再出现的托管类型须无存活实例，保留实例须满足新字段声明。
    /// 任一预检错误均在当前目录替换前返回；成功一次替换准备好的四项状态。
    /// 跨 World 的提交边界见 LevelManager 的 VM 同步协调，本函数只处理一个 World。
    pub(crate) fn sync_vm_types(
        &mut self,
        registrations: &[ReflectTypeRegistration],
    ) -> SceneResult<()> {
        let (component_types, type_registry, vm_catalog_type_paths, vm_dynamic_type_paths) =
            self.prepare_vm_type_sync(registrations)?;
        self.component_types = component_types;
        self.type_registry = type_registry;
        self.vm_catalog_type_paths = vm_catalog_type_paths;
        self.vm_dynamic_type_paths = vm_dynamic_type_paths;
        Ok(())
    }

    /// 将插件描述符同时发布到动态描述符目录、反射目录和 ECS 组件编号目录。
    ///
    /// # 参数与返回
    /// `descriptor` 的插件前缀、类型重名、字段声明及反射登记均须通过预检；
    /// 任一步返回错误时三个当前目录尚未发布，成功后可用该类型附加 JSON 组件。
    ///
    /// # 生命周期
    /// 反射适配器拥有从描述符提取的模式资料，不借用传入描述符。
    /// ECS 编号通过同一预检增量发布；已有实例的值代数不因登记而递增。
    pub fn register_component_type(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> SceneResult<()> {
        let registration =
            crate::scene::reflect::registration_from_component_descriptor(&descriptor)?;
        self.component_types.validate_new_descriptor(&descriptor)?;
        if self.type_registry.contains(&descriptor.type_id) {
            return Err(ReflectError::DuplicateTypePath {
                type_path: descriptor.type_id.clone(),
            }
            .into());
        }
        let component =
            crate::scene::reflect::reflect_component_for_dynamic_descriptor(&descriptor);
        let component_type_id = descriptor.type_id.clone();
        let runtime_registration = RuntimeTypeRegistration {
            registration,
            component: Some(component),
            resource: None,
        };
        self.type_registry
            .validate_new_registration(&runtime_registration)?;

        let mut descriptor_imports = self
            .component_registry
            .begin_transferred_descriptor_imports();
        let retention_provider =
            crate::scene::ecs::ComponentRegistry::dynamic_retention_provider_v1::<
                super::typed_api::DynamicComponentPresence,
            >(Self::measure_dynamic_component_presence_retention_v1)
            .expect("dynamic component presence uses sparse-set storage");
        self.component_registry
            .preflight_dynamic_descriptor_import(
                &mut descriptor_imports,
                &component_type_id,
                retention_provider,
            )
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: component_type_id.clone(),
                reason: "dynamic presence retained owner descriptor conflict".to_string(),
            })?;

        self.component_registry
            .publish_preflighted_transferred_descriptor_imports(descriptor_imports);
        self.publish_prevalidated_dynamic_component_type(descriptor, runtime_registration);
        Ok(())
    }

    /// 发布已完成整体预检的描述符与反射适配器，不再返回可恢复的登记错误。
    ///
    /// 调用方须保证两项资料指向同一路径、两个目录均无重名，且 ECS 描述符增量
    /// 已按同一次预检发布。此入口供单次登记和动态场景事务的提交阶段共用。
    /// 调试断言只核对部分前提；不能用此入口替代两个目录的完整预检。
    pub(in crate::scene) fn publish_prevalidated_dynamic_component_type(
        &mut self,
        descriptor: ComponentTypeDescriptor,
        registration: RuntimeTypeRegistration,
    ) {
        let type_path = descriptor.type_id.clone();
        debug_assert!(!self.component_types.contains(&type_path));
        debug_assert!(!self.type_registry.contains_type_path(&type_path));
        self.component_types.publish_prevalidated(descriptor);
        self.type_registry.publish_prevalidated(registration);
    }

    /// 借用当前目录中指定完整类型路径的描述符；未登记返回 `None`。
    /// 返回引用受本次 World 共享借用约束，不能作为跨目录更新的快照。
    pub fn component_type_descriptor(&self, type_id: &str) -> Option<&ComponentTypeDescriptor> {
        self.component_types.descriptor(type_id)
    }

    /// Returns the dynamic schema revision for one component type.
    ///
    /// This changes when that type's descriptor changes, not when an instance
    /// value changes, so compiled field writers can rebind precisely.
    /// 返回该路径最近一次模式变更的代数；从未产生代数时为零，移除后仍保留变更记录。
    /// 此值用于编译字段绑定失效，不表示实例写入次数，也不是可永久唯一的类型编号。
    pub fn component_type_schema_generation(&self, type_id: &str) -> u64 {
        self.component_types.schema_generation(type_id)
    }

    /// 返回整个描述符目录的代数，供绑定时尚未声明类型的编译字段检测目录变化。
    pub(in crate::scene::world) fn component_type_schema_catalog_generation(&self) -> u64 {
        self.component_types.schema_catalog_generation()
    }

    /// 按完整类型路径排序收集描述符引用；结果借用当前 World，未复制描述符。
    pub fn component_type_descriptors(&self) -> Vec<&ComponentTypeDescriptor> {
        let descriptors = self.component_types.descriptors();
        let mut result = Vec::with_capacity(descriptors.size_hint().0);
        for descriptor in descriptors {
            result.push(descriptor);
        }
        result
    }

    /// 附加或替换实体的一整个动态组件值。
    ///
    /// # 参数与返回
    /// `entity` 必须存在；`component_id` 在非空描述符目录中必须已声明。
    /// VM 动态路径还须通过完整对象及字段类型检查；其他路径沿用兼容接纳规则。
    /// 返回 `false` 表示 JSON 与原值相等，`true` 表示成功写入了变化值。
    ///
    /// 成功变更会同步 ECS 存在标记、字段缓存、该类型的值代数和 World 代数；
    /// 模式代数不随实例值写入变化。输入 JSON 由 World 接管，读取引用不能跨后续可变借用。
    pub fn set_dynamic_component(
        &mut self,
        entity: EntityId,
        component_id: impl Into<String>,
        value: Value,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "attach dynamic component to",
                entity,
            ));
        }
        let component_id = component_id.into();
        self.validate_dynamic_component_type(&component_id)?;
        if self.vm_dynamic_type_paths.contains(&component_id) {
            self.validate_dynamic_component_value(&component_id, &value)?;
        }
        let components = self.dynamic_components.entry(entity).or_default();
        if components.get(&component_id) == Some(&value) {
            return Ok(false);
        }
        components.insert(component_id.clone(), value);
        self.insert_dynamic_component_presence(entity, &component_id)?;
        self.inspection_artifact_cache.mark_fields_dirty(entity);
        self.advance_dynamic_component_generation(&component_id);
        self.advance_world_generation();
        self.invalidate_world_component_type(&component_id);
        Ok(true)
    }

    /// 借用实体上指定类型的 JSON；实体映射或组件缺席时均返回 `None`。
    /// 此入口不复制值，也不重新校验 JSON 是否满足当前声明。
    pub fn dynamic_component(&self, entity: EntityId, component_id: &str) -> Option<&Value> {
        let components = self.dynamic_components.get(&entity)?;
        components.get(component_id)
    }

    /// 清空并填充指定动态类型的存活行，最后按稳定实体 ID 排序。
    ///
    /// `component_type_id` 先解析为当前 World 的 ECS 编号，再沿稀疏存储查找实例；
    /// 无编号、无实体位置或无对应 JSON 的行不会输出。`output` 可复用容量。
    /// 返回的 JSON 引用均借用同一个 World，不能在持有这些引用时可变访问该 World。
    pub(crate) fn dynamic_component_rows<'a>(
        &'a self,
        component_type_id: &str,
        output: &mut Vec<(EntityId, &'a Value)>,
    ) {
        output.clear();
        let Some(component_id) = self
            .component_registry
            .registered_dynamic_component_id(component_type_id)
        else {
            return;
        };

        output.reserve(self.component_storage.len_for_component(component_id));
        let dynamic_components = &self.dynamic_components;
        let entity_registry = &self.entity_registry;
        self.component_storage
            .for_each_sparse_entity(component_id, |internal| {
                let Ok(location) = entity_registry.location_for_internal(internal) else {
                    return;
                };
                let Some(value) = dynamic_components
                    .get(&location.stable_id)
                    .and_then(|components| components.get(component_type_id))
                else {
                    return;
                };
                output.push((location.stable_id, value));
            });
        output.sort_unstable_by_key(|(entity, _)| *entity);
    }

    /// 查询路径是否走 VM 动态实例的完整字段校验分支。
    pub(in crate::scene) fn is_vm_dynamic_type_path(&self, type_path: &str) -> bool {
        self.vm_dynamic_type_paths.contains(type_path)
    }

    /// 按类型路径排序复制实体上的全部动态组件，返回可独立持有的快照。
    ///
    /// 空结果不区分实体缺席与没有动态组件；描述符副本仅在当前目录能查到时提供。
    pub fn dynamic_components_for_entity(&self, entity: EntityId) -> Vec<DynamicComponentInstance> {
        let Some(components) = self.dynamic_components.get(&entity) else {
            return Vec::new();
        };
        let mut instances = Vec::with_capacity(components.len());
        for (component_id, value) in components {
            instances.push(DynamicComponentInstance {
                component_id: component_id.clone(),
                value: value.clone(),
                descriptor: self.component_types.descriptor(component_id).cloned(),
            });
        }
        instances.sort_by(|left, right| left.component_id.cmp(&right.component_id));
        instances
    }

    /// 移除实体上的动态组件；实体不存在报错，组件缺席返回 `false`。
    ///
    /// 成功删除后同步 ECS 存在标记并失效字段缓存及类型值代数；
    /// 空的实体组件映射会一并移除。此入口不会注销类型描述符或改动模式代数。
    pub fn remove_dynamic_component(
        &mut self,
        entity: EntityId,
        component_id: &str,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "remove dynamic component from",
                entity,
            ));
        }
        let Some(components) = self.dynamic_components.get_mut(&entity) else {
            return Ok(false);
        };
        let removed = components.remove(component_id).is_some();
        if components.is_empty() {
            self.dynamic_components.remove(&entity);
        }
        if removed {
            self.remove_dynamic_component_presence(entity, component_id)?;
            self.inspection_artifact_cache.mark_fields_dirty(entity);
            self.advance_dynamic_component_generation(component_id);
            self.advance_world_generation();
            self.invalidate_world_component_type(component_id);
        }
        Ok(removed)
    }

    /// 按插件前缀及紧随其后的点分隔符统计实例数，同一类型在不同实体上分别计数。
    pub fn dynamic_component_count_for_plugin(&self, plugin_id: &str) -> usize {
        let mut count = 0_usize;
        for components in self.dynamic_components.values() {
            for component_id in components.keys() {
                if dynamic_component_belongs_to_plugin(component_id, plugin_id) {
                    count += 1;
                }
            }
        }
        count
    }

    /// 检查该插件前缀下是否仍有动态组件实例，存在时返回携带实体诊断的错误。
    ///
    /// 此处只检查动态 JSON 实例；不移除组件、注销目录或执行插件卸载。
    /// 返回成功不代表其它插件资源、回调和句柄的生命周期检查已经完成。
    pub fn ensure_plugin_components_can_unload(&self, plugin_id: &str) -> SceneResult<()> {
        let mut active_components = String::new();
        let mut has_active_components = false;
        for (entity, components) in &self.dynamic_components {
            for component_id in components.keys() {
                if !dynamic_component_belongs_to_plugin(component_id, plugin_id) {
                    continue;
                }
                if has_active_components {
                    active_components.push_str(", ");
                }
                has_active_components = true;
                let _ = write!(&mut active_components, "{component_id} on entity {entity}");
            }
        }
        if !has_active_components {
            return Ok(());
        }
        Err(SceneError::PluginComponentsActive {
            plugin_id: plugin_id.to_string(),
            active_components,
        })
    }

    /// 按文本路径最后一个点拆出组件和单个属性名，返回拥有式场景属性值。
    /// 路径无法拆分、实例或字段缺席、JSON 不支持此投影时均返回 `None`；不递归遍历对象。
    pub(crate) fn dynamic_component_property(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> Option<ScenePropertyValue> {
        let (component_id, property) = split_dynamic_property_path(property_path)?;
        self.compiled_dynamic_component_property(entity, component_id, property)
    }

    /// 用已拆出的组件和属性段读取，避免重复解析文本路径。
    /// 返回值拥有其字符串或数值资料；模式新鲜度由编译绑定的调用方检查。
    pub(super) fn compiled_dynamic_component_property(
        &self,
        entity: EntityId,
        component_id: &str,
        property: &str,
    ) -> Option<ScenePropertyValue> {
        let value = self.dynamic_component(entity, component_id)?;
        let value = json_property(value, property)?;
        scene_value_from_json(value)
    }

    /// 先将场景属性投影为 JSON，再沿文本路径写入单个字段。
    ///
    /// 不支持的属性种类或非有限标量、向量会在写入前报错；
    /// 返回值及 VM 候选校验由共享字段写入入口决定。
    pub(crate) fn set_dynamic_component_property(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let Some(json_value) = json_from_scene_property_value(value) else {
            return Err(SceneError::DynamicComponentPropertyUnsupportedValue {
                property_path: property_path.to_string(),
            });
        };
        self.set_dynamic_component_json_property(entity, property_path, json_value)
    }

    /// 用编译绑定缓存的组件和属性段写入，复用与文本路径相同的字段接纳规则。
    ///
    /// `property_path` 仅供不支持值的错误诊断；实际写入位置由两个字符串段决定。
    /// 调用方须保证三者对应并已检查模式代数，不能仅凭缓存的字符串跳过重新绑定。
    pub(super) fn set_compiled_dynamic_component_property(
        &mut self,
        entity: EntityId,
        component_id: &str,
        property: &str,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        let Some(json_value) = json_from_scene_property_value(value) else {
            return Err(SceneError::DynamicComponentPropertyUnsupportedValue {
                property_path: property_path.to_string(),
            });
        };
        self.set_dynamic_component_json_field(entity, component_id, property, json_value)
    }

    /// 将文本路径拆为组件及单个字段后写入 JSON，无法拆分时返回未知属性错误。
    /// 原始 JSON 不经过场景属性转换；完整类型校验仅在 VM 动态分支执行。
    pub(crate) fn set_dynamic_component_json_property(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        json_value: Value,
    ) -> SceneResult<bool> {
        let Some((component_id, property)) = split_dynamic_property_path(property_path) else {
            return Err(SceneError::UnknownDynamicComponentProperty {
                property_path: property_path.to_string(),
            });
        };
        self.set_dynamic_component_json_field(entity, component_id, property, json_value)
    }

    /// 执行实体、类型及字段可编辑性检查，并按 VM 标记选择字段写入策略。
    ///
    /// VM 分支要求实例已存在，在副本中替换字段并验证完整候选后才写回；
    /// JSON 相等时直接返回 `false`。兼容分支可创建对象，但不执行 VM 完整模式检查。
    /// 两支均刷新缓存和类型、World 值代数；兼容分支还同步 ECS 存在标记，不改模式代数。
    fn set_dynamic_component_json_field(
        &mut self,
        entity: EntityId,
        component_id: &str,
        property: &str,
        json_value: Value,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update", entity));
        }
        self.validate_dynamic_component_type(component_id)?;
        self.validate_dynamic_component_property_write(component_id, property)?;
        if self.vm_dynamic_type_paths.contains(component_id) {
            let Some(component) = self.dynamic_component(entity, component_id) else {
                return Err(ReflectError::MissingComponent {
                    entity,
                    type_path: component_id.to_string(),
                }
                .into());
            };
            let Some(object) = component.as_object() else {
                return Err(SceneError::DynamicComponentNotObject {
                    component_id: component_id.to_string(),
                });
            };
            if object.get(property) == Some(&json_value) {
                return Ok(false);
            }
            let mut candidate = object.clone();
            candidate.insert(property.to_string(), json_value);
            let candidate = Value::Object(candidate);
            self.validate_dynamic_component_value(component_id, &candidate)?;
            // Preserve transactional validation: commit to the live component only
            // after the complete candidate has passed its declared VM schema.
            let Some(components) = self.dynamic_components.get_mut(&entity) else {
                return Err(ReflectError::MissingComponent {
                    entity,
                    type_path: component_id.to_string(),
                }
                .into());
            };
            let Some(component) = components.get_mut(component_id) else {
                return Err(ReflectError::MissingComponent {
                    entity,
                    type_path: component_id.to_string(),
                }
                .into());
            };
            *component = candidate;
            self.inspection_artifact_cache.mark_fields_dirty(entity);
            self.advance_dynamic_component_generation(component_id);
            self.advance_world_generation();
            self.invalidate_world_component_type(component_id);
            return Ok(true);
        }
        let components = self.dynamic_components.entry(entity).or_default();
        let component = components
            .entry(component_id.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(object) = component.as_object_mut() else {
            return Err(SceneError::DynamicComponentNotObject {
                component_id: component_id.to_string(),
            });
        };
        if object.get(property) == Some(&json_value) {
            return Ok(false);
        }
        object.insert(property.to_string(), json_value);
        self.insert_dynamic_component_presence(entity, component_id)?;
        self.inspection_artifact_cache.mark_fields_dirty(entity);
        self.advance_dynamic_component_generation(component_id);
        self.advance_world_generation();
        self.invalidate_world_component_type(component_id);
        Ok(true)
    }

    /// 非空目录要求类型已登记；空目录保留允许任意动态类型路径的兼容行为。
    fn validate_dynamic_component_type(&self, component_id: &str) -> SceneResult<()> {
        if self.component_types.is_empty() || self.component_types.contains(component_id) {
            return Ok(());
        }
        Err(SceneError::UnregisteredDynamicComponentType {
            component_id: component_id.to_string(),
        })
    }

    /// 在克隆目录上准备 VM 全量同步，返回拥有式候选目录及两组路径集合。
    ///
    /// 只清除旧托管集合中已撤销且无实例的类型；与非托管类型重名会报错。
    /// 入参声明还会校验当前保留的 JSON；任何错误都不会发布候选。
    /// 非托管的 VM 动态标记被保留，新托管集合来自本次候选路径。
    fn prepare_vm_type_sync(
        &self,
        registrations: &[ReflectTypeRegistration],
    ) -> SceneResult<(
        super::ComponentTypeRegistry,
        crate::scene::reflect::TypeRegistry,
        std::collections::BTreeSet<String>,
        std::collections::BTreeSet<String>,
    )> {
        let mut component_types = self.component_types.clone();
        let mut type_registry = self.type_registry.clone();
        let desired_type_paths = registrations
            .iter()
            .map(|registration| registration.type_path.type_path().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        for obsolete_type_path in self.vm_catalog_type_paths.difference(&desired_type_paths) {
            self.ensure_vm_catalog_type_unused(obsolete_type_path)?;
            component_types.remove_vm_descriptor(obsolete_type_path);
            type_registry.remove_vm_type(obsolete_type_path)?;
        }
        for registration in registrations {
            let type_path = registration.type_path.type_path();
            if type_registry.contains_type_path(type_path)
                && !self.vm_catalog_type_paths.contains(type_path)
            {
                return Err(ReflectError::DuplicateTypePath {
                    type_path: type_path.to_string(),
                }
                .into());
            }
            let backing = VmTypeBacking::DynamicComponent;
            let descriptor = type_registry.upsert_vm_type(registration.clone(), backing)?;
            component_types.upsert_vm_descriptor(descriptor)?;
        }
        self.validate_retained_vm_payloads(registrations)?;
        let mut vm_dynamic_type_paths = self
            .vm_dynamic_type_paths
            .difference(&self.vm_catalog_type_paths)
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        vm_dynamic_type_paths.extend(desired_type_paths.iter().cloned());
        Ok((
            component_types,
            type_registry,
            desired_type_paths,
            vm_dynamic_type_paths,
        ))
    }

    /// 撤销一个托管 VM 类型前检查其动态实例；发现实例则返回插件仍在使用的诊断。
    fn ensure_vm_catalog_type_unused(&self, type_path: &str) -> SceneResult<()> {
        let active_components = self
            .dynamic_components
            .iter()
            .filter_map(|(entity, components)| {
                components
                    .contains_key(type_path)
                    .then(|| format!("{type_path} on entity {entity}"))
            })
            .collect::<Vec<_>>();
        if active_components.is_empty() {
            return Ok(());
        }
        let plugin_id = self
            .type_registry
            .registration(type_path)?
            .type_path
            .plugin_id()
            .unwrap_or("<unknown>")
            .to_string();
        Err(SceneError::PluginComponentsActive {
            plugin_id,
            active_components: active_components.join(", "),
        })
    }

    /// 有非空属性声明时要求字段存在且可编辑；缺描述符或空属性表沿用兼容放行。
    /// 此检查只管字段声明与编辑权限，不代替 JSON 值类型或完整对象校验。
    fn validate_dynamic_component_property_write(
        &self,
        component_id: &str,
        property: &str,
    ) -> SceneResult<()> {
        let Some(descriptor) = self.component_types.descriptor(component_id) else {
            return Ok(());
        };
        if descriptor.properties.is_empty() {
            return Ok(());
        }
        let mut property_descriptor = None;
        for descriptor in &descriptor.properties {
            if descriptor.name == property {
                property_descriptor = Some(descriptor);
                break;
            }
        }
        let Some(property_descriptor) = property_descriptor else {
            return Err(SceneError::UndeclaredDynamicComponentProperty {
                component_id: component_id.to_string(),
                property: property.to_string(),
            });
        };
        if !property_descriptor.editable {
            return Err(SceneError::NonEditableDynamicComponentProperty {
                component_id: component_id.to_string(),
                property: property.to_string(),
            });
        }
        Ok(())
    }

    /// 按当前反射登记检查完整 JSON；没有该类型登记时放行，其他查询错误继续返回。
    fn validate_dynamic_component_value(
        &self,
        component_id: &str,
        value: &Value,
    ) -> SceneResult<()> {
        let registration = match self.type_registry.registration(component_id) {
            Ok(registration) => registration,
            Err(ReflectError::UnknownType { .. }) => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        Self::validate_dynamic_component_value_against_registration(registration, value)
    }

    /// 临时按类型分组借用所有现存 JSON，再校验候选声明对应的实例。
    ///
    /// `registrations` 为空时无需扫描；分组只保存引用，不复制或修改实例。
    /// 一处存量值不匹配便返回错误，供发布候选目录前拒绝模式刷新。
    fn validate_retained_vm_payloads(
        &self,
        registrations: &[ReflectTypeRegistration],
    ) -> SceneResult<()> {
        if registrations.is_empty() {
            return Ok(());
        }
        let mut payloads_by_type = HashMap::with_capacity(registrations.len());
        for components in self.dynamic_components.values() {
            for (type_path, value) in components {
                payloads_by_type
                    .entry(type_path.as_str())
                    .or_insert_with(Vec::new)
                    .push(value);
            }
        }
        for registration in registrations {
            let type_path = registration.type_path.type_path();
            let Some(payloads) = payloads_by_type.get(type_path) else {
                continue;
            };
            for value in payloads {
                Self::validate_dynamic_component_value_against_registration(registration, value)?;
            }
        }
        Ok(())
    }

    /// 要求 JSON 为对象，且键集合与登记字段完全对应，再逐字段检查声明类型。
    ///
    /// 未声明键和缺失声明键均返回未知字段错误；可空字段仍须提供对应键。
    /// 本函数不写入实例，不承担递归图预算，类型检查交由反射 JSON 接纳入口。
    fn validate_dynamic_component_value_against_registration(
        registration: &ReflectTypeRegistration,
        value: &Value,
    ) -> SceneResult<()> {
        let component_id = registration.type_path.type_path();
        let Some(object) = value.as_object() else {
            return Err(SceneError::DynamicComponentNotObject {
                component_id: component_id.to_string(),
            });
        };
        for (field_name, field_value) in object {
            let Some(field) = registration
                .type_info
                .fields
                .iter()
                .find(|field| field.name == *field_name)
            else {
                return Err(ReflectError::UnknownField {
                    type_path: component_id.to_string(),
                    field_name: field_name.clone(),
                }
                .into());
            };
            crate::scene::reflect::ensure_json_value_type(
                component_id,
                field_name,
                &field.value_type_path,
                field_value,
            )?;
        }
        for field in &registration.type_info.fields {
            if !object.contains_key(&field.name) {
                return Err(ReflectError::UnknownField {
                    type_path: component_id.to_string(),
                    field_name: field.name.clone(),
                }
                .into());
            }
        }
        Ok(())
    }
}

/// 从文本路径末尾借用拆出组件和属性段；不分配字符串，也不处理点号转义或嵌套字段。
fn split_dynamic_property_path(property_path: &ComponentPropertyPath) -> Option<(&str, &str)> {
    property_path.as_str().rsplit_once('.')
}

/// 仅匹配插件前缀后紧随点分隔符的组件名称，避免把相似名称算作同一插件。
fn dynamic_component_belongs_to_plugin(component_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = component_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}

fn json_property<'a>(value: &'a Value, property: &str) -> Option<&'a Value> {
    value.as_object()?.get(property)
}

/// 将可表示的 JSON 投影为场景属性：数值依次尝试有符号、无符号整数，再窄化为 f32。
///
/// 数组只接受 2、3、4 个数值，包装对象交给资源或实体投影；空值和不支持形状返回 `None`。
/// 浮点窄化在此不检查溢出后的有限性；该投影不代替 VM 模式或图预算验证。
fn scene_value_from_json(value: &Value) -> Option<ScenePropertyValue> {
    match value {
        Value::Bool(value) => Some(ScenePropertyValue::Bool(*value)),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                return Some(ScenePropertyValue::Integer(value));
            }
            if let Some(value) = value.as_u64() {
                return Some(ScenePropertyValue::Unsigned(value));
            }
            match value.as_f64() {
                Some(value) => Some(ScenePropertyValue::Scalar(value as _)),
                None => None,
            }
        }
        Value::String(value) => Some(ScenePropertyValue::String(value.clone())),
        Value::Array(values) => scene_vector_from_json(values),
        Value::Object(object) => scene_object_from_json(object),
        Value::Null => None,
    }
}

/// 将场景属性消费为动态组件使用的 JSON 表示。
///
/// 标量与向量的每个分量须有限；字符串与枚举写为字符串，实体与资源写为单键包装对象。
/// 四元数、动画参数及无法投影的数值返回 `None`，不会返回部分转换结果。
/// 该转换供动态反射适配器和 World 字段写入共用，不验证目标字段的声明类型。
pub(in crate::scene) fn json_from_scene_property_value(value: ScenePropertyValue) -> Option<Value> {
    match value {
        ScenePropertyValue::Bool(value) => Some(Value::Bool(value)),
        ScenePropertyValue::Integer(value) => Some(Value::Number(value.into())),
        ScenePropertyValue::Unsigned(value) => Some(Value::Number(value.into())),
        ScenePropertyValue::Scalar(value) => match finite_json_number(value) {
            Some(number) => Some(Value::Number(number)),
            None => None,
        },
        ScenePropertyValue::String(value) | ScenePropertyValue::Enum(value) => {
            Some(Value::String(value))
        }
        ScenePropertyValue::Vec2(value) => vector_to_json(value),
        ScenePropertyValue::Vec3(value) => vector_to_json(value),
        ScenePropertyValue::Vec4(value) => vector_to_json(value),
        ScenePropertyValue::Entity(value) => {
            let entity = match value {
                Some(entity) => Value::Number(Number::from(entity)),
                None => Value::Null,
            };
            Some(single_property_object("entity", entity))
        }
        ScenePropertyValue::Resource(value) => {
            Some(single_property_object("resource", Value::String(value)))
        }
        ScenePropertyValue::Quaternion(_) | ScenePropertyValue::AnimationParameter(_) => None,
    }
}

/// 只把长度为 2、3、4 的数值数组投影为对应 f32 向量；其它长度或非数值分量返回 `None`。
fn scene_vector_from_json(values: &[Value]) -> Option<ScenePropertyValue> {
    match values {
        [x, y] => Some(ScenePropertyValue::Vec2([
            json_number_as_f32(x)?,
            json_number_as_f32(y)?,
        ])),
        [x, y, z] => Some(ScenePropertyValue::Vec3([
            json_number_as_f32(x)?,
            json_number_as_f32(y)?,
            json_number_as_f32(z)?,
        ])),
        [x, y, z, w] => Some(ScenePropertyValue::Vec4([
            json_number_as_f32(x)?,
            json_number_as_f32(y)?,
            json_number_as_f32(z)?,
            json_number_as_f32(w)?,
        ])),
        _ => None,
    }
}

/// 从 JSON 数值读取 f64 后直接窄化为 f32；此辅助函数不额外保证窄化结果有限。
fn json_number_as_f32(value: &Value) -> Option<f32> {
    match value.as_f64() {
        Some(value) => Some(value as _),
        None => None,
    }
}

/// 优先识别字符串资源包装，其次识别空值或 u64 实体包装；其它对象返回 `None`。
/// 包装外的附加键不在此拒绝；需要严格模式的调用方须另行校验。
fn scene_object_from_json(object: &Map<String, Value>) -> Option<ScenePropertyValue> {
    if let Some(value) = object.get("resource") {
        if let Some(value) = value.as_str() {
            return Some(ScenePropertyValue::Resource(value.to_string()));
        }
    }
    if let Some(value) = object.get("entity") {
        return match value {
            Value::Null => Some(ScenePropertyValue::Entity(None)),
            Value::Number(number) => match number.as_u64() {
                Some(entity) => Some(ScenePropertyValue::Entity(Some(entity))),
                None => None,
            },
            _ => None,
        };
    }
    None
}

fn single_property_object(key: &str, value: Value) -> Value {
    let mut object = Map::with_capacity(1);
    object.insert(key.to_string(), value);
    Value::Object(object)
}

/// 按固定维数预留数组容量；任一分量无法生成有限 JSON 数值时返回 `None`。
fn vector_to_json<const N: usize>(values: [f32; N]) -> Option<Value> {
    let mut array = Vec::with_capacity(N);
    for value in values {
        array.push(Value::Number(finite_json_number(value)?));
    }
    Some(Value::Array(array))
}

/// 先采用 f32 的最短十进制文本，再解析为 JSON 数值，避免直接拓宽暴露二进制尾差。
/// 非有限输入、文本解析失败或 JSON 数值构造失败时返回 `None`。
fn finite_json_number(value: f32) -> Option<Number> {
    if !value.is_finite() {
        return None;
    }
    let text = value.to_string();
    let Ok(value) = text.parse::<f64>() else {
        return None;
    };
    Number::from_f64(value)
}
