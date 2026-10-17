use std::collections::{BTreeMap, HashSet};
use std::fmt;

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectSchemaCatalog,
    ReflectSchemaCatalogEntry, ReflectTypeRegistration, ReflectedValue,
};

use crate::core::framework::scene::ComponentTypeDescriptor;

use super::declared_value_type::DeclaredValueType;
use super::runtime_type_registration::RuntimeTypeRegistration;
use super::value_admission::validate_reflected_value_contract;
use super::vm_type_backing::VmTypeBacking;

// 这些上限约束单条 Runtime 注册的元数据。字段数与枚举选项数按条目计；展示名称和选项值按 UTF-8 字节计，选项值另受 ASCII 字符集限制。单字段与全类型枚举选项预算分别检查。
const MAX_REFLECT_FIELDS_PER_TYPE: usize = 4_096;
const MAX_REFLECT_FIELD_DISPLAY_NAME_BYTES: usize = 256;
const MAX_REFLECT_ENUM_OPTIONS_PER_FIELD: usize = 4_096;
const MAX_REFLECT_ENUM_OPTIONS_PER_TYPE: usize = 16_384;
const MAX_REFLECT_ENUM_VALUE_BYTES: usize = 128;
const MAX_REFLECT_ENUM_DISPLAY_NAME_BYTES: usize = 256;

#[derive(Clone, Default)]
/// World 拥有的类型与稳定字段 ID 目录；插件或 VM 模式变更应通过注册入口推进目录代数。
/// 维护 World 自有的规范化反射目录及其 Runtime 适配器投影。
///
/// # 状态与生命周期
///
/// `registrations` 拥有按完整类型路径排序的 Runtime 注册；`schema_catalog` 负责类型和稳定字段身份，发布时将其规范化元数据同步回 Runtime 注册。`schema_catalog_generation` 是本表的饱和版本计数。`Clone` 复制本表，返回的注册或路径借用仍依附被查询的实例。World 持久状态恢复会重建内建反射注册；本表不是可序列化回调存储。
///
/// # 约束
///
/// 注册接口处理目录及适配器表；World 上的组件描述符、实例数据和 VM 保留载荷由 World 对应入口协调。调用者须将适配器槽位语义与注册字段顺序配对。
///
/// # 参考
///
/// `register`、`schema_catalog_generation`；`crate::scene::World::register_vm_type`、`crate::scene::reflect::register_builtin_reflection`。
pub struct TypeRegistry {
    registrations: BTreeMap<String, RuntimeTypeRegistration>,
    schema_catalog: ReflectSchemaCatalog,
    schema_catalog_generation: u64,
}

impl TypeRegistry {
    /// 验证并发布一条 Runtime 反射注册。
    ///
    /// # 参数
    ///
    /// `registration` 将元数据及可选适配器表的所有权交给本目录；元数据注册可不带实时访问适配器。
    ///
    /// # 返回值
    ///
    /// 成功后两份目录投影一致并推进版本；重复身份、目录依赖或字段元数据无效时返回 `ReflectError`。
    ///
    /// # 约束与生命周期
    ///
    /// 可返回错误的准入检查先于发布。本入口不调用适配器，也不创建 World 组件或资源实例；适配器访问能力由后续门面检查。
    ///
    /// # 参考
    ///
    /// `validate_new_registration`、`publish_prevalidated`；`WorldReflection` 的实时访问路径。
    pub fn register(&mut self, registration: RuntimeTypeRegistration) -> Result<(), ReflectError> {
        self.validate_new_registration(&registration)?;
        self.publish_prevalidated(registration);
        Ok(())
    }

    /// 只读预检候选 Runtime 注册及其目录插入资格。
    ///
    /// # 参数
    ///
    /// `registration` 是待发布候选的共享借用。
    ///
    /// # 返回值
    ///
    /// Runtime 字段规则和中立目录准入均通过时返回 `Ok(())`；失败不修改本表。
    ///
    /// # 约束与生命周期
    ///
    /// 预检不预留类型或字段身份。后续 `publish_prevalidated` 必须针对此候选和未发生使预检失效的目录状态执行。
    ///
    /// # 参考
    ///
    /// `validate_registration`、`ReflectSchemaCatalog::validate_insert`；World 动态组件描述符的预检/发布路径。
    pub(crate) fn validate_new_registration(
        &self,
        registration: &RuntimeTypeRegistration,
    ) -> Result<(), ReflectError> {
        validate_registration(&registration.registration)?;
        self.schema_catalog
            .validate_insert(&ReflectSchemaCatalogEntry::new(
                registration.registration.clone(),
            ))
    }

