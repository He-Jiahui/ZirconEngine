use super::{
    bindless_material_eligibility, BindlessMaterialEligibility, BindlessMaterialFallbackReason,
};

#[test]
fn render_bindless_material_eligibility_accepts_only_the_representable_standard_case() {
    assert_eq!(
        bindless_material_eligibility(true, false, false),
        BindlessMaterialEligibility::Eligible
    );
    assert!(bindless_material_eligibility(true, false, false).uses_bindless());
}

#[test]
fn render_bindless_material_eligibility_fails_closed_for_each_unrepresented_input() {
    assert_eq!(
        bindless_material_eligibility(false, false, false),
        BindlessMaterialEligibility::PerMaterialFallback(
            BindlessMaterialFallbackReason::NonStandardSurface
        )
    );
    assert_eq!(
        bindless_material_eligibility(true, true, false),
        BindlessMaterialEligibility::PerMaterialFallback(
            BindlessMaterialFallbackReason::PropertyUniformOverride
        )
    );
    assert_eq!(
        bindless_material_eligibility(true, false, true),
        BindlessMaterialEligibility::PerMaterialFallback(
            BindlessMaterialFallbackReason::OutputTargetTexture
        )
    );
}
