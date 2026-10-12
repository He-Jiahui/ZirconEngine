use super::AdvancedPbrMaterialFrameUsage;
use crate::core::framework::render::StandardPbrMaterialFeatures;

#[test]
fn render_advanced_material_frame_usage_only_requests_copy_for_specular_transmission() {
    let mut usage = AdvancedPbrMaterialFrameUsage::default();
    usage.record(&StandardPbrMaterialFeatures {
        diffuse_transmission: 0.4,
        ..Default::default()
    });

    assert!(usage.requires_forward_path());
    assert!(!usage.requires_scene_color_copy());

    usage.record(&StandardPbrMaterialFeatures {
        specular_transmission: 0.7,
        ..Default::default()
    });
    assert!(usage.requires_scene_color_copy());
}

#[test]
fn render_advanced_material_frame_usage_separates_opaque_forward_from_transmission() {
    let mut usage = AdvancedPbrMaterialFrameUsage::default();
    usage.record(&StandardPbrMaterialFeatures {
        clearcoat: 1.0,
        ..Default::default()
    });
    assert!(usage.requires_late_forward_opaque_pass());

    let mut transmitted = AdvancedPbrMaterialFrameUsage::default();
    transmitted.record(&StandardPbrMaterialFeatures {
        clearcoat: 1.0,
        specular_transmission: 0.5,
        ..Default::default()
    });
    assert!(!transmitted.requires_late_forward_opaque_pass());
    assert!(transmitted.uses_transmission());
    assert!(transmitted.requires_scene_color_copy());
}

#[test]
fn render_advanced_material_frame_usage_routes_only_non_default_ior_to_late_opaque() {
    let mut usage = AdvancedPbrMaterialFrameUsage::default();
    usage.record(&StandardPbrMaterialFeatures {
        ior: 2.5,
        ..Default::default()
    });

    assert!(usage.dielectric_f0_override);
    assert!(usage.requires_forward_path());
    assert!(usage.requires_late_forward_opaque_pass());
    assert!(!usage.requires_scene_color_copy());
}
