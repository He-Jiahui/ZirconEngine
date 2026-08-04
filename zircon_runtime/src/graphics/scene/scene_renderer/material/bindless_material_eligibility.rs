/// Why a material must use the established per-material group-2 path for this frame.
///
/// These are correctness gates, not capability failures: a bindless-capable device can still
/// render materials with instance overrides or output-target textures through the fallback path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BindlessMaterialFallbackReason {
    NonStandardSurface,
    PropertyUniformOverride,
    OutputTargetTexture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BindlessMaterialEligibility {
    Eligible,
    PerMaterialFallback(BindlessMaterialFallbackReason),
}

impl BindlessMaterialEligibility {
    pub(crate) const fn uses_bindless(self) -> bool {
        matches!(self, Self::Eligible)
    }
}

/// Applies the material-level constraints that sit below the device capability gate.
///
/// The current 224-byte payload mirrors only the Standard PBR uniform layout. A custom material
/// surface, per-entity property override, or render-target texture therefore keeps the existing
/// group-2 binding contract until it has an exact equivalent bindless representation.
pub(crate) const fn bindless_material_eligibility(
    uses_standard_surface: bool,
    has_property_uniform_override: bool,
    has_output_target_texture: bool,
) -> BindlessMaterialEligibility {
    if !uses_standard_surface {
        return BindlessMaterialEligibility::PerMaterialFallback(
            BindlessMaterialFallbackReason::NonStandardSurface,
        );
    }
    if has_property_uniform_override {
        return BindlessMaterialEligibility::PerMaterialFallback(
            BindlessMaterialFallbackReason::PropertyUniformOverride,
        );
    }
    if has_output_target_texture {
        return BindlessMaterialEligibility::PerMaterialFallback(
            BindlessMaterialFallbackReason::OutputTargetTexture,
        );
    }
    BindlessMaterialEligibility::Eligible
}

#[cfg(test)]
mod tests {
    use super::{
        BindlessMaterialEligibility, BindlessMaterialFallbackReason, bindless_material_eligibility,
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
}
