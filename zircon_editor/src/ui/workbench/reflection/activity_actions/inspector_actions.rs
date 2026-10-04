//! 检查器公开批量提交、字段草稿和动画轨道入口；反射描述本身不执行写入。
use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) const INSPECTOR_ACTION_COUNT: usize = 3;

/// 声明检查器动作的外部参数形状，供路由注册和远程发现共用。
pub(super) fn inspector_actions() -> [UiActionDescriptor; INSPECTOR_ACTION_COUNT] {
    [
        UiActionDescriptor::new(
            "inspector.apply_batch.invoke",
            UiEventKind::Click,
            "InspectorFieldBatch",
        )
        .with_parameter(UiParameterDescriptor::new(
            "subject_path",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("changes", UiValueType::Array)),
        UiActionDescriptor::new(
            "inspector.field.edit",
            UiEventKind::Change,
            "DraftCommand.SetInspectorField",
        )
        .with_parameter(UiParameterDescriptor::new(
            "subject_path",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("field_id", UiValueType::String))
        .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
        UiActionDescriptor::new(
            "animation.track.create",
            UiEventKind::Click,
            "AnimationCommand.CreateTrack",
        )
        .with_parameter(UiParameterDescriptor::new(
            "track_path",
            UiValueType::String,
        )),
    ]
}
