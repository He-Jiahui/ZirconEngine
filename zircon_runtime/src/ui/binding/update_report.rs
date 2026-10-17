//! 属性事务、组件默认行为和辅助技术动作共同使用的报告构造边界。
//! 调用者先决定是否应用、前后值及脏域；这里保留来源与目标区分，不执行写入或触发刷新。
//!
//! # 共同参数与所有权
//! 节点和属性标识只进入报告，不在这里查找目标或验证属性；前后值、状态和脏标记由调用者提供。
//! 属性名转换为拥有的字符串，前后值移入报告；保留的拒绝消息也由报告拥有，不借用调用者数据。
//! 状态不通过前后值比较推断；Rejected 也会保留传入的脏标记，聚合时不会据状态过滤它们。

use zircon_runtime_interface::ui::{
    binding::{
        UiBindingSource, UiBindingSourceKind, UiBindingTarget, UiBindingUpdate,
        UiBindingUpdateReport, UiBindingUpdateStatus,
    },
    component::UiValue,
    event_ui::{UiNodeId, UiReflectedPropertySource},
    tree::UiDirtyFlags,
};

/// 为同一节点、属性构造保留属性来源和目标的更新项；在调用方决定事务结果后使用。
///
/// # 参数与返回
/// `node_id`、`property` 标识报告两端；`previous`、`value` 移入报告，`dirty` 与 `status`
/// 原样决定报告分类，不触发写入。返回拥有型更新项；Rejected 的消息为空字符串。
pub fn retained_attribute_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        UiBindingSource::retained_attribute(node_id, property.clone()),
        UiBindingTarget::retained_attribute(node_id, property),
        previous,
        value,
        dirty,
        status,
        None,
    )
}

/// 构造运行时状态来源、组件状态值目标的报告；参数所有权与 [`retained_attribute_update`] 相同。
/// `node_id` 和 `property` 复用于两端；前后值、脏标记和状态由调用者决定，返回项不执行状态写入。
pub fn component_state_value_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
) -> UiBindingUpdate {
    component_state_value_update_with_source_kind(
        node_id,
        property,
        UiBindingSourceKind::RuntimeState,
        previous,
        value,
        dirty,
        status,
    )
}

/// 为组件状态值目标记录调用方选定的来源分类。
///
/// # 参数与返回
/// `node_id`、`property` 复用于来源及目标，`source_kind` 只决定报告分类；
/// `previous`、`value` 移入返回项，`dirty`、`status` 不重新计算。返回项拥有属性名与值，
/// 不借用调用者数据，也不修改组件；Rejected 没有另传消息，使用空字符串。
pub fn component_state_value_update_with_source_kind(
    node_id: UiNodeId,
    property: impl Into<String>,
    source_kind: UiBindingSourceKind,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        UiBindingSource {
            kind: source_kind,
            node_id: Some(node_id),
            property: Some(property.clone()),
            path: None,
        },
        UiBindingTarget::component_state_value(node_id, property),
        previous,
        value,
        dirty,
        status,
        None,
    )
}

/// 报告一个来源位置到运行时状态目标的结果，允许来源和目标位于不同节点或属性。
///
/// # 参数与返回
/// 两组节点、属性参数分别标识来源和目标，`source_kind` 标记来源；前后值、脏标记和状态
/// 由调用方提供并移入拥有型更新项，不在此写入运行时状态。`message` 仅在 Rejected 时保留，
/// 缺少时使用空字符串；Applied 和 Unchanged 不保留该消息参数。
pub fn runtime_state_update_with_source_kind(
    source_node_id: UiNodeId,
    source_property: impl Into<String>,
    source_kind: UiBindingSourceKind,
    target_node_id: UiNodeId,
    target_property: impl Into<String>,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    update_with_status(
        UiBindingSource {
            kind: source_kind,
            node_id: Some(source_node_id),
            property: Some(source_property.into()),
            path: None,
        },
        UiBindingTarget::runtime_state(target_node_id, target_property),
        previous,
        value,
        dirty,
        status,
        message,
    )
}

/// 为组件行为尝试写入别名的拒绝结果构造报告。
///
/// `node_id`、`property` 复用于来源和别名目标；`value` 与 `message` 移入返回项。
/// 返回值固定为 Rejected、无前值且无脏域；本函数不尝试写入，也不根据属性名判断是否可写。
pub fn rejected_widget_alias_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    value: UiValue,
    message: impl Into<String>,
) -> UiBindingUpdate {
    let property = property.into();
    UiBindingUpdate::rejected(
        UiBindingSource::widget_behavior(node_id, property.clone()),
        UiBindingTarget::widget_alias(node_id, property),
        value,
        message,
    )
}

