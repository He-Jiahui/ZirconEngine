use crate::scene::{
    reflect::{ReflectComponent, ReflectResource, TypeRegistry},
    World,
};
use zircon_runtime_interface::reflect::{
    ReflectError, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue, ReflectFieldsRequest,
    ReflectFieldsResponse, ReflectObjectAddress, ReflectReadRequest, ReflectReadResponse,
    ReflectSchemaFilter, ReflectSchemaRequest, ReflectSchemaResponse, ReflectTypeRegistration,
    ReflectWriteRequest, ReflectWriteResponse, ReflectedValue,
};

use super::validate_reflected_value;

/// 编辑器、脚本和动态场景共享的反射门面；地址与字段 ID 先由模式目录解析，再由 World 适配器访问实时数据。
/// 将反射请求路由到当前 World 的模式目录与实时数据适配器。
///
/// # 状态与生命周期
///
/// 门面本身不持有状态。类型与稳定字段身份来自 World 自有注册表；读取期间借用 World，响应中的元数据和字段值拥有自身数据。
///
/// # 约束
///
/// schema 可见性标记用于类型列举；具体字段访问按地址、身份、角色、可编辑性及适配器规则执行。
///
/// # 参考
///
/// `TypeRegistry`、`World::type_registry`、`Self::reflect_read`、`Self::reflect_write`。
pub struct WorldReflection;

impl WorldReflection {
    /// 列举当前目录中符合请求筛选的反射元数据。
    ///
    /// # 参数
    ///
    /// `world` 提供目录的共享借用；`request` 消费筛选条件，可指定完整路径或唯一短路径。
    ///
    /// # 返回值
    ///
    /// 返回拥有注册元数据的响应，以及整个当前 schema catalog 的 fingerprint；该 fingerprint 不随本次筛选缩小。指定的类型未知或短名歧义时返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 筛选只检查元数据，不访问组件或资源实例。返回的注册副本不借用 World；是否包含插件拥有的类型由筛选条件决定。
    ///
    /// # 参考
    ///
    /// `schema_filter_matches`、`ReflectSchemaCatalog::fingerprint`、`Self::reflect_schema`。
    pub fn list_reflect_types(
        world: &World,
        request: ReflectSchemaRequest,
    ) -> Result<ReflectSchemaResponse, ReflectError> {
        let filter = request.filter;
        let schema_catalog = world.type_registry().schema_catalog();
        let registrations = if let Some(type_path) = filter.type_path.as_deref() {
            let registration = schema_catalog.registration(type_path)?;
            let mut registrations = Vec::with_capacity(1);
            if schema_filter_matches(registration, &filter) {
                registrations.push(registration.clone());
            }
            registrations
        } else {
            let registry_entries = schema_catalog.registrations();
            let mut registrations = Vec::with_capacity(registry_entries.size_hint().0);
            for registration in registry_entries {
                if schema_filter_matches(registration, &filter) {
                    registrations.push(registration.clone());
                }
            }
            registrations
        };

        Ok(ReflectSchemaResponse::new(
            schema_catalog.fingerprint(),
            registrations,
        ))
    }

    /// 取得一个已注册类型的规范化反射元数据副本。
    ///
    /// # 参数
    ///
    /// `world` 提供目录；`type_path` 是完整路径或唯一短路径。
    ///
    /// # 返回值
    ///
    /// 返回拥有自身数据的注册；未知类型或歧义短名返回目录错误。
    ///
    /// # 约束与生命周期
    ///
    /// 不应用类型列举的可见性筛选，也不要求适配器或实例存在。副本可在本次 World 借用结束后使用。
    ///
    /// # 参考
    ///
    /// `TypeRegistry::registration`、`Self::list_reflect_types`。
    pub fn reflect_schema(
        world: &World,
        type_path: &str,
    ) -> Result<ReflectTypeRegistration, ReflectError> {
        Ok(world.type_registry().registration(type_path)?.clone())
    }

