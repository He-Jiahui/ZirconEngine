//! 引用事件保留拖放对象及来源身份，供后续宿主定位、打开和投影；被接受的引用值在节点状态中持有已转移的载荷。

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiDragPayload,
    UiDragPayloadKind, UiDragSourceMetadata, UiValidationState, UiValue,
};

// 上层先检查事件支持，再由描述符整体的 drop_policy 筛选载荷种类；来源元数据随引用转移，后续普通值写入会清理它。
pub(super) fn drop_reference(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    payload: UiDragPayload,
) -> Result<(), UiComponentEventError> {
    if !descriptor.accepts_drag_payload(payload.kind) {
        state.validation = UiValidationState::error(format!(
            "rejected drop payload `{}` for {}",
            payload.kind.as_str(),
            descriptor.id
        ));
        return Err(UiComponentEventError::RejectedDrop {
            component_id: descriptor.id.clone(),
            payload_kind: payload.kind.as_str().to_string(),
        });
    }

    let (kind, reference, source) = into_reference_parts(payload);
    let value = match kind {
        UiDragPayloadKind::Asset => UiValue::AssetRef(reference),
        UiDragPayloadKind::SceneInstance | UiDragPayloadKind::Object => {
            UiValue::InstanceRef(reference)
        }
    };
    if let Some(source) = source {
        state.reference_sources.insert(property.clone(), source);
    } else {
        state.reference_sources.remove(&property);
    }
    state.values.insert(property, value);
    Ok(())
}

fn into_reference_parts(
    payload: UiDragPayload,
) -> (UiDragPayloadKind, String, Option<UiDragSourceMetadata>) {
    (payload.kind, payload.reference, payload.source)
}

pub(super) fn clear_reference(state: &mut UiComponentState, property: String) {
    state.reference_sources.remove(&property);
    state.values.insert(property, UiValue::Null);
}

// Locate/Open 在归约层只确认存在可用引用；实际定位或资源打开由消费事件的宿主适配器负责。
pub(super) fn ensure_reference_value(
    state: &mut UiComponentState,
    property: String,
) -> Result<(), UiComponentEventError> {
    match state.values.get(&property) {
        Some(UiValue::AssetRef(reference)) | Some(UiValue::InstanceRef(reference))
            if !reference.is_empty() =>
        {
            Ok(())
        }
        _ => {
            state.validation =
                UiValidationState::error(format!("reference property `{property}` is empty"));
            Err(UiComponentEventError::MissingReference { property })
        }
    }
}

#[cfg(test)]
#[path = "reference/tests/owned_drag_payload_tests.rs"]
mod owned_drag_payload_tests;
