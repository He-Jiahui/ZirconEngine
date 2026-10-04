use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::render::RenderMaterialTextureDimension;

use super::*;

#[test]
fn texture_dimension_mismatch_projects_dependency_diagnostic() {
    let reference = AssetReference::from_locator(
        AssetUri::parse("res://textures/sky.ztexture").expect("texture uri"),
    );

    let row = diagnostic_row_for_error(RenderMaterialValidationError::TextureDimensionMismatch {
        slot: "environment".to_string(),
        reference,
        expected: RenderMaterialTextureDimension::Cube,
        actual: RenderMaterialTextureDimension::D2,
    });

    assert_eq!(
        row.source,
        Some(RenderMaterialDiagnosticSource::DependencyResolution)
    );
    assert_eq!(row.path, "textures.environment");
    assert!(row.message.contains("expected Cube"));
    assert!(row.message.contains("resolved D2"));
}