    /// 按当前注册字段顺序读取组件或资源的全部反射字段。
    ///
    /// # 参数
    ///
    /// `world` 是共享借用；`request` 消费待访问的组件或资源地址。
    ///
    /// # 返回值
    ///
    /// 返回原请求地址和拥有稳定字段 ID、规范名、值的响应；地址角色、适配器、实例或某字段读取/预算无效时返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 读取按当前 schema 槽位进行，每个值通过预算后才进入响应；失败不返回部分字段列表。此处读取全部 schema 字段，动态场景的可序列化字段筛选由调用者另行执行。
    ///
    /// # 参考
    ///
    /// `Self::read_component_fields_by_slot`、`Self::read_resource_fields_by_slot`、`read_schema_fields_by_slot`。
    pub fn reflect_fields(
        world: &World,
        request: ReflectFieldsRequest,
    ) -> Result<ReflectFieldsResponse, ReflectError> {
        let fields = match &request.address {
            ReflectObjectAddress::Component { entity, type_path } => {
                let registration = world.type_registry().registration(type_path)?;
                let adapter = component_adapter(world, type_path)?;
                Self::read_component_fields_by_slot(world, *entity, registration, adapter)?
            }
            ReflectObjectAddress::Resource { type_path } => {
                let registration = world.type_registry().registration(type_path)?;
                let adapter = resource_adapter_ref(world, type_path)?;
                Self::read_resource_fields_by_slot(world, registration, adapter)?
            }
        };

        Ok(ReflectFieldsResponse::new(request.address, fields))
    }

    /// 按稳定字段 ID 读取一个组件或资源字段。
    ///
    /// # 参数
    ///
    /// `world` 是共享借用；`request` 消费对象地址和稳定字段 ID。
    ///
    /// # 返回值
    ///
    /// 返回保留请求地址、使用字段规范名的拥有型响应；目录/槽位投影、角色、适配器、读取或值预算失败时返回错误。
    ///
    /// # 约束与生命周期
    ///
    /// 先从当前目录解析并核对槽位，再由适配器生成值，最后进行值树预算检查。字段值的具体 Rust/动态类型转换属于适配器；响应不持有 World 借用。
    ///
    /// # 参考
    ///
    /// `field_access`、`read_reflected_field_by_slot`、`validate_reflected_value`。
    pub fn reflect_read(
        world: &World,
        request: ReflectReadRequest,
    ) -> Result<ReflectReadResponse, ReflectError> {
        let type_path = request.address.type_path();
        let (field_slot, field_name) = field_access(world, type_path, request.field_id, false)?;
        let value = read_reflected_field_by_slot(world, &request.address, field_slot)?;
        validate_reflected_value(type_path, &field_name, &value)?;
        let field = ReflectFieldValue::new(request.field_id, field_name, value);

        Ok(ReflectReadResponse::new(request.address, field))
    }