    /// 发布已针对当前目录预检的注册，并同步规范化元数据与版本。
    ///
    /// # 参数
    ///
    /// `registration` 是将被本表接管的候选。
    ///
    /// # 约束与生命周期
    ///
    /// 调用者先执行 `validate_new_registration`，并在发布前保持该候选的准入条件。发布顺序为 schema catalog、规范化 Runtime 元数据、Runtime map 和版本计数。这里没有可恢复的错误返回；前提失效会触发断言或 `expect`。
    ///
    /// # 参考
    ///
    /// `register`；`crate::scene::World::publish_prevalidated_dynamic_component_type`。
    pub(crate) fn publish_prevalidated(&mut self, mut registration: RuntimeTypeRegistration) {
        debug_assert!(self.validate_new_registration(&registration).is_ok());
        let type_path = registration.registration.type_path.type_path().to_string();
        self.schema_catalog
            .try_insert(ReflectSchemaCatalogEntry::new(
                registration.registration.clone(),
            ))
            .expect("prevalidated reflection schema entry must publish");
        registration.registration = self
            .schema_catalog
            .registration(&type_path)
            .expect("published schema entry must resolve")
            .clone();
        let previous = self.registrations.insert(type_path, registration);
        debug_assert!(previous.is_none());
        self.advance_schema_catalog_generation();
    }

    /// 为资源类型安装反射元数据和访问适配器。
    ///
    /// # 参数
    ///
    /// `registration` 必须是资源角色；`adapter` 是按具体 World 资源类型访问的函数表，随注册交给目录。
    ///
    /// # 返回值
    ///
    /// 角色不符返回 `InvalidRegistration`，其余准入错误由 `register` 返回。
    ///
    /// # 约束与生命周期
    ///
    /// 成功只安装 schema/适配器；资源实例仍由 World 资源接口创建和拥有，访问不存在的实例可返回 `MissingResource`。
    ///
    /// # 参考
    ///
    /// `register`、`crate::scene::reflect::ReflectResource`；`WorldReflection::reflect_write`。
    pub fn register_resource(
        &mut self,
        registration: ReflectTypeRegistration,
        adapter: crate::scene::reflect::ReflectResource,
    ) -> Result<(), ReflectError> {
        if !registration.is_resource() {
            return Err(ReflectError::InvalidRegistration {
                type_path: registration.type_path.type_path().to_string(),
                reason: "resource adapters require resource-only registrations".to_string(),
            });
        }

        self.register(RuntimeTypeRegistration {
            registration,
            component: None,
            resource: Some(adapter),
        })
    }

