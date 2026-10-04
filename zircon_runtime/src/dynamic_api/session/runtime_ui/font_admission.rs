use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::font::{prepare_runtime_font_asset_admission, RuntimeFontAssetClaimScope};
use crate::ui::surface::UiSurface;

pub(super) fn admit_surface_font_dependencies<'a>(
    surfaces: impl IntoIterator<Item = &'a UiSurface>,
    asset_manager: &ProjectAssetManager,
    font_claim_scope: &mut RuntimeFontAssetClaimScope,
) {
    let mut font_dependencies = surfaces
        .into_iter()
        .flat_map(UiSurface::text_font_asset_dependencies)
        .collect::<Vec<_>>();
    font_dependencies.sort_unstable();
    font_dependencies.dedup();
    let shared_dependencies = font_dependencies
        .iter()
        .map(|asset_ref| Arc::<str>::from(asset_ref.as_str()))
        .collect::<Vec<_>>();
    let admissions = shared_dependencies
        .iter()
        .cloned()
        .map(|asset_ref| prepare_runtime_font_asset_admission(asset_manager, asset_ref))
        .collect();
    // 以本次完整依赖集合替换旧 claim，统一计算新增、释放和 admission 结果。
    let transition =
        font_claim_scope.replace_shared_claims_with_admissions(&shared_dependencies, admissions);
    let claim_report = transition.claims;
    let mut admission_success_count = 0_usize;
    let mut admission_failure_count = 0_usize;
    let mut admission_changed_count = 0_usize;
    let mut registered_face_count = 0_usize;
    for outcome in transition.admissions {
        match outcome.result {
            Ok(report) => {
                admission_success_count = admission_success_count.saturating_add(1);
                admission_changed_count =
                    admission_changed_count.saturating_add(usize::from(report.font_inputs_changed));
                registered_face_count =
                    registered_face_count.saturating_add(report.registered_face_count);
            }
            Err(_) => {
                admission_failure_count = admission_failure_count.saturating_add(1);
            }
        }
    }
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_dependency_count",
        font_dependencies.len()
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_claim_added_count",
        claim_report.added_claim_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_claim_released_count",
        claim_report.released_claim_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_claim_unclaimed_count",
        claim_report.unclaimed_asset_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_claim_font_inputs_changed",
        u8::from(claim_report.font_inputs_changed)
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_admission_success_count",
        admission_success_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_admission_failure_count",
        admission_failure_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.font_asset_admission_changed_count",
        admission_changed_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.project_text.registered_font_face_count",
        registered_face_count
    );
}

#[cfg(test)]
#[path = "tests/font_admission.rs"]
mod tests;