    /// 接纳一个字段写入请求，并由当前 World 适配器执行写入。
    ///
    /// # 参数
    ///
    /// `world` 为独占可变借用；`request` 拥有目标地址、稳定字段 ID 和待消费的值。
    ///
    /// # 返回值
    ///
    /// 成功响应中的 `field` 确认已接纳的请求值，`changed` 原样采用适配器返回的变更标记。要观察当前存储值，另行调用 `reflect_read`。
    ///
    /// # 约束与生命周期
    ///
    /// 先取得适配器表，再检查字段身份、可编辑性和值树预算，之后才进入写回调。适配器表 clone/copy 用于结束目录共享借用，不复制 World 数据。本门面在回调之后不做通用回读或回滚；具体类型转换、领域验证及提交策略由对应适配器/World writer 承担。
    ///
    /// # 错误
    ///
    /// 门面准入失败时尚未调用写回调。回调错误原样传播，其对数据提交的约定属于适配器。
    ///
    /// # 参考
    ///
    /// `field_access`、`validate_reflected_value`、`Self::reflect_read`；派生组件适配器和动态组件写入实现。
    pub fn reflect_write(
        world: &mut World,
        request: ReflectWriteRequest,
    ) -> Result<ReflectWriteResponse, ReflectError> {
        let (changed, accepted_field) = match &request.address {
            ReflectObjectAddress::Component { entity, type_path } => {
                let adapter = component_adapter_for_write(world, type_path)?;
                let (field_slot, field_name) =
                    field_access(world, type_path, request.field_id, true)?;
                validate_reflected_value(type_path, &field_name, &request.value)?;
                // The response acknowledges the admitted request; observation remains an explicit read.
                let accepted_field =
                    ReflectFieldValue::new(request.field_id, field_name, request.value.clone());
                let changed =
                    adapter.write_field_by_slot(world, *entity, field_slot, request.value)?;
                (changed, accepted_field)
            }
            ReflectObjectAddress::Resource { type_path } => {
                let adapter = resource_adapter_for_write(world, type_path)?;
                let (field_slot, field_name) =
                    field_access(world, type_path, request.field_id, true)?;
                validate_reflected_value(type_path, &field_name, &request.value)?;
                let accepted_field =
                    ReflectFieldValue::new(request.field_id, field_name, request.value.clone());
                let changed = adapter.write_field_by_slot(world, field_slot, request.value)?;
                (changed, accepted_field)
            }
        };

        Ok(ReflectWriteResponse::new(
            request.address,
            accepted_field,
            changed,
        ))
    }
}

impl WorldReflection {
    /// 按当前组件 schema 的槽位读取全部字段。
    ///
    /// # 参数
    ///
    /// `world` 和 `adapter` 为共享借用；`entity` 标识目标实体；`registration` 提供有序字段声明。
    ///
    /// # 返回值
    ///
    /// 返回拥有型字段列表；`contains` 为假时返回 `MissingComponent`，字段读取或预算失败时返回相应错误。
    ///
    /// # 约束与生命周期
    ///
    /// 调用者须将同一当前注册及其适配器配对，槽位依据其字段原序。借用仅持续本次读取。
    ///
    /// # 参考
    ///
    /// `read_schema_fields_by_slot`；动态场景捕获的组件路径。
    pub(crate) fn read_component_fields_by_slot(
        world: &World,
        entity: crate::scene::EntityId,
        registration: &ReflectTypeRegistration,
        adapter: &ReflectComponent,
    ) -> Result<Vec<ReflectFieldValue>, ReflectError> {
        let type_path = registration.type_path.type_path();
        if !adapter.contains(world, entity) {
            return Err(ReflectError::MissingComponent {
                entity,
                type_path: type_path.to_string(),
            });
        }
        read_schema_fields_by_slot(type_path, &registration.type_info.fields, |slot| {
            adapter.read_field_by_slot(world, entity, slot)
        })
    }

    /// 按当前资源 schema 的槽位读取全部字段。
    ///
    /// # 参数
    ///
    /// `world` 和 `adapter` 为共享借用，`registration` 提供对应资源字段序列。
    ///
    /// # 返回值
    ///
    /// 返回拥有型字段列表；资源不存在时返回 `MissingResource`，字段读取或预算失败时返回相应错误。
    ///
    /// # 约束与生命周期
    ///
    /// 注册字段顺序与适配器槽位必须对应；不携带实体身份，返回值不借用资源实例。
    ///
    /// # 参考
    ///
    /// `read_schema_fields_by_slot`；动态场景捕获的资源路径。
    pub(crate) fn read_resource_fields_by_slot(
        world: &World,
        registration: &ReflectTypeRegistration,
        adapter: &ReflectResource,
    ) -> Result<Vec<ReflectFieldValue>, ReflectError> {
        let type_path = registration.type_path.type_path();
        if !adapter.contains(world) {
            return Err(ReflectError::MissingResource {
                type_path: type_path.to_string(),
            });
        }
        read_schema_fields_by_slot(type_path, &registration.type_info.fields, |slot| {
            adapter.read_field_by_slot(world, slot)
        })
    }
}