    /// 将 VM 类型接纳到本反射表。
    ///
    /// # 参数
    ///
    /// `registration` 是插件拥有的 VM 元数据；`backing` 指定 Runtime 数据承载方式。
    ///
    /// # 返回值
    ///
    /// 成功发布本表注册；失败返回 VM/目录准入错误。
    ///
    /// # 约束与生命周期
    ///
    /// 此入口丢弃生成的组件描述符，不协调 World 的组件目录或已存在载荷。需要 World 数据与目录一起接纳时使用 World 的对应注册入口。
    ///
    /// # 参考
    ///
    /// `register_vm_type_with_descriptor`、`crate::scene::World::register_vm_type`。
    pub fn register_vm_type(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<(), ReflectError> {
        self.register_vm_type_with_descriptor(registration, backing)?;
        Ok(())
    }

    /// 接纳 VM 元数据并返回用于 World 同步的组件描述符。
    ///
    /// # 参数
    ///
    /// `registration` 的字段顺序用于 schema 与动态适配器；`backing` 当前支持动态组件。
    ///
    /// # 返回值
    ///
    /// 发布成功后返回拥有自身文本和属性列表的 `ComponentTypeDescriptor`；重复路径或准入失败返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 先构造并验证描述符，再验证目录插入，最后发布。调用者负责将返回描述符与本表候选一起接入 World。
    ///
    /// # 参考
    ///
    /// `vm_component_descriptor`、`validate_new_registration`；`crate::scene::World::register_vm_type`。
    pub(crate) fn register_vm_type_with_descriptor(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let type_path = registration.type_path.type_path().to_string();
        if self.registrations.contains_key(&type_path) {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        let descriptor = Self::vm_component_descriptor(&registration, backing)?;
        match backing {
            VmTypeBacking::DynamicComponent => {
                let component =
                    super::dynamic_component::reflect_component_for_dynamic_descriptor(&descriptor);
                let runtime_registration = RuntimeTypeRegistration {
                    registration,
                    component: Some(component),
                    resource: None,
                };
                self.validate_new_registration(&runtime_registration)?;
                self.publish_prevalidated(runtime_registration);
            }
        }
        Ok(descriptor)
    }

    /// 在 VM 同步候选中新增或替换同一插件拥有的反射注册。
    ///
    /// # 参数
    ///
    /// `registration` 是新的完整元数据；`backing` 决定动态组件适配器。
    ///
    /// # 返回值
    ///
    /// 返回对应组件描述符；已有类型的插件归属不符或候选目录无效时返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 不存在时执行新注册准入。相等的 Runtime 注册不替换也不推进版本；这里的相等比较元数据及适配器存在性。替换先让 catalog 验证完整候选，再同步 Runtime 投影并推进版本。World 的调用者另行限制可被 VM catalog 更新的类型范围。
    ///
    /// # 参考
    ///
    /// `RuntimeTypeRegistration::eq`、`ReflectSchemaCatalog::try_replace`；World 的 `prepare_vm_type_sync`。
    pub(crate) fn upsert_vm_type(
        &mut self,
        registration: ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let descriptor = Self::vm_component_descriptor(&registration, backing)?;
        let type_path = registration.type_path.type_path().to_string();
        let mut replacement = match backing {
            VmTypeBacking::DynamicComponent => RuntimeTypeRegistration {
                component: Some(
                    super::dynamic_component::reflect_component_for_dynamic_descriptor(&descriptor),
                ),
                registration,
                resource: None,
            },
        };
        let Some(existing) = self.registrations.get(&type_path) else {
            self.validate_new_registration(&replacement)?;
            self.publish_prevalidated(replacement);
            return Ok(descriptor);
        };
        if !existing.registration.is_plugin_owned()
            || existing.registration.type_path.plugin_id()
                != replacement.registration.type_path.plugin_id()
        {
            return Err(ReflectError::DuplicateTypePath { type_path });
        }
        if existing == &replacement {
            return Ok(descriptor);
        }

        self.schema_catalog
            .try_replace(ReflectSchemaCatalogEntry::new(
                replacement.registration.clone(),
            ))?;
        replacement.registration = self.schema_catalog.registration(&type_path)?.clone();
        self.registrations.insert(type_path.clone(), replacement);
        self.advance_schema_catalog_generation();
        Ok(descriptor)
    }

    /// 从 VM 同步候选中移除插件拥有的组件注册。
    ///
    /// # 参数
    ///
    /// `type_path` 是完整类型路径键；本入口不解析短名称。
    ///
    /// # 返回值
    ///
    /// 不存在时成功且不推进版本；角色/适配器条件不符或仍有目录依赖时返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 先完成 catalog 删除准入，再删除 Runtime 投影并推进版本。World 组件实例或载荷是否可移除，由同步调用者在此之前检查。
    ///
    /// # 参考
    ///
    /// `ReflectSchemaCatalog::try_remove`；World 的 `prepare_vm_type_sync`、`ensure_vm_catalog_type_unused`。
    pub(crate) fn remove_vm_type(&mut self, type_path: &str) -> Result<(), ReflectError> {
        let Some(existing) = self.registrations.get(type_path) else {
            return Ok(());
        };
        if !existing.registration.is_plugin_owned() || existing.component.is_none() {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason:
                    "only plugin-owned VM component registrations may be removed by the VM catalog"
                        .to_string(),
            });
        }
        self.schema_catalog.try_remove(type_path)?;
        self.registrations.remove(type_path);
        self.advance_schema_catalog_generation();
        Ok(())
    }

    /// 将已满足 VM 约束的反射声明转换为组件描述符。
    ///
    /// # 参数
    ///
    /// `registration` 是只读元数据，`backing` 指定承载方式。
    ///
    /// # 返回值
    ///
    /// 返回拥有类型路径、插件身份、展示名及字段属性的描述符；不修改注册表。
    ///
    /// # 约束与生命周期
    ///
    /// 先执行 VM 专项校验，按注册字段原顺序复制名称、值类型和可编辑标记。返回值不借用输入。
    ///
    /// # 参考
    ///
    /// `validate_vm_registration`、`register_vm_type_with_descriptor`。
    pub(crate) fn vm_component_descriptor(
        registration: &ReflectTypeRegistration,
        backing: VmTypeBacking,
    ) -> Result<ComponentTypeDescriptor, ReflectError> {
        let plugin_id = validate_vm_registration(registration, backing)?.to_string();
        let mut descriptor = ComponentTypeDescriptor::new(
            registration.type_path.type_path().to_string(),
            plugin_id,
            registration.display_name.clone(),
        );
        for field in &registration.type_info.fields {
            descriptor = descriptor.with_property(
                field.name.clone(),
                field.value_type_path.clone(),
                field.editable,
            );
        }
        Ok(descriptor)
    }

    /// 查询规范化的反射元数据。
    ///
    /// # 参数
    ///
    /// `type_path` 可为完整路径或当前目录中唯一的短路径。
    ///
    /// # 返回值
    ///
    /// 返回依附本表的元数据借用；未知类型或短名歧义返回目录错误。
    ///
    /// # 约束与生命周期
    ///
    /// 共享借用有效期间不能改变同一注册表。查询不要求实时访问适配器或 World 实例存在。
    ///
    /// # 参考
    ///
    /// `ReflectSchemaCatalog::registration`、`runtime_registration`。
    pub fn registration(&self, type_path: &str) -> Result<&ReflectTypeRegistration, ReflectError> {
        self.schema_catalog.registration(type_path)
    }

