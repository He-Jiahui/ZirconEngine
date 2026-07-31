use zircon_editor::core::asset::{
    DirtyDocumentSnapshot, DirtyExternalEffectId, DirtyExternalEffectIdError,
    DirtyExternalEffectRevision, DirtyRegistry, DirtyRegistryCursor, DirtyRegistryDelta,
    DirtyRegistryError, EditorAssetImportAdmissionLimits, EditorAssetImportFlow,
    EditorAssetImportReason, EditorAssetImportRequest, EditorAssetImportResult,
    EditorAssetImportSubmitError, EditorAssetImportTicket,
};
use zircon_runtime::asset::AssetUri;

fn assert_public_type<T>() {}

#[test]
fn asset_facade_exposes_dirty_and_import_contracts_from_one_public_path() {
    assert_public_type::<DirtyDocumentSnapshot>();
    assert_public_type::<DirtyExternalEffectRevision>();
    assert_public_type::<DirtyRegistry>();
    assert_public_type::<DirtyRegistryCursor>();
    assert_public_type::<DirtyRegistryDelta>();
    assert_public_type::<DirtyRegistryError>();
    assert_public_type::<DirtyExternalEffectIdError>();
    assert_public_type::<EditorAssetImportAdmissionLimits>();
    assert_public_type::<EditorAssetImportFlow>();
    assert_public_type::<EditorAssetImportResult>();
    assert_public_type::<EditorAssetImportSubmitError>();
    assert_public_type::<EditorAssetImportTicket>();

    let effect = DirtyExternalEffectId::parse("asset.import_settings").unwrap();
    assert_eq!(effect.as_str(), "asset.import_settings");

    let uri = AssetUri::parse("res://textures/sky.png").unwrap();
    let request = EditorAssetImportRequest::new(uri.clone(), EditorAssetImportReason::Manual);
    assert_eq!(request.uri(), &uri);
    assert_eq!(request.reason(), EditorAssetImportReason::Manual);
}