/// 以字段声明原序构造经预算验证的拥有型字段列表。
///
/// # 参数
///
/// `type_path` 用于错误定位，`fields` 提供稳定身份及规范名，`read` 按当前零基槽位生成拥有型值。
///
/// # 返回值
///
/// 全部字段成功后返回预分配列表；槽位超出 `u32`、读取或值验证失败时停止并丢弃局部结果。
///
/// # 约束与生命周期
///
/// 回调只在本函数内同步调用；每个值独立应用 Runtime 预算，不累计整个字段列表的节点或文本字节。
///
/// # 参考
///
/// `validate_reflected_value`、`ReflectFieldValue::new`。
fn read_schema_fields_by_slot(
    type_path: &str,
    fields: &[ReflectFieldInfo],
    mut read: impl FnMut(u32) -> Result<ReflectedValue, ReflectError>,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    let mut values = Vec::with_capacity(fields.len());
    for (slot, field) in fields.iter().enumerate() {
        let slot = u32::try_from(slot).map_err(|_| ReflectError::InvalidRegistration {
            type_path: type_path.to_string(),
            reason: "reflection schema has more than u32::MAX fields".to_string(),
        })?;
        let value = read(slot)?;
        validate_reflected_value(type_path, &field.name, &value)?;
        values.push(ReflectFieldValue::new(field.id, field.name.clone(), value));
    }
    Ok(values)
}

impl World {
    /// 借用此 World 自有的反射注册表。
    ///
    /// # 返回值
    ///
    /// 返回 `TypeRegistry` 的只读借用，有效期依附此 World。
    ///
    /// # 约束与生命周期
    ///
    /// 查询不注册类型，也不允许通过此借用修改目录；注册变更由 World 或注册表对应接纳入口执行。
    ///
    /// # 参考
    ///
    /// `TypeRegistry`、`WorldReflection`。
    pub fn type_registry(&self) -> &TypeRegistry {
        self.type_registry_for_reflection()
    }

    /// 按请求筛选列举此 World 的反射类型。
    ///
    /// # 参数
    ///
    /// `request` 拥有本次 schema 筛选条件。
    ///
    /// # 返回值
    ///
    /// 返回类型副本及整个目录 fingerprint，或目录查询错误。
    ///
    /// # 参考
    ///
    /// 筛选和返回拥有权约定见 `WorldReflection::list_reflect_types`。
    pub fn list_reflect_types(
        &self,
        request: ReflectSchemaRequest,
    ) -> Result<ReflectSchemaResponse, ReflectError> {
        WorldReflection::list_reflect_types(self, request)
    }

    /// 查询此 World 中指定类型的反射元数据副本。
    ///
    /// # 参数
    ///
    /// `type_path` 是完整或唯一短路径。
    ///
    /// # 返回值
    ///
    /// 返回拥有型注册或目录错误。
    ///
    /// # 参考
    ///
    /// `WorldReflection::reflect_schema`。
    pub fn reflect_schema(&self, type_path: &str) -> Result<ReflectTypeRegistration, ReflectError> {
        WorldReflection::reflect_schema(self, type_path)
    }

    /// 通过门面读取目标地址的全部当前反射字段。
    ///
    /// # 参数
    ///
    /// `request` 拥有组件或资源地址。
    ///
    /// # 返回值
    ///
    /// 返回拥有型字段列表响应或访问/预算错误。
    ///
    /// # 参考
    ///
    /// 槽位与预算合同见 `WorldReflection::reflect_fields`。
    pub fn reflect_fields(
        &self,
        request: ReflectFieldsRequest,
    ) -> Result<ReflectFieldsResponse, ReflectError> {
        WorldReflection::reflect_fields(self, request)
    }