    /// 查询与目录身份对应的 Runtime 注册及适配器投影。
    ///
    /// # 参数
    ///
    /// `type_path` 先作为完整键查询，再按目录规则解析短路径。
    ///
    /// # 返回值
    ///
    /// 返回依附本表的注册借用；未知或歧义由目录报错，目录与 Runtime 投影不一致返回 `InvalidRegistration`。
    ///
    /// # 约束与生命周期
    ///
    /// 适配器可能为空；可否访问组件或资源由门面按角色及适配器存在性判定。
    ///
    /// # 参考
    ///
    /// `resolve`、`WorldReflection` 的适配器选择路径。
    pub fn runtime_registration(
        &self,
        type_path: &str,
    ) -> Result<&RuntimeTypeRegistration, ReflectError> {
        if let Some(registration) = self.registrations.get(type_path) {
            return Ok(registration);
        }

        let resolved = self.schema_catalog.resolve_type_path(type_path)?;
        self.registrations
            .get(resolved)
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: resolved.to_string(),
                reason: "schema catalog entry is missing its runtime adapter projection"
                    .to_string(),
            })
    }

    /// 将完整路径或唯一短路径解析为目录拥有的完整路径。
    ///
    /// # 参数
    ///
    /// `type_path` 是查询文本，调用结束后无需保留该输入字符串。
    ///
    /// # 返回值
    ///
    /// 成功返回借用本目录的规范路径；未知或歧义返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 返回借用的有效期随本表，不能据此跨目录变更保存未检查的访问计划。
    ///
    /// # 参考
    ///
    /// `ReflectSchemaCatalog::resolve_type_path`、`schema_catalog_generation`。
    pub fn resolve(&self, type_path: &str) -> Result<&str, ReflectError> {
        self.schema_catalog.resolve_type_path(type_path)
    }

    /// 把稳定字段身份解析为当前注册的稠密槽位。
    ///
    /// # 参数
    ///
    /// `type_path` 标识目录中的类型；`field_id` 是稳定字段 ID。
    ///
    /// # 返回值
    ///
    /// 返回当前字段序列中的零基 `u32` 索引；未知类型或字段返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 槽位仅对本次 schema 顺序有效。持久引用使用稳定 ID；缓存计划应绑定目录版本，并在使用前检查。
    ///
    /// # 参考
    ///
    /// `ReflectSchemaCatalog::field_slot_by_id`、`schema_catalog_generation`；`WorldReflection` 的 `field_access`。
    pub(crate) fn resolve_field_slot_by_id(
        &self,
        type_path: &str,
        field_id: ReflectFieldId,
    ) -> Result<u32, ReflectError> {
        self.schema_catalog.field_slot_by_id(type_path, field_id)
    }

    /// 按完整类型路径键序遍历 Runtime 注册。
    ///
    /// # 返回值
    ///
    /// 迭代器逐项借用本表中的注册；次序由 `BTreeMap` 键序决定。
    ///
    /// # 约束与生命周期
    ///
    /// 迭代期间共享借用注册表；目录不能在同一借用期间改变。
    ///
    /// # 参考
    ///
    /// 动态场景捕获对组件/资源的 `serializable` 与适配器筛选。
    pub fn iter(&self) -> impl Iterator<Item = &RuntimeTypeRegistration> {
        self.registrations.values()
    }

    /// 判断完整路径或唯一短路径是否能在目录中解析。
    ///
    /// # 参数
    ///
    /// `type_path` 是待查询路径。
    ///
    /// # 返回值
    ///
    /// 有完整项或唯一短名时返回 `true`；未知或歧义短名返回 `false`。
    ///
    /// # 约束
    ///
    /// 此判断只涉及目录身份，不检查适配器或实例；需要区分失败原因时调用 `resolve`。
    ///
    /// # 参考
    ///
    /// `resolve`、`contains_type_path`。
    pub fn contains(&self, type_path: &str) -> bool {
        self.schema_catalog.contains(type_path)
    }

    /// 判断目录是否包含指定完整类型路径键。
    ///
    /// # 参数
    ///
    /// `type_path` 按完整键匹配。
    ///
    /// # 返回值
    ///
    /// 完整键存在时返回 `true`。
    ///
    /// # 约束
    ///
    /// 需要短路径解析时使用 `contains` 或 `resolve`。
    ///
    /// # 参考
    ///
    /// World VM 同步对已有路径归属的检查。
    pub fn contains_type_path(&self, type_path: &str) -> bool {
        self.schema_catalog.contains_type_path(type_path)
    }

    /// 借用本表的中立模式目录。
    ///
    /// # 返回值
    ///
    /// 返回目录身份、字段槽位索引和 fingerprint 的只读访问对象。
    ///
    /// # 约束与生命周期
    ///
    /// 借用归属于本表；所有 Runtime 注册变更应通过本表的接纳入口同步两份投影。
    ///
    /// # 参考
    ///
    /// `registration`、`resolve_field_slot_by_id`、`WorldReflection::list_reflect_types`。
    pub fn schema_catalog(&self) -> &ReflectSchemaCatalog {
        &self.schema_catalog
    }

    /// Returns the revision of the registered reflection catalog.
    ///
    /// Dynamic-scene spawn plans bind this value so a reflected registration
    /// change cannot reuse a plan compiled against an older schema catalog.
    /// 取得本反射目录的当前版本计数。
    ///
    /// # 返回值
    ///
    /// 返回本表持有的 `u64` 计数；增量采用饱和加法，到达 `u64::MAX` 后不再增大。
    ///
    /// # 约束与生命周期
    ///
    /// 成功发布目录变化以及非空清空时推进计数。动态场景计划将它与编译时版本比较；该值不单独标识 World，也不是内容 fingerprint。
    ///
    /// # 参考
    ///
    /// `advance_schema_catalog_generation`；动态场景 spawn 计划的编译和使用前检查。
    pub fn schema_catalog_generation(&self) -> u64 {
        self.schema_catalog_generation
    }

    /// 清空 Runtime 注册与模式目录。
    ///
    /// # 约束与生命周期
    ///
    /// 原表非空时推进目录版本，已为空时不推进。本操作仅处理反射表，不清理 World 的实例数据或组件描述符。
    ///
    /// # 参考
    ///
    /// `schema_catalog_generation`、World 的目录重建入口。
    pub fn clear(&mut self) {
        let had_registrations = !self.registrations.is_empty();
        self.registrations.clear();
        self.schema_catalog.clear();
        if had_registrations {
            self.advance_schema_catalog_generation();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    fn advance_schema_catalog_generation(&mut self) {
        self.schema_catalog_generation = self.schema_catalog_generation.saturating_add(1);
    }
}

/// 校验 VM 类型在 Runtime 通用准入之外的插件与承载约束。
///
/// # 参数
///
/// `registration` 是待转换的元数据借用；`backing` 指定 Runtime 承载类型。
///
/// # 返回值
///
/// 成功返回借用注册内部的插件 ID；失败返回含类型上下文的错误。
///
/// # 约束与生命周期
///
/// 要求插件拥有、声明插件 ID、完整路径具备插件前缀，且字段类型使用 VM grammar。当前动态承载只接受组件角色；插件 ID 借用不得超过输入注册的有效期。
///
/// # 参考
///
/// `validate_registration`、`DeclaredValueType::parse_vm`、`vm_component_descriptor`。
fn validate_vm_registration(
    registration: &ReflectTypeRegistration,
    backing: VmTypeBacking,
) -> Result<&str, ReflectError> {
    validate_registration(registration)?;
    let type_path = registration.type_path.type_path();
    validate_canonical_text(type_path, "display name", &registration.display_name)?;
    if !registration.is_plugin_owned() {
        return Err(invalid_vm_registration(
            type_path,
            "VM types must be plugin-owned",
        ));
    }
    let Some(plugin_id) = registration.type_path.plugin_id() else {
        return Err(invalid_vm_registration(
            type_path,
            "VM types must declare a plugin id",
        ));
    };
    let plugin_prefix = format!("{plugin_id}.");
    if !type_path.starts_with(&plugin_prefix) {
        return Err(invalid_vm_registration(
            type_path,
            &format!("VM full type path must begin with `{plugin_prefix}`"),
        ));
    }
    for field in &registration.type_info.fields {
        if let Err(reason) = DeclaredValueType::parse_vm(&field.value_type_path) {
            return Err(invalid_vm_registration(
                type_path,
                &format!("reflected field `{}` {reason}", field.name),
            ));
        }
    }

    match backing {
        VmTypeBacking::DynamicComponent if !registration.is_component() => {
            Err(invalid_vm_registration(
                type_path,
                "dynamic VM backing requires a component-only registration",
            ))
        }
        VmTypeBacking::DynamicComponent => Ok(plugin_id),
    }
}

/// 校验 Runtime 注册的字段元数据和默认值语义。
///
/// # 参数
///
/// `registration` 是尚未接纳的只读元数据。
///
/// # 返回值
///
/// 身份规范、字段预算、展示信息、类型/hint、默认值和 enum/numeric 元数据兼容时返回成功。
///
/// # 约束
///
/// 中立 catalog 身份检查与 Runtime 值语义分开执行。默认值先通过值树预算，再匹配声明类型；Named 默认值通过特定 editor hint 与值变体匹配。本函数不执行适配器，也不保证其未来读写结果满足字段类型。
///
/// # 参考
///
/// `ReflectSchemaCatalog::validate_entry`、`validate_reflected_value_contract`、`DeclaredValueType`。
fn validate_registration(registration: &ReflectTypeRegistration) -> Result<(), ReflectError> {
    let type_path = registration.type_path.type_path();
    if registration.type_info.fields.len() > MAX_REFLECT_FIELDS_PER_TYPE {
        return Err(ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: format!(
                "reflected types must not declare more than {MAX_REFLECT_FIELDS_PER_TYPE} fields"
            ),
        });
    }

    ReflectSchemaCatalog::validate_entry(&ReflectSchemaCatalogEntry::new(registration.clone()))?;

    let mut enum_option_count = 0_usize;
    for field in &registration.type_info.fields {
        validate_field_text(
            type_path,
            field,
            "display name",
            &field.display_name,
            MAX_REFLECT_FIELD_DISPLAY_NAME_BYTES,
        )?;
        let declared = DeclaredValueType::parse(&field.value_type_path)
            .map_err(|reason| invalid_field_registration(type_path, &field.name, &reason))?;
        validate_editor_hint(type_path, field, &declared)?;
        if let Some(default_value) = &field.default_value {
            validate_reflected_value_contract(default_value).map_err(|error| {
                invalid_field_registration(
                    type_path,
                    &field.name,
                    &format!("default value rejected: {error}"),
                )
            })?;
            if !declared.matches_reflected(default_value)
                && !(declared.is_named()
                    && editor_hint_matches_value(&field.editor_hint, default_value))
            {
                return Err(invalid_field_registration(
                    type_path,
                    &field.name,
                    &format!(
                        "default value type `{}` does not match declared value type `{}`",
                        default_value.type_name(),
                        field.value_type_path
                    ),
                ));
            }
        }
        validate_numeric_metadata(type_path, field, &declared)?;
        enum_option_count = enum_option_count
            .checked_add(field.enum_options.len())
            .ok_or_else(|| ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason: "reflected enum option count overflowed".to_string(),
            })?;
        if enum_option_count > MAX_REFLECT_ENUM_OPTIONS_PER_TYPE {
            return Err(ReflectError::InvalidRegistration {
                type_path: type_path.to_string(),
                reason: format!(
                    "reflected types must not declare more than {MAX_REFLECT_ENUM_OPTIONS_PER_TYPE} total enum options"
                ),
            });
        }
        validate_enum_metadata(type_path, field, &declared)?;
    }
    Ok(())
}

