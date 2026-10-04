//! 只记录固定名称的次数与字节规模，分析文本编辑复制成本时不暴露文本、控件身份或属性名。

#[inline]
pub(in crate::ui::surface::input) fn record_state_materialization(source_bytes: usize) {
    crate::profile_counter!("runtime", "ui_text.edit.state_materializations", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.state_materialized_bytes",
        source_bytes
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = source_bytes;
}

#[inline]
pub(super) fn record_property_value_clone(source_bytes: usize) {
    crate::profile_counter!("runtime", "ui_text.edit.property_value_clones", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.property_value_clone_bytes",
        source_bytes
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = source_bytes;
}

#[inline]
pub(super) fn record_property_projection(
    source_bytes: usize,
    committed: bool,
    composition_active: bool,
    visible_preedit_bytes: usize,
) {
    crate::profile_counter!("runtime", "ui_text.edit.property_projections", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.property_projected_bytes",
        source_bytes
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.committed_projections",
        committed as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.state_only_projections",
        (!committed) as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.composition_projections",
        composition_active as usize
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.visible_preedit_bytes",
        visible_preedit_bytes
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = (
        source_bytes,
        committed,
        composition_active,
        visible_preedit_bytes,
    );
}

#[inline]
pub(super) fn record_component_payload(payload_bytes: usize) {
    crate::profile_counter!("runtime", "ui_text.edit.component_payloads", 1);
    crate::profile_counter!(
        "runtime",
        "ui_text.edit.component_payload_bytes",
        payload_bytes
    );
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    let _ = payload_bytes;
}

#[cfg(test)]
#[path = "tests/profile.rs"]
mod tests;
