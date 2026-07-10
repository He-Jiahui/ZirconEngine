use super::*;

#[test]
fn runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let document_dir = source_root.join("scene/dynamic_scene/document");
    let retired_document = document_dir.join("legacy.rs");
    let v1_document = document_dir.join("v1_project_document.rs");

    assert!(
        !retired_document.exists(),
        "dynamic scene document owner should not keep retired legacy-named path {:?}",
        retired_document
    );
    assert!(
        v1_document.exists(),
        "dynamic scene document owner should use explicit v1 schema path {:?}",
        v1_document
    );

    let document_mod = read_text(
        &document_dir.join("mod.rs"),
        "dynamic scene document module entry should be readable",
    );
    let document_read = read_text(
        &document_dir.join("read.rs"),
        "dynamic scene document reader should be readable",
    );
    let document_owner = read_text(
        &v1_document,
        "dynamic scene v1 project document owner should be readable",
    );
    let audit_script = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py",
    );

    assert_contains_all(
        "dynamic scene document module",
        &document_mod,
        &["mod read;", "mod v1_project_document;", "mod write;"],
    );
    assert!(
        !document_mod.contains("mod legacy;"),
        "dynamic scene document module should not preserve retired legacy child"
    );
    assert_contains_all(
        "dynamic scene document reader",
        &document_read,
        &[
            "use super::v1_project_document::V1ProjectDocument;",
            "let document: V1ProjectDocument",
            "Self::from_world(&document.world)",
        ],
    );
    assert!(
        !document_read.contains("LegacyProjectDocument")
            && !document_read.contains("super::legacy"),
        "dynamic scene document reader should not preserve legacy owner references"
    );
    assert_contains_all(
        "dynamic scene v1 project document owner",
        &document_owner,
        &[
            "pub(super) struct V1ProjectDocument",
            "pub(super) world: World",
        ],
    );
    assert!(
        !document_owner.contains("LegacyProjectDocument"),
        "dynamic scene v1 project document owner should not preserve legacy type name"
    );
    assert_contains_all(
        "scene project serialization audit",
        &audit_script,
        &["zircon_runtime/src/scene/dynamic_scene/document/v1_project_document.rs"],
    );
    assert!(
        !audit_script.contains("zircon_runtime/src/scene/dynamic_scene/document/legacy.rs"),
        "scene project serialization audit should not keep retired dynamic scene document path"
    );

    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let dynamic_scene_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/scene/dynamic_scene.md");
    let session_note = read_repo_text(
        manifest_root,
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    );
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("dynamic scene doc", dynamic_scene_doc),
        ("session note", session_note),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                SCENE_DYNAMIC_DOCUMENT_V1_SLICE,
                SCENE_DYNAMIC_DOCUMENT_V1_STATUS,
                "scene/dynamic_scene/document/v1_project_document.rs",
                "V1ProjectDocument",
                SCENE_DYNAMIC_DOCUMENT_V1_GUARD,
            ],
        );
    }
}
