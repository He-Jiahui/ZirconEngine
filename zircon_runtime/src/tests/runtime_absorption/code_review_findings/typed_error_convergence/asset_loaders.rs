#[test]
fn review_f5_texture_loader_uses_typed_error() {
    let texture_loader = include_str!("../../../../asset/load/texture.rs");
    let worker_pool = include_str!("../../../../asset/pipeline/worker_pool.rs");
    let texture_tests = include_str!("../../../../asset/tests/load/texture.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let worker_doc = include_str!("../../../../../../docs/zircon_runtime/asset/worker_pool.md");

    for required in [
        "pub(crate) type TextureLoadResult<T> = std::result::Result<T, TextureLoadError>;",
        "pub(crate) enum TextureLoadError",
        "OpenImage {",
        "#[source]",
        "source: image::ImageError",
        "pub(crate) fn load_texture(source: &TextureSource) -> TextureLoadResult<CpuTexturePayload>",
        "pub(crate) fn decode_image_file(path: &str) -> TextureLoadResult<CpuTexturePayload>",
        "TextureLoadError::OpenImage",
    ] {
        assert!(
            texture_loader.contains(required),
            "F5 texture loader typed error owner should contain `{required}`"
        );
    }

    for forbidden in [
        "Result<CpuTexturePayload, String>",
        "format!(\"open image {path}: {error}\")",
    ] {
        assert!(
            !texture_loader.contains(forbidden),
            "texture loader should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required in [
        "message: error.to_string()",
        "missing_image_file_reports_typed_texture_load_error",
        "TextureLoadError::OpenImage",
    ] {
        assert!(
            worker_pool.contains(required) || texture_tests.contains(required),
            "texture loader worker/test surface should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 texture loader typed errors",
        "runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred",
        "review_f5_texture_loader_uses_typed_error",
        "TextureLoadError::OpenImage",
        "asset/load/texture.rs",
        "asset/tests/load/texture.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || worker_doc.contains(doc_anchor),
            "F5 texture loader docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_mesh_loader_and_obj_decoder_use_typed_errors() {
    let mesh_loader = include_str!("../../../../asset/load/mesh.rs");
    let obj_error = include_str!("../../../../asset/formats/obj/error.rs");
    let obj_mod = include_str!("../../../../asset/formats/obj/mod.rs");
    let obj_decode = include_str!("../../../../asset/formats/obj/decode_obj_file.rs");
    let obj_face = include_str!("../../../../asset/formats/obj/parse_obj_face_vertex.rs");
    let obj_scalar = include_str!("../../../../asset/formats/obj/parse_obj_scalar.rs");
    let obj_index = include_str!("../../../../asset/formats/obj/resolve_obj_index.rs");
    let worker_pool = include_str!("../../../../asset/pipeline/worker_pool.rs");
    let mesh_tests = include_str!("../../../../asset/tests/load/mesh.rs");
    let obj_tests = include_str!("../../../../asset/tests/formats/obj.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let worker_doc = include_str!("../../../../../../docs/zircon_runtime/asset/worker_pool.md");

    for required in [
        "pub(crate) type MeshLoadResult<T> = std::result::Result<T, MeshLoadError>;",
        "pub(crate) enum MeshLoadError",
        "Obj {",
        "source: obj::ObjDecodeError",
        "UnsupportedFormat { path: String, extension: String }",
        "pub(crate) fn load_mesh(source: &MeshSource) -> MeshLoadResult<CpuMeshPayload>",
        "pub(crate) fn decode_mesh_file(path: &str) -> MeshLoadResult<CpuMeshPayload>",
        "MeshLoadError::UnsupportedFormat",
    ] {
        assert!(
            mesh_loader.contains(required),
            "F5 mesh loader typed error owner should contain `{required}`"
        );
    }

    for forbidden in [
        "Result<CpuMeshPayload, String>",
        "unsupported mesh format for {path}",
        "Err(format!(",
    ] {
        assert!(
            !mesh_loader.contains(forbidden),
            "mesh loader should not keep lossy String error branch `{forbidden}`"
        );
    }

    for required in [
        "pub(crate) type ObjDecodeResult<T> = std::result::Result<T, ObjDecodeError>;",
        "pub(crate) enum ObjDecodeError",
        "Read {",
        "InvalidScalar {",
        "FaceVertex {",
        "EmptyPositions { path: String }",
        "EmptyFaces { path: String }",
        "#[source]",
    ] {
        assert!(
            obj_error.contains(required),
            "OBJ decoder typed error owner should contain `{required}`"
        );
    }
    assert!(
        obj_mod.contains("pub(crate) use error::{ObjDecodeError, ObjDecodeResult};"),
        "OBJ module should export typed decode error/result to mesh loader and tests"
    );

    for required in [
        "pub(crate) fn decode_obj_file(path: &str) -> ObjDecodeResult<CpuMeshPayload>",
        "ObjDecodeError::Read",
        "ObjDecodeError::FaceVertexCount",
        "ObjDecodeError::FaceVertex",
        "ObjDecodeError::EmptyPositions",
        "ObjDecodeError::EmptyFaces",
        ") -> ObjDecodeResult<f32>",
        ") -> ObjDecodeResult<ObjVertexKey>",
        ") -> ObjDecodeResult<usize>",
        "ObjDecodeError::InvalidScalar",
        "ObjDecodeError::InvalidIndex",
        "ObjDecodeError::IndexOutOfBounds",
    ] {
        assert!(
            obj_decode.contains(required)
                || obj_face.contains(required)
                || obj_scalar.contains(required)
                || obj_index.contains(required),
            "OBJ decoder typed path should contain `{required}`"
        );
    }

    for (label, source) in [
        ("OBJ decode", obj_decode),
        ("OBJ face parser", obj_face),
        ("OBJ scalar parser", obj_scalar),
        ("OBJ index resolver", obj_index),
    ] {
        for forbidden in [
            "Result<CpuMeshPayload, String>",
            "Result<ObjVertexKey, String>",
            "Result<f32, String>",
            "Result<usize, String>",
            "Err(format!(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String error branch `{forbidden}`"
            );
        }
    }

    let mesh_branch = worker_pool
        .split("AssetRequest::Mesh(source)")
        .nth(1)
        .expect("worker pool mesh branch");
    assert!(
        mesh_branch.contains("message: error.to_string()"),
        "worker pool mesh branch should stringify MeshLoadError only at CpuAssetPayload::Failure boundary"
    );
    for required in [
        "unsupported_mesh_file_reports_typed_mesh_load_error",
        "MeshLoadError::UnsupportedFormat",
        "obj_decode_reports_typed_read_error_source",
        "obj_decode_reports_typed_scalar_parse_error_source",
        "ObjDecodeError::Read",
        "ObjDecodeError::InvalidScalar",
    ] {
        assert!(
            mesh_tests.contains(required) || obj_tests.contains(required),
            "mesh/OBJ typed error tests should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 mesh loader typed errors",
        "runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred",
        "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
        "MeshLoadError::UnsupportedFormat",
        "ObjDecodeError::Read",
        "asset/load/mesh.rs",
        "asset/formats/obj/error.rs",
        "asset/tests/formats/obj.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || worker_doc.contains(doc_anchor),
            "F5 mesh loader docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_animation_asset_binary_uses_typed_errors() {
    let animation_mod = include_str!("../../../../asset/assets/animation/mod.rs");
    let animation_error = include_str!("../../../../asset/assets/animation/error.rs");
    let binary = include_str!("../../../../asset/assets/animation/binary.rs");
    let channel = include_str!("../../../../asset/assets/animation/channel.rs");
    let clip = include_str!("../../../../asset/assets/animation/clip.rs");
    let graph = include_str!("../../../../asset/assets/animation/graph.rs");
    let reference = include_str!("../../../../asset/assets/animation/reference.rs");
    let sequence = include_str!("../../../../asset/assets/animation/sequence.rs");
    let skeleton = include_str!("../../../../asset/assets/animation/skeleton.rs");
    let state_machine = include_str!("../../../../asset/assets/animation/state_machine.rs");
    let importer_error = include_str!("../../../../asset/importer/error.rs");
    let import_animation_asset =
        include_str!("../../../../asset/importer/ingest/import_animation_asset.rs");
    let animation_tests = include_str!("../../../../asset/tests/assets/animation.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let animation_doc =
        include_str!("../../../../../../docs/zircon_runtime/asset/assets/animation.md");

    for required in [
        "pub type AnimationAssetResult<T> = std::result::Result<T, AnimationAssetError>;",
        "pub enum AnimationAssetError",
        "Serialize {",
        "DocumentDeserialize {",
        "DocumentAndStreamDecode",
        "CurrentAndV1PayloadDecode",
        "InvalidReferenceUuid",
        "InvalidReferenceLocator",
        "KindMismatch",
        "UnknownGraphNodeTag",
    ] {
        assert!(
            animation_error.contains(required),
            "animation asset typed error owner should contain `{required}`"
        );
    }
    assert!(
        animation_mod.contains("pub use error::{AnimationAssetError, AnimationAssetResult};"),
        "animation module should export AnimationAssetError/AnimationAssetResult"
    );

    for (label, source) in [
        ("animation binary", binary),
        ("animation channel", channel),
        ("animation clip", clip),
        ("animation graph", graph),
        ("animation reference", reference),
        ("animation sequence", sequence),
        ("animation skeleton", skeleton),
        ("animation state machine", state_machine),
    ] {
        for forbidden in [
            "Result<Self, String>",
            "Result<Vec<u8>, String>",
            "type Error = String",
            "Err(format!(",
            ".map_err(|error| error.to_string())",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy animation asset error branch `{forbidden}`"
            );
        }
    }

    for required in [
        "AnimationAsset(#[from] AnimationAssetError)",
        ".map_err(AssetImportError::AnimationAsset)",
        "AnimationAssetError::DocumentAndStreamDecode",
        "AnimationAssetError::CurrentAndV1PayloadDecode",
    ] {
        assert!(
            importer_error.contains(required)
                || import_animation_asset.contains(required)
                || animation_tests.contains(required),
            "animation import/test surface should contain `{required}`"
        );
    }

    for doc_anchor in [
        "F5 animation asset binary typed errors",
        "runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred",
        "review_f5_animation_asset_binary_uses_typed_errors",
        "AnimationAssetError::KindMismatch",
        "asset/assets/animation/error.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || animation_doc.contains(doc_anchor),
            "F5 animation asset docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f7_asset_artifact_errors_use_asset_import_error_sources() {
    let importer_error = include_str!("../../../../asset/importer/error.rs");
    let cache_payload = include_str!("../../../../asset/artifact/cache_payload.rs");
    let cache_payload_ui = include_str!("../../../../asset/artifact/cache_payload/ui.rs");
    let toml_value = include_str!("../../../../asset/artifact/cache_payload/toml_value.rs");
    let artifact_store = include_str!("../../../../asset/artifact/store.rs");
    let importer_tests = include_str!("../../../../asset/tests/assets/importer/registry_errors.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_04_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let artifact_doc = include_str!("../../../../../../docs/zircon_runtime/asset/artifact.md");

    for forbidden in [
        "Registry(String)",
        "Self::Registry(error.to_string())",
        "impl From<AssetImporterRegistryError> for AssetImportError",
    ] {
        assert!(
            !importer_error.contains(forbidden),
            "F7 should not preserve lossy registry error conversion `{forbidden}`"
        );
    }
    for required in [
        "Registry(#[from] AssetImporterRegistryError)",
        "TomlSerialize {",
        "TomlDeserialize {",
        "CachedTomlDatetime {",
        "UiDocument {",
        "UiV2Document {",
        "ArtifactCacheSerialize(#[source] bincode::Error)",
        "ArtifactCacheDeserialize(#[source] bincode::Error)",
    ] {
        assert!(
            importer_error.contains(required),
            "F7 AssetImportError should expose typed source anchor `{required}`"
        );
    }

    for forbidden in [
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, String>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, String>",
        "fn into_asset(self) -> Result<MaterialAsset, String>",
        "fn into_asset(self) -> Result<ShaderAsset, String>",
        "fn into_asset(self) -> Result<ShaderMaterialPropertyAsset, String>",
        "format!(\"serialize ui asset document cache",
        "format!(\"deserialize ui layout document cache",
        "format!(\"deserialize ui v2 view document cache",
    ] {
        assert!(
            !cache_payload.contains(forbidden),
            "F7 cache payload should not keep String error/string-format anchor `{forbidden}`"
        );
    }
    for required in [
        "use crate::asset::{",
        "AssetImportError,",
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, AssetImportError>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, AssetImportError>",
        "ArtifactCacheUiAssetDocument::from_document",
        "ArtifactCacheUiV2AssetDocument::from_document",
    ] {
        assert!(
            cache_payload.contains(required),
            "F7 cache payload should use AssetImportError anchor `{required}`"
        );
    }
    for required in [
        "AssetImportError::TomlSerialize",
        "AssetImportError::UiDocument",
        "AssetImportError::UiV2Document",
    ] {
        assert!(
            cache_payload_ui.contains(required),
            "F7 UI cache payload should preserve AssetImportError source anchor `{required}`"
        );
    }

    assert!(
        toml_value.contains("Result<toml::Value, AssetImportError>")
            && toml_value.contains("AssetImportError::CachedTomlDatetime")
            && !toml_value.contains("format!(\"invalid cached TOML datetime"),
        "F7 TOML cache conversion should report typed cached datetime errors"
    );
    for required in [
        "map_err(AssetImportError::ArtifactCacheSerialize)",
        "map_err(AssetImportError::ArtifactCacheDeserialize)",
        "let cache_asset = ArtifactCacheAsset::from_imported(asset)?;",
        "let asset = cache_asset.into_imported()?;",
    ] {
        assert!(
            artifact_store.contains(required),
            "F7 artifact store should preserve typed source anchor `{required}`"
        );
    }
    assert!(
        !artifact_store
            .contains("map_err(|error| AssetImportError::Parse(format!(\"serialize artifact cache")
            && !artifact_store.contains(
                "map_err(|error| AssetImportError::Parse(format!(\"deserialize artifact cache"
            ),
        "F7 artifact store should not lossy-wrap cache conversion sources in Parse(String)"
    );
    assert!(
        importer_tests.contains("asset_import_error_preserves_registry_error_source")
            && importer_tests.contains(
                "AssetImportError::Registry(AssetImporterRegistryError::DuplicateMatcher"
            ),
        "F7 should keep behavior coverage for typed registry error preservation"
    );

    for doc_anchor in [
        "F7 asset artifact/importer typed errors",
        "asset_artifact_importer_typed_errors_coremin_passed",
        "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        "asset_import_error_preserves_registry_error_source",
        "AssetImportError::CachedTomlDatetime",
        "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_04_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || artifact_doc.contains(doc_anchor),
            "F7 docs should record `{doc_anchor}`"
        );
    }
    let f7_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F7 |"))
        .expect("F7 review findings top row");
    assert!(
        f7_row.contains("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred")
            && f7_row.ends_with("| Runtime 04 / review closed |"),
        "F7 top row should record typed-error review closed status"
    );
}