    /// 通过稳定字段 ID 读取此 World 的一个反射字段。
    ///
    /// # 参数
    ///
    /// `request` 拥有对象地址与字段身份。
    ///
    /// # 返回值
    ///
    /// 返回拥有型字段响应或访问/预算错误。
    ///
    /// # 参考
    ///
    /// `WorldReflection::reflect_read`；写入请求确认与观察值的区别见 `WorldReflection::reflect_write`。
    pub fn reflect_read(
        &self,
        request: ReflectReadRequest,
    ) -> Result<ReflectReadResponse, ReflectError> {
        WorldReflection::reflect_read(self, request)
    }

    /// 通过门面接纳并执行此 World 的单字段写入。
    ///
    /// # 参数
    ///
    /// `request` 拥有地址、稳定字段 ID 和待写入值。
    ///
    /// # 返回值
    ///
    /// 返回已接纳请求字段及适配器变更标记，或准入/写入错误。
    ///
    /// # 参考
    ///
    /// 具体提交和错误边界见 `WorldReflection::reflect_write`；观察当前数据用 `World::reflect_read`。
    pub fn reflect_write(
        &mut self,
        request: ReflectWriteRequest,
    ) -> Result<ReflectWriteResponse, ReflectError> {
        WorldReflection::reflect_write(self, request)
    }

    #[cfg(test)]
    pub(crate) fn type_registry_mut_for_tests(&mut self) -> &mut TypeRegistry {
        self.type_registry_mut_for_reflection()
    }
}

/// 判断一条注册是否满足 schema 列举筛选。
///
/// # 参数
///
/// `registration` 为只读元数据，`filter` 提供类别、可见性和插件拥有项选择。
///
/// # 返回值
///
/// 各启用条件都满足时返回 `true`。两个类别标记均关闭时，不按组件/资源类别过滤。
///
/// # 约束
///
/// 此谓词只控制类型列举；单字段读写不会将它自动用作访问权限准入。
///
/// # 参考
///
/// `WorldReflection::list_reflect_types`。
fn schema_filter_matches(
    registration: &ReflectTypeRegistration,
    filter: &ReflectSchemaFilter,
) -> bool {
    let category_filter_active = filter.include_components || filter.include_resources;
    if category_filter_active
        && !((filter.include_components && registration.is_component())
            || (filter.include_resources && registration.is_resource()))
    {
        return false;
    }

    if filter.editor_visible && !registration.editor_visible {
        return false;
    }

    if filter.remote_visible && !registration.remote_visible {
        return false;
    }

    if !filter.include_plugin_owned && registration.is_plugin_owned() {
        return false;
    }

    true
}

fn component_adapter<'a>(
    world: &'a World,
    type_path: &str,
) -> Result<&'a ReflectComponent, ReflectError> {
    let registration = world.type_registry().runtime_registration(type_path)?;
    if !registration.registration.is_component() {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!(
                "component `{}`",
                registration.registration.type_path.type_path()
            ),
            actual: format!(
                "non-component `{}`",
                registration.registration.type_path.type_path()
            ),
        });
    }

    let Some(adapter) = registration.component.as_ref() else {
        return Err(ReflectError::NoComponentAdapter {
            type_path: registration.registration.type_path.type_path().to_string(),
        });
    };

    Ok(adapter)
}

/// 为可变 World 写入取得不借用目录的组件适配器表。
///
/// # 参数
///
/// `world` 提供当前注册，`type_path` 是查询路径。
///
/// # 返回值
///
/// 返回克隆的函数表及其拥有型路径，或角色/适配器错误。
///
/// # 约束与生命周期
///
/// 克隆不复制 World 或组件实例；结束表的共享借用后，调用者可将 World 独占借给写回调。
///
/// # 参考
///
/// `component_adapter`、`WorldReflection::reflect_write`。
fn component_adapter_for_write(
    world: &World,
    type_path: &str,
) -> Result<ReflectComponent, ReflectError> {
    component_adapter(world, type_path).cloned()
}