fn validate_field_text(
    type_path: &str,
    field: &ReflectFieldInfo,
    label: &str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must be non-empty and already trimmed"),
        ));
    }
    if value.len() > max_bytes {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("field {label} must not exceed {max_bytes} bytes"),
        ));
    }
    Ok(())
}

/// 检查字段展示提示与声明值类型的兼容性。
///
/// # 参数
///
/// `type_path` 提供错误上下文，`field` 提供提示与字段身份，`declared` 是已解析的值类型。
///
/// # 返回值
///
/// 兼容时成功，否则返回 `InvalidFieldRegistration`。
///
/// # 约束
///
/// Named 类型允许展示提示补充其表现形式；这只是元数据准入，读写值的具体类型仍由转换适配器处理。
///
/// # 参考
///
/// `validate_registration`、`editor_hint_matches_value`。
fn validate_editor_hint(
    type_path: &str,
    field: &ReflectFieldInfo,
    declared: &DeclaredValueType,
) -> Result<(), ReflectError> {
    let compatible = match &field.editor_hint {
        ReflectEditorHint::None | ReflectEditorHint::Json => true,
        ReflectEditorHint::String | ReflectEditorHint::MultilineString => {
            matches!(declared, DeclaredValueType::String) || declared.is_named()
        }
        ReflectEditorHint::Bool => {
            matches!(declared, DeclaredValueType::Bool) || declared.is_named()
        }
        ReflectEditorHint::Integer => {
            matches!(declared, DeclaredValueType::Integer) || declared.is_named()
        }
        ReflectEditorHint::Unsigned => {
            matches!(declared, DeclaredValueType::Unsigned) || declared.is_named()
        }
        ReflectEditorHint::Scalar => {
            matches!(declared, DeclaredValueType::Scalar) || declared.is_named()
        }
        ReflectEditorHint::Vec2 => {
            matches!(declared, DeclaredValueType::Vec2) || declared.is_named()
        }
        ReflectEditorHint::Vec3 => {
            matches!(declared, DeclaredValueType::Vec3) || declared.is_named()
        }
        ReflectEditorHint::Vec4 => {
            matches!(
                declared,
                DeclaredValueType::Vec4 | DeclaredValueType::Quaternion
            ) || declared.is_named()
        }
        ReflectEditorHint::Enum => declared.supports_enum_metadata() || declared.is_named(),
        ReflectEditorHint::Entity => {
            matches!(declared, DeclaredValueType::Entity) || declared.is_named()
        }
        ReflectEditorHint::Resource => {
            matches!(declared, DeclaredValueType::Resource) || declared.is_named()
        }
        ReflectEditorHint::Color => {
            matches!(declared, DeclaredValueType::Vec3 | DeclaredValueType::Vec4)
                || declared.is_named()
        }
    };
    if compatible {
        return Ok(());
    }
    Err(invalid_field_registration(
        type_path,
        &field.name,
        &format!(
            "editor hint `{:?}` is incompatible with declared value type `{}`",
            field.editor_hint, field.value_type_path
        ),
    ))
}

