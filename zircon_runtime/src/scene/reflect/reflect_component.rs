//! 组件反射的同步 Rust 回调表，供 World 门面和动态场景预检调用。
//! 表持有自己的类型路径，World 存储由调用方按本次操作借用；字段准入和提交策略由门面及具体适配器负责。

/// 按当前注册字段顺序读取一个组件字段的回调。
///
/// 参数依次为共享 World、稳定实体 ID、完整类型路径和零基槽位；成功结果拥有字段值。
/// 槽位须与本次 schema 配对，持久引用使用稳定字段 ID；参见 `TypeRegistry::resolve_field_slot_by_id`。
pub type ReflectComponentReadFieldBySlot = fn(
    &crate::scene::World,
    crate::scene::EntityId,
    &str,
    u32,
) -> Result<
    zircon_runtime_interface::reflect::ReflectedValue,
    zircon_runtime_interface::reflect::ReflectError,
>;

/// 按当前槽位消费一个字段值的组件写回调。
///
/// 参数依次为独占 World、稳定实体 ID、完整类型路径、槽位和拥有型值；返回值作为变更标记传给调用者。
/// 具体适配器负责值转换和 World 提交；参见 `WorldReflection::reflect_write`。
pub type ReflectComponentWriteFieldBySlot =
    fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
        u32,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

/// 消费有序槽位/值列表的组件批量写回调。
///
/// World 独占借用和实体/路径定位仅持续本次调用；列表的所有权交给回调。
/// 返回整批的变更标记；输入次序、重复槽位和失败时的提交边界由具体适配器规定。
pub type ReflectComponentWriteFieldsBySlot =
    fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
        Vec<(u32, zircon_runtime_interface::reflect::ReflectedValue)>,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

/// 把一个组件的克隆值暂存到独立预检 World 的回调。
///
/// 参数依次为共享源 World、稳定实体 ID、完整类型路径和独占目标 World；成功返回 `()`。
/// 调用方先在目标恢复对应实体身份，并准备所需 schema；克隆范围和资源共享行为由具体组件实现决定。
/// 参见 `World::stage_reflected_component_clone`。
pub type ReflectComponentStageClone =
    fn(
        &crate::scene::World,
        crate::scene::EntityId,
        &str,
        &mut crate::scene::World,
    ) -> Result<(), zircon_runtime_interface::reflect::ReflectError>;