/// 将反射属性的来源分类映射到绑定报告，目标始终为同节点的保留属性。
///
/// # 参数与返回
/// `source` 由 `reflected_property_source_kind` 的固定映射解释；节点、属性定位两端，
/// 前后值、脏标记和状态由调用者提供。属性名和值移入返回项，不执行属性事务；
/// `message` 只供 Rejected 使用，其他状态忽略它，缺少拒绝消息时使用空字符串。
pub fn reflected_property_update(
    node_id: UiNodeId,
    property: impl Into<String>,
    source: UiReflectedPropertySource,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        reflected_property_source(source, node_id, property.clone()),
        UiBindingTarget::retained_attribute(node_id, property),
        previous,
        value,
        dirty,
        status,
        message,
    )
}

/// 为反射属性报告显式指定来源分类，跳过反射来源枚举的默认映射。
///
/// `node_id`、`property` 标识来源与保留属性目标；`source_kind` 只影响来源分类。
/// 前后值、脏标记、状态和消息的所有权及处理规则同 [`reflected_property_update`]，
/// 返回独立拥有的更新项，不修改表面或验证来源分类与属性是否匹配。
pub fn reflected_property_update_with_source_kind(
    node_id: UiNodeId,
    property: impl Into<String>,
    source_kind: UiBindingSourceKind,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    let property = property.into();
    update_with_status(
        UiBindingSource {
            kind: source_kind,
            node_id: Some(node_id),
            property: Some(property.clone()),
            path: None,
        },
        UiBindingTarget::retained_attribute(node_id, property),
        previous,
        value,
        dirty,
        status,
        message,
    )
}

/// 接管调用方已构造的更新项，按状态计数并按首次出现顺序汇总去重后的脏域。
///
/// 返回报告拥有原顺序的 `updates`，不应用这些更新，也不补造事务或执行回执；
/// 聚合行为由 [`UiBindingUpdateReport::from_updates`] 完成，包含所有状态项的脏域。
pub fn binding_update_report(updates: Vec<UiBindingUpdate>) -> UiBindingUpdateReport {
    UiBindingUpdateReport::from_updates(updates)
}

/// 将反射来源映射到报告来源：作者值和默认值归保留属性，运行时和系统值归运行时，绑定归组件事件。
/// 这是穷尽的静态分类，不解析绑定、不调用组件，也不据目标类型推断来源。
pub(crate) const fn reflected_property_source_kind(
    source: UiReflectedPropertySource,
) -> UiBindingSourceKind {
    match source {
        UiReflectedPropertySource::Authored
        | UiReflectedPropertySource::DescriptorDefault
        | UiReflectedPropertySource::InferredDefault => UiBindingSourceKind::RetainedAttribute,
        UiReflectedPropertySource::RuntimeState | UiReflectedPropertySource::SystemState => {
            UiBindingSourceKind::RuntimeState
        }
        UiReflectedPropertySource::Binding => UiBindingSourceKind::ComponentEvent,
    }
}

/// 按上述固定分类构造拥有型来源；仅三个映射结果可到达，因此其余分支是本地不变量保护。
fn reflected_property_source(
    source: UiReflectedPropertySource,
    node_id: UiNodeId,
    property: String,
) -> UiBindingSource {
    match reflected_property_source_kind(source) {
        UiBindingSourceKind::RetainedAttribute => {
            UiBindingSource::retained_attribute(node_id, property)
        }
        UiBindingSourceKind::RuntimeState => UiBindingSource::runtime_state(node_id, property),
        UiBindingSourceKind::ComponentEvent => UiBindingSource::component_event(node_id, property),
        _ => unreachable!("reflected property sources map to retained, runtime, or binding state"),
    }
}

/// 仅按给定状态选择构造器，再为所有状态附加前值和脏标记；消息只进入拒绝项。
fn update_with_status(
    source: UiBindingSource,
    target: UiBindingTarget,
    previous: Option<UiValue>,
    value: UiValue,
    dirty: UiDirtyFlags,
    status: UiBindingUpdateStatus,
    message: Option<String>,
) -> UiBindingUpdate {
    let update = match status {
        UiBindingUpdateStatus::Applied => UiBindingUpdate::applied(source, target, value),
        UiBindingUpdateStatus::Unchanged => UiBindingUpdate::unchanged(source, target, value),
        UiBindingUpdateStatus::Rejected => {
            UiBindingUpdate::rejected(source, target, value, message.unwrap_or_default())
        }
    };
    update.with_previous(previous).with_dirty_flags(dirty)
}