/// 检查数值范围元数据是否用于可表示数值的字段。
///
/// # 参数
///
/// `field` 提供可选范围，`declared` 是字段已解析的类型；`type_path` 供错误定位。
///
/// # 返回值
///
/// 未设范围、数值字段或 Named 配合数值提示时成功；不兼容时返回字段错误。
///
/// # 约束
///
/// 范围对象的有限值、顺序和步长由 `ReflectNumericRange` 构造及反序列化检查；此函数负责字段兼容性。
///
/// # 参考
///
/// `ReflectNumericRange::new`、`validate_registration`。
fn validate_numeric_metadata(
    type_path: &str,
    field: &ReflectFieldInfo,
    declared: &DeclaredValueType,
) -> Result<(), ReflectError> {
    if field.numeric_range.is_none()
        || declared.supports_numeric_metadata()
        || (declared.is_named()
            && matches!(
                &field.editor_hint,
                ReflectEditorHint::Integer
                    | ReflectEditorHint::Unsigned
                    | ReflectEditorHint::Scalar
            ))
    {
        return Ok(());
    }
    Err(invalid_field_registration(
        type_path,
        &field.name,
        "numeric range metadata requires an integer, unsigned, or scalar field",
    ))
}

/// 检查枚举选项的预算、值身份和默认成员关系。
///
/// # 参数
///
/// `field` 提供 enum options 与默认值，`declared` 是字段类型；`type_path` 供错误定位。
///
/// # 返回值
///
/// 选项为空或全部规则通过时成功，失败返回字段错误。
///
/// # 约束
///
/// 非空选项要求枚举类型或 Named 配合 Enum 提示；值须满足长度、ASCII token 与唯一性规则。默认值为 Enum 时必须属于选项集合，全类型选项预算由外层累计。
///
/// # 参考
///
/// `validate_registration`、`valid_enum_value`。
fn validate_enum_metadata(
    type_path: &str,
    field: &ReflectFieldInfo,
    declared: &DeclaredValueType,
) -> Result<(), ReflectError> {
    if field.enum_options.len() > MAX_REFLECT_ENUM_OPTIONS_PER_FIELD {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!(
                "enum fields must not declare more than {MAX_REFLECT_ENUM_OPTIONS_PER_FIELD} options"
            ),
        ));
    }
    if field.enum_options.is_empty() {
        return Ok(());
    }
    if !declared.supports_enum_metadata()
        && !(declared.is_named() && field.editor_hint == ReflectEditorHint::Enum)
    {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            "enum options require an enum field",
        ));
    }

    let mut values = HashSet::with_capacity(field.enum_options.len());
    for option in &field.enum_options {
        validate_enum_option_text(
            type_path,
            field,
            "value",
            &option.value,
            MAX_REFLECT_ENUM_VALUE_BYTES,
        )?;
        if !valid_enum_value(&option.value) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                "enum option values must use ASCII letters, digits, `_`, `-`, `.`, or `:`",
            ));
        }
        validate_enum_option_text(
            type_path,
            field,
            "display name",
            &option.display_name,
            MAX_REFLECT_ENUM_DISPLAY_NAME_BYTES,
        )?;
        if !values.insert(option.value.as_str()) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("duplicate enum option value `{}`", option.value),
            ));
        }
    }
    if let Some(ReflectedValue::Enum(default)) = &field.default_value {
        if !values.contains(default.as_str()) {
            return Err(invalid_field_registration(
                type_path,
                &field.name,
                &format!("enum default `{default}` is not present in enum options"),
            ));
        }
    }
    Ok(())
}

