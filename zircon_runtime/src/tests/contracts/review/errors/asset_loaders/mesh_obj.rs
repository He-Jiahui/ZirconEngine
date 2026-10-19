//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f5_mesh_loader_and_obj_decoder_use_typed_errors() {
    let mesh_loader = include_str!("../../../../../asset/load/mesh.rs");
    let obj_error = include_str!("../../../../../asset/formats/obj/error.rs");
    let obj_mod = include_str!("../../../../../asset/formats/obj/mod.rs");
    let obj_decode = include_str!("../../../../../asset/formats/obj/decode_obj_file.rs");
    let obj_face = include_str!("../../../../../asset/formats/obj/parse_obj_face_vertex.rs");
    let obj_scalar = include_str!("../../../../../asset/formats/obj/parse_obj_scalar.rs");
    let obj_index = include_str!("../../../../../asset/formats/obj/resolve_obj_index.rs");
    let worker_pool = include_str!("../../../../../asset/pipeline/worker_pool.rs");
    let mesh_tests = include_str!("../../../../../asset/tests/load/mesh.rs");
    let obj_tests = include_str!("../../../../../asset/tests/formats/obj.rs");
    let review_findings = crate::runtime_test_history_concat!(
        include_str!("../../../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/crates/zircon_runtime/structure/module-convention.md"
    );
    let runtime_index = include_str!(
        "../../../../../../../docs/crates/zircon_runtime/structure/runtime-absorption-evidence.md"
    );
    let convention =
        include_str!("../../../../../../../docs/testing-and-validation/development-conventions.md");
    let worker_doc =
        include_str!("../../../../../../../docs/crates/zircon_runtime/asset/worker_pool.md");

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
            !production_section(mesh_loader).contains(forbidden),
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
                !production_section(source).contains(forbidden),
                "{label} should not keep lossy String error branch `{forbidden}`"
            );
        }
    }

    let worker_payload = include_str!("../../../../../asset/pipeline/worker_pool/payload.rs");
    assert!(
        worker_pool
            .replace("\r\n", "\n")
            .contains("#[path = \"worker_pool/payload.rs\"]\nmod payload;"),
        "exact current module mount in asset/pipeline/worker_pool.rs"
    );
    let mesh_branch = worker_payload
        .split("AssetRequest::Mesh(source) => match mesh::load_mesh(&source)")
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
