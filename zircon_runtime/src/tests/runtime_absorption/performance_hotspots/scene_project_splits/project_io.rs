#[test]
fn runtime_07_project_io_folder_split_keeps_entry_and_converter_owners() {
    let project_io_root = include_str!("../../../../scene/world/project_io.rs");
    let camera = include_str!("../../../../scene/world/project_io/camera.rs");
    let physics = include_str!("../../../../scene/world/project_io/physics.rs");
    let post_process = include_str!("../../../../scene/world/project_io/post_process.rs");
    let references = include_str!("../../../../scene/world/project_io/references.rs");
    let script = include_str!("../../../../scene/world/project_io/script.rs");
    let transform = include_str!("../../../../scene/world/project_io/transform.rs");
    let project_io_doc =
        include_str!("../../../../../../docs/zircon_runtime/scene/world/project_io.md");
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let large_file_doc =
        include_str!("../../../../../../docs/engine-architecture/large-file-ownership-m1.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");

    for root_anchor in [
        "mod camera;",
        "mod physics;",
        "mod post_process;",
        "mod references;",
        "mod script;",
        "mod transform;",
        "pub fn from_scene_asset",
        "pub fn to_scene_asset",
    ] {
        assert!(
            project_io_root.contains(root_anchor),
            "project_io.rs should keep entry orchestration anchor `{root_anchor}`"
        );
    }

    for moved_helper in [
        "fn camera_target_from_asset",
        "fn collider_shape_from_asset",
        "fn post_process_settings_from_asset",
        "fn model_handle_for_reference",
        "fn script_bindings_for_record",
        "fn transform_from_asset",
    ] {
        assert!(
            !project_io_root.contains(moved_helper),
            "project_io.rs should not reclaim converter helper `{moved_helper}`"
        );
    }

    for (module_name, module_source, expected_anchor) in [
        ("camera", camera, "pub(super) fn camera_to_asset"),
        ("physics", physics, "pub(super) fn collider_shape_to_asset"),
        (
            "post_process",
            post_process,
            "pub(super) fn post_process_volume_to_asset",
        ),
        (
            "references",
            references,
            "pub(super) fn reference_for_model_handle",
        ),
        ("script", script, "pub(super) fn script_bindings_for_record"),
        ("transform", transform, "pub(super) fn transform_to_asset"),
    ] {
        assert!(
            module_source.contains(expected_anchor),
            "project_io/{module_name}.rs should own `{expected_anchor}`"
        );
    }

    for doc_anchor in [
        "Project I/O Folder Split",
        "project_io/{camera,physics,post_process,references,script,transform}.rs",
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
        "project_io.rs 772 行",
        "project_io folder split",
    ] {
        assert!(
            project_io_doc.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || large_file_doc.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor),
            "Project I/O split docs should retain `{doc_anchor}`"
        );
    }
}