fn validate_enum_option_text(
    type_path: &str,
    field: &ReflectFieldInfo,
    label: &str,
    value: &str,
    max_bytes: usize,
) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("enum option {label} must be non-empty and already trimmed"),
        ));
    }
    if value.len() > max_bytes {
        return Err(invalid_field_registration(
            type_path,
            &field.name,
            &format!("enum option {label} must not exceed {max_bytes} bytes"),
        ));
    }
    Ok(())
}

/// 为 Named 字段的默认值提供展示提示到值变体的匹配。
///
/// # 参数
///
/// `hint` 是字段展示形式，`value` 是默认反射值的借用。
///
/// # 返回值
///
/// 指定提示与值变体相符且适用浮点分量有限时返回 `true`。
///
/// # 约束
///
/// 仅在 `validate_registration` 的 Named 默认值分支使用；不替代运行时写入的字段转换或值树预算。
///
/// # 参考
///
/// `validate_registration`、`ensure_reflected_value_type`。
fn editor_hint_matches_value(hint: &ReflectEditorHint, value: &ReflectedValue) -> bool {
    match (hint, value) {
        (
            ReflectEditorHint::String | ReflectEditorHint::MultilineString,
            ReflectedValue::String(_),
        )
        | (ReflectEditorHint::Bool, ReflectedValue::Bool(_))
        | (ReflectEditorHint::Integer, ReflectedValue::Integer(_))
        | (ReflectEditorHint::Unsigned, ReflectedValue::Unsigned(_))
        | (ReflectEditorHint::Enum, ReflectedValue::Enum(_))
        | (ReflectEditorHint::Resource, ReflectedValue::Resource(_))
        | (ReflectEditorHint::Json, ReflectedValue::Json(_)) => true,
        (ReflectEditorHint::Scalar, ReflectedValue::Scalar(value)) => value.is_finite(),
        (ReflectEditorHint::Vec2, ReflectedValue::Vec2(values)) => finite(values),
        (ReflectEditorHint::Vec3 | ReflectEditorHint::Color, ReflectedValue::Vec3(values)) => {
            finite(values)
        }
        (
            ReflectEditorHint::Vec4 | ReflectEditorHint::Color,
            ReflectedValue::Vec4(values) | ReflectedValue::Quaternion(values),
        ) => finite(values),
        (ReflectEditorHint::Entity, ReflectedValue::Entity(_) | ReflectedValue::Null) => true,
        _ => false,
    }
}

