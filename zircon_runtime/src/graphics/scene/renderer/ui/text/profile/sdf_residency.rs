use super::super::ScreenSpaceUiTextPrepareReport;

/// 观察 SDF 烘焙期间的字体资源驻留失败；供预算和 fallback 排查，不改变本帧回退策略。
pub(super) fn record_sdf_residency_profile(report: &ScreenSpaceUiTextPrepareReport) {
    let bake = &report.sdf_renderer.bake;
    crate::profile_counter!(
        "runtime",
        "ui_text.sdf_prepare.resident_font_asset_errors",
        bake.resident_font_asset_error_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.sdf_prepare.resident_font_asset_no_registered_faces",
        bake.resident_font_asset_no_registered_faces_count
    );
}