#[derive(Clone)]
/// 反射组件的 World 访问能力表；类型元数据由注册表持有，读写回调负责具体 ECS 存储和失效通知。
///
/// # 约束与生命周期
///
/// 本表只持有路径和函数指针，调用期间才借用 World。读结果拥有数据，写入值由回调消费。
/// 槽位依据对应注册的当前字段顺序；本表不重新执行目录准入或值树预算。
/// `WorldReflection` 在进入写回调前检查字段身份、可编辑性及预算，具体转换和提交由适配器承担。
/// `Clone` 复制表与路径，用于结束目录借用后访问 World。
///
/// # 参考
///
/// `WorldReflection::reflect_write`、`derived_component_registration`、动态组件适配器。
pub struct ReflectComponent {
    /// 传给回调的拥有型完整路径，须与配对元数据一致。
    pub type_path: String,
    /// 由提供方判断给定 World、实体和路径是否有组件实例。
    pub contains: fn(&crate::scene::World, crate::scene::EntityId, &str) -> bool,
    /// 按规范字段名读取拥有型值；错误由提供方返回。
    pub read_field: fn(
        &crate::scene::World,
        crate::scene::EntityId,
        &str,
        &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    >,
    /// 消费按名写入的值，并返回提供方的变更标记。
    pub write_field: fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
        &str,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
    /// 可选槽位读取能力；缺省时槽位入口返回 `InvalidRegistration`。
    pub read_field_by_slot: Option<ReflectComponentReadFieldBySlot>,
    /// 可选槽位写入能力；槽位入口不回退到按名回调。
    pub write_field_by_slot: Option<ReflectComponentWriteFieldBySlot>,
    /// 可选整批槽位写入能力；缺省时不拆成多次单字段写入。
    pub write_fields_by_slot: Option<ReflectComponentWriteFieldsBySlot>,
    /// 可选组件暂存能力；是否安装与是否存在组件是不同检查。
    pub stage_clone: Option<ReflectComponentStageClone>,
    /// 由提供方移除组件，并解释受保护类型或存储错误。
    pub remove: fn(
        &mut crate::scene::World,
        crate::scene::EntityId,
        &str,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
}

impl ReflectComponent {
    /// 组装按名访问和移除回调的候选表。
    ///
    /// # 参数
    ///
    /// `type_path` 的所有权归本表；其余参数为对应组件的存在性、读、写、移除函数。
    /// 调用方负责与注册元数据配对。
    ///
    /// # 返回值与约束
    ///
    /// 返回未安装槽位读写、批量写入或暂存回调的表；这里不校验路径、不注册类型、不执行回调。
    /// 需要这些能力时使用下方构建方法，参见 `derived_component_registration`。
    pub fn new(
        type_path: impl Into<String>,
        contains: fn(&crate::scene::World, crate::scene::EntityId, &str) -> bool,
        read_field: fn(
            &crate::scene::World,
            crate::scene::EntityId,
            &str,
            &str,
        ) -> Result<
            zircon_runtime_interface::reflect::ReflectedValue,
            zircon_runtime_interface::reflect::ReflectError,
        >,
        write_field: fn(
            &mut crate::scene::World,
            crate::scene::EntityId,
            &str,
            &str,
            zircon_runtime_interface::reflect::ReflectedValue,
        ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
        remove: fn(
            &mut crate::scene::World,
            crate::scene::EntityId,
            &str,
        ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
    ) -> Self {
        Self {
            type_path: type_path.into(),
            contains,
            read_field,
            write_field,
            read_field_by_slot: None,
            write_field_by_slot: None,
            write_fields_by_slot: None,
            stage_clone: None,
            remove,
        }
    }

    /// 消费本表并安装成对的当前槽位读写回调，返回更新后的表。
    ///
    /// 两个回调的槽位须对应配对 schema 的字段顺序；这里只保存函数指针。
    pub fn with_dense_field_slots(
        mut self,
        read_field_by_slot: ReflectComponentReadFieldBySlot,
        write_field_by_slot: ReflectComponentWriteFieldBySlot,
    ) -> Self {
        self.read_field_by_slot = Some(read_field_by_slot);
        self.write_field_by_slot = Some(write_field_by_slot);
        self
    }

    /// 消费本表并安装批量槽位写入回调，返回更新后的表。
    ///
    /// 批量转换、次序和提交边界由该回调实现；本方法不把单字段回调包装成批量操作。
    pub fn with_dense_field_batch_write(
        mut self,
        write_fields_by_slot: ReflectComponentWriteFieldsBySlot,
    ) -> Self {
        self.write_fields_by_slot = Some(write_fields_by_slot);
        self
    }

    /// Enables affected-row transactions to clone one component into an
    /// isolated preflight World without cloning the live World.
    ///
    /// 消费本表并保存暂存回调，返回更新后的表；不在构建时克隆数据。
    /// 调用协议与目标实体前提见 `ReflectComponentStageClone`。
    pub fn with_stage_clone(mut self, stage_clone: ReflectComponentStageClone) -> Self {
        self.stage_clone = Some(stage_clone);
        self
    }

    /// 把共享 World、稳定实体 ID 和本表路径交给存在性回调，并原样返回结果。
    pub fn contains(&self, world: &crate::scene::World, entity: crate::scene::EntityId) -> bool {
        (self.contains)(world, entity, &self.type_path)
    }

    /// 按规范字段名转发读取，成功结果拥有其数据，World 借用仅持续本次调用。
    ///
    /// `entity` 与本表路径定位组件；字段名和读取错误的解释由回调负责。
    pub fn read_field(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
        field_name: &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_field)(world, entity, &self.type_path, field_name)
    }

    /// 把按名写入的拥有型 `value` 交给回调，原样返回变更标记或错误。
    ///
    /// `world` 为本次操作的独占借用；本方法不追加字段准入或回滚，参见 `WorldReflection::reflect_write`。
    pub fn write_field(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
        field_name: &str,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_field)(world, entity, &self.type_path, field_name, value)
    }

    /// 按当前注册的零基槽位读取拥有型值。
    ///
    /// 未安装槽位读取回调时返回 `InvalidRegistration`；已安装时由回调解释槽位和存储错误。
    /// 调用方须将槽位与当前 schema 配对，参见 `TypeRegistry::resolve_field_slot_by_id`。
    pub fn read_field_by_slot(
        &self,
        world: &crate::scene::World,
        entity: crate::scene::EntityId,
        field_slot: u32,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        let Some(read_field_by_slot) = self.read_field_by_slot else {
            return Err(missing_dense_slot_adapter(&self.type_path, "read"));
        };
        read_field_by_slot(world, entity, &self.type_path, field_slot)
    }

    /// 消费当前槽位的字段值，原样返回写回调的变更标记或错误。
    ///
    /// 缺省回调先返回 `InvalidRegistration`；本方法不通过字段名补做写入。
    /// 调用方负责当前槽位及准入，参见 `WorldReflection::reflect_write`。
    pub fn write_field_by_slot(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
        field_slot: u32,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(write_field_by_slot) = self.write_field_by_slot else {
            return Err(missing_dense_slot_adapter(&self.type_path, "write"));
        };
        write_field_by_slot(world, entity, &self.type_path, field_slot, value)
    }

    /// 把有序 `fields` 列表交给批量回调，原样返回整批变更标记或错误。
    ///
    /// 缺省回调返回 `InvalidRegistration`；本层不追加逐项准入或通用事务保证。
    /// 派生组件和动态组件分别实现自己的候选与提交策略。
    pub fn write_fields_by_slot(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
        fields: Vec<(u32, zircon_runtime_interface::reflect::ReflectedValue)>,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(write_fields_by_slot) = self.write_fields_by_slot else {
            return Err(missing_dense_slot_adapter(&self.type_path, "batch write"));
        };
        write_fields_by_slot(world, entity, &self.type_path, fields)
    }

    /// 按组件暂存协议调用可选回调。
    ///
    /// `source` 为共享源 World，`entity` 是待暂存的稳定身份，`target` 为已准备身份的独占预检 World。
    /// 缺省回调返回 `Ok(false)`；回调成功返回 `Ok(true)`，错误原样传播。
    /// 这里的布尔值表示暂存能力已执行，不表示目标数据是否变化；上层 `World::stage_reflected_component_clone` 会把缺省能力转成错误。
    pub fn stage_clone(
        &self,
        source: &crate::scene::World,
        entity: crate::scene::EntityId,
        target: &mut crate::scene::World,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(stage_clone) = self.stage_clone else {
            return Ok(false);
        };
        stage_clone(source, entity, &self.type_path, target)?;
        Ok(true)
    }

    /// 把独占 World、实体与本表路径交给移除回调，原样返回其变更标记或错误。
    ///
    /// 可移除性与保护类型错误由具体适配器决定，参见派生组件的 `remove` 回调。
    pub fn remove(
        &self,
        world: &mut crate::scene::World,
        entity: crate::scene::EntityId,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.remove)(world, entity, &self.type_path)
    }
}

fn missing_dense_slot_adapter(
    type_path: &str,
    operation: &str,
) -> zircon_runtime_interface::reflect::ReflectError {
    zircon_runtime_interface::reflect::ReflectError::InvalidRegistration {
        type_path: type_path.to_string(),
        reason: format!("component reflection has no dense field-slot {operation} adapter"),
    }
}