fn finite(values: &[f32]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn valid_enum_value(value: &str) -> bool {
    value.bytes().all(|byte| {
        byte == b'_' || byte == b'-' || byte == b'.' || byte == b':' || byte.is_ascii_alphanumeric()
    })
}

/// 检查反射值是否符合已声明的具体值类型。
///
/// # 参数
///
/// `expected` 是待解析的声明类型路径，`value` 是共享借用的反射值；`type_path` 与 `field_name` 用于错误上下文。
///
/// # 返回值
///
/// 解析与变体匹配成功时返回成功；否则返回拥有上下文的 `TypeMismatch`。
///
/// # 约束
///
/// 列表/映射按声明递归匹配，适用浮点值须有限；此检查与 Runtime 值树资源预算各自负责不同条件。
///
/// # 参考
///
/// `DeclaredValueType::matches_reflected`；动态组件 VM 写入路径；`validate_reflected_value`。
pub(crate) fn ensure_reflected_value_type(
    type_path: &str,
    field_name: &str,
    expected: &str,
    value: &ReflectedValue,
) -> Result<(), ReflectError> {
    if reflected_value_matches_type_path(expected, value) {
        return Ok(());
    }
    Err(ReflectError::TypeMismatch {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        expected: expected.to_string(),
        actual: value.type_name().to_string(),
    })
}

fn reflected_value_matches_type_path(expected: &str, value: &ReflectedValue) -> bool {
    DeclaredValueType::parse(expected)
        .map(|declared| declared.matches_reflected(value))
        .unwrap_or(false)
}

fn validate_canonical_text(type_path: &str, label: &str, value: &str) -> Result<(), ReflectError> {
    if value.is_empty() || value.trim() != value {
        return Err(invalid_vm_registration(
            type_path,
            &format!("VM {label} must be non-empty and already trimmed"),
        ));
    }
    Ok(())
}

fn invalid_vm_registration(type_path: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidRegistration {
        type_path: type_path.to_string(),
        reason: reason.to_string(),
    }
}

fn invalid_field_registration(type_path: &str, field_name: &str, reason: &str) -> ReflectError {
    ReflectError::InvalidFieldRegistration {
        type_path: type_path.to_string(),
        field_name: field_name.to_string(),
        reason: reason.to_string(),
    }
}

impl fmt::Debug for TypeRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TypeRegistry")
            .field("registrations", &self.registrations)
            .field("schema_catalog", &self.schema_catalog)
            .field("schema_catalog_generation", &self.schema_catalog_generation)
            .finish()
    }
}

/// 比较注册内容及中立目录内容。
///
/// # 约束
///
/// 比较不包含目录版本。Runtime 注册中的适配器只比较存在性，不比较函数地址；因此相等不能单独说明访问计划版本或回调身份相同。
///
/// # 参考
///
/// `RuntimeTypeRegistration::eq`、`schema_catalog_generation`。
impl PartialEq for TypeRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.registrations == other.registrations && self.schema_catalog == other.schema_catalog
    }
}