fn resource_adapter_ref<'a>(
    world: &'a World,
    type_path: &str,
) -> Result<&'a ReflectResource, ReflectError> {
    let registration = world.type_registry().runtime_registration(type_path)?;
    if !registration.registration.is_resource() {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!(
                "resource `{}`",
                registration.registration.type_path.type_path()
            ),
            actual: format!(
                "non-resource `{}`",
                registration.registration.type_path.type_path()
            ),
        });
    }

    let Some(adapter) = registration.resource.as_ref() else {
        return Err(ReflectError::NoResourceAdapter {
            type_path: registration.registration.type_path.type_path().to_string(),
        });
    };

    Ok(adapter)
}

/// 为可变 World 写入取得不借用目录的资源适配器表。
///
/// # 参数
///
/// `world` 提供当前注册，`type_path` 是查询路径。
///
/// # 返回值
///
/// 返回复制的函数表，或角色/适配器错误。
///
/// # 约束与生命周期
///
/// 复制不复制资源实例；结束表的共享借用后，调用者可将 World 独占借给资源写回调。
///
/// # 参考
///
/// `resource_adapter_ref`、`WorldReflection::reflect_write`。
fn resource_adapter_for_write(
    world: &World,
    type_path: &str,
) -> Result<ReflectResource, ReflectError> {
    resource_adapter_ref(world, type_path).copied()
}

/// 将稳定字段身份接入当前 Runtime 槽位，并检查写入可编辑性。
///
/// # 参数
///
/// `world` 提供当前目录和 Runtime 投影，`type_path` 标识类型，`field_id` 是稳定身份；`require_editable` 为写入开启字段可编辑检查。
///
/// # 返回值
///
/// 返回当期零基槽位和拥有型字段规范名；身份未知、投影不一致或不可编辑时返回错误。
///
/// # 约束与生命周期
///
/// 目录解析后再次核对 Runtime 槽位处字段 ID。返回槽位只对应当前注册顺序，持久计划须绑定目录版本；此函数不读取或修改实例数据。
///
/// # 参考
///
/// `TypeRegistry::resolve_field_slot_by_id`、`WorldReflection::reflect_read`、`WorldReflection::reflect_write`。
fn field_access(
    world: &World,
    type_path: &str,
    field_id: ReflectFieldId,
    require_editable: bool,
) -> Result<(u32, String), ReflectError> {
    let registry = world.type_registry();
    let registration = registry.runtime_registration(type_path)?;
    let field_slot = registry.resolve_field_slot_by_id(type_path, field_id)?;
    let Some(field) = registration
        .registration
        .type_info
        .fields
        .get(field_slot as usize)
        .filter(|field| field.id == field_id)
    else {
        return Err(ReflectError::InvalidRegistration {
            type_path: registration.registration.type_path.type_path().to_string(),
            reason: format!(
                "catalog slot {field_slot} does not project stable field ID `{field_id}`"
            ),
        });
    };
    if require_editable && !field.editable {
        return Err(ReflectError::NonEditableField {
            type_path: registration.registration.type_path.type_path().to_string(),
            field_name: field.name.clone(),
        });
    }
    Ok((field_slot, field.name.clone()))
}

fn read_reflected_field_by_slot(
    world: &World,
    address: &ReflectObjectAddress,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    match address {
        ReflectObjectAddress::Component { entity, type_path } => {
            let adapter = component_adapter(world, type_path)?;
            adapter.read_field_by_slot(world, *entity, field_slot)
        }
        ReflectObjectAddress::Resource { type_path } => {
            let adapter = resource_adapter_ref(world, type_path)?;
            adapter.read_field_by_slot(world, field_slot)
        }
    }
}
