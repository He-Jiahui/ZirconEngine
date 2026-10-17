//! 资源反射的同步 Rust 回调表，供 World 访问和动态场景资源预检使用。
//! 表复制仅复制函数指针；资源值由各 World 持有，暂存、创建与转移是分别提供的能力。

/// 按当前资源注册的零基槽位读取拥有型字段值的回调。
///
/// 共享 World 只在本次调用中借用；槽位须与配对 schema 一致。
pub type ReflectResourceReadFieldBySlot = fn(
    &crate::scene::World,
    u32,
) -> Result<
    zircon_runtime_interface::reflect::ReflectedValue,
    zircon_runtime_interface::reflect::ReflectError,
>;

/// 消费当前槽位的拥有型值并返回变更标记的资源写回调。
///
/// World 为独占借用；具体值转换、资源变更和错误边界由提供方实现。
pub type ReflectResourceWriteFieldBySlot =
    fn(
        &mut crate::scene::World,
        u32,
        zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

/// 消费有序槽位/值列表的资源批量回调，返回整批变更标记。
///
/// 输入次序、重复槽位和提交策略由提供方负责；调用者不能仅由此签名推导通用回滚。
pub type ReflectResourceWriteFieldsBySlot =
    fn(
        &mut crate::scene::World,
        Vec<(u32, zircon_runtime_interface::reflect::ReflectedValue)>,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>;

/// 把已在预检 World 验证和写入的资源转移到专用 artifact World 的回调。
///
/// 两个参数分别为独占源预检 World 和独占 artifact World；成功返回 `()`，错误交给预检调用方。
/// 调用发生在 live target 发布前，参见动态场景的 `transfer_preflight_resource_writes` 和 `World::transfer_preflight_resource`。
pub type ReflectResourcePreflightTransfer =
    fn(
        &mut crate::scene::World,
        &mut crate::scene::World,
    ) -> Result<(), zircon_runtime_interface::reflect::ReflectError>;

#[derive(Clone, Copy)]
/// 反射资源的 World 访问能力表；预检转移回调用于场景暂存，调用者不得假定所有资源都可持久化克隆。
///
/// # 约束与生命周期
///
/// 本表可 `Copy`，不持有 World 借用或资源值；调用期间按读写需求借用 World。
/// 字段槽位须与配对元数据的当前顺序对应，返回字段值拥有其数据。
/// 预算统计、资源是否存在及缺省能力的错误处理由调用方负责；具体回调决定转换和提交策略。
///
/// # 参考
///
/// `WorldReflection`；动态场景的 `stage_existing_resource_bounded`、`ensure_reflected_resource_exists`。
pub struct ReflectResource {
    /// 可选暂存字节估算，单位为字节；回调本身不累计或强制总预算。
    pub estimate_stage_clone_bytes: Option<
        fn(&crate::scene::World) -> Result<usize, zircon_runtime_interface::reflect::ReflectError>,
    >,
    /// 可选从共享源到预检目标的资源克隆能力，有界场景预检还要求提供估算能力。
    pub stage_clone: Option<
        fn(
            &crate::scene::World,
            &mut crate::scene::World,
        ) -> Result<(), zircon_runtime_interface::reflect::ReflectError>,
    >,
    /// Moves the already validated preflight value into a dedicated artifact
    /// World before the live target begins publication.
    ///
    /// 转移的是预检中已经接纳的资源；调用方在最终 target 发布前执行。
    pub transfer_preflight: ReflectResourcePreflightTransfer,
    /// 可选资源创建能力；返回 `false` 也可能表示资源已经存在，调用方另行检查 `contains`。
    pub ensure: Option<
        fn(
            &mut crate::scene::World,
        ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError>,
    >,
    /// 检查本次 World 中的资源实例是否存在。
    pub contains: fn(&crate::scene::World) -> bool,
    /// 按规范字段名读取拥有型值；名称、资源和转换错误由回调解释。
    pub read_field: fn(
        &crate::scene::World,
        &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    >,
    /// 必需的当前槽位读取回调，供稳定字段 ID 解析后的访问使用。
    pub read_field_by_slot: ReflectResourceReadFieldBySlot,
    /// 必需的当前槽位写回调，消费字段值并报告变更。
    pub write_field_by_slot: ReflectResourceWriteFieldBySlot,
    /// 必需的批量槽位写回调；不由本表自动拆成多次单字段写入。
    pub write_fields_by_slot: ReflectResourceWriteFieldsBySlot,
}

impl ReflectResource {
    /// 向可选估算回调查询本次暂存所需的字节数。
    ///
    /// `source` 为共享借用。成功时 `Some(bytes)` 表示提供方估算，`None` 表示没有估算能力；回调错误传播。
    /// 本方法不累计预算或克隆资源；动态场景调用方先累计并检查上限，才进入 `stage_clone`。
    pub fn estimate_stage_clone_bytes(
        &self,
        source: &crate::scene::World,
    ) -> Result<Option<usize>, zircon_runtime_interface::reflect::ReflectError> {
        self.estimate_stage_clone_bytes
            .map(|estimate| estimate(source))
            .transpose()
    }

    /// 调用可选资源克隆回调，把共享源值暂存到独占预检 World。
    ///
    /// 缺省回调返回 `Ok(false)`；执行成功返回 `Ok(true)`，错误原样传播。
    /// 布尔值表示能力已执行，资源存在性与预算由调用方先检查；参见 `stage_existing_resource_bounded`。
    pub fn stage_clone(
        &self,
        source: &crate::scene::World,
        target: &mut crate::scene::World,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(stage_clone) = self.stage_clone else {
            return Ok(false);
        };
        stage_clone(source, target)?;
        Ok(true)
    }

    /// 把两个独占 World 借用交给转移回调，并原样返回结果。
    ///
    /// `source` 是已完成资源验证/写入的预检 World，`artifact` 接管该资源，供后续发布。
    /// 本包装不重新校验字段，也不复制或恢复源资源；实际转移由 `ReflectResourcePreflightTransfer` 的提供方实现。
    pub fn transfer_preflight(
        &self,
        source: &mut crate::scene::World,
        artifact: &mut crate::scene::World,
    ) -> Result<(), zircon_runtime_interface::reflect::ReflectError> {
        (self.transfer_preflight)(source, artifact)
    }

    /// 尝试调用可选资源创建回调，原样返回其变更标记或错误。
    ///
    /// 没有回调时返回 `Ok(false)`；该结果不能单独表示资源已存在或创建失败。
    /// 动态场景的 `ensure_reflected_resource_exists` 在调用后复查 `contains`。
    pub fn ensure(
        &self,
        world: &mut crate::scene::World,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        let Some(ensure) = self.ensure else {
            return Ok(false);
        };
        ensure(world)
    }

    /// 把共享 World 交给资源存在性回调，并原样返回结果。
    pub fn contains(&self, world: &crate::scene::World) -> bool {
        (self.contains)(world)
    }

    /// 按规范字段名转发读取，返回拥有型值或提供方错误；World 借用仅持续本次调用。
    pub fn read_field(
        &self,
        world: &crate::scene::World,
        field_name: &str,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_field)(world, field_name)
    }

    /// 按配对注册的当前零基槽位转发读取，返回拥有型值或回调错误。
    ///
    /// 持久字段引用先由 catalog 解析为当前槽位，参见 `WorldReflection`。
    pub fn read_field_by_slot(
        &self,
        world: &crate::scene::World,
        field_slot: u32,
    ) -> Result<
        zircon_runtime_interface::reflect::ReflectedValue,
        zircon_runtime_interface::reflect::ReflectError,
    > {
        (self.read_field_by_slot)(world, field_slot)
    }

    /// 把独占 World 和有序 `fields` 的所有权交给批量回调，原样返回整批变更标记或错误。
    ///
    /// 本层不追加准入、逐项回调或通用回滚；场景调用方先确保资源存在。
    pub fn write_fields_by_slot(
        &self,
        world: &mut crate::scene::World,
        fields: Vec<(u32, zircon_runtime_interface::reflect::ReflectedValue)>,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_fields_by_slot)(world, fields)
    }

    /// 把独占 World、当前槽位和拥有型值交给回调，原样返回变更标记或错误。
    ///
    /// 本层不追加值预算或字段准入；门面路径见 `WorldReflection::reflect_write`。
    pub fn write_field_by_slot(
        &self,
        world: &mut crate::scene::World,
        field_slot: u32,
        value: zircon_runtime_interface::reflect::ReflectedValue,
    ) -> Result<bool, zircon_runtime_interface::reflect::ReflectError> {
        (self.write_field_by_slot)(world, field_slot, value)
    }
}
