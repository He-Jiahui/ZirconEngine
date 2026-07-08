use super::super::*;
use super::super::{guard_names::GuardNames, sources::GuardSources};

pub(super) fn assert_asset_children(sources: &GuardSources, guards: &GuardNames) {
    assert_contains_all(
        "asset test-budget child owns child-owner mounts",
        &sources.asset_tests,
        &[
            "use super::*;",
            "mod facade;",
            "mod material;",
            "mod pack;",
            "mod project;",
            "fn runtime_15_asset_test_budget_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "asset pack test-budget child owns pack guard",
        &sources.asset_test_pack,
        &["use super::*;", guards.asset_pack_guard.as_str()],
    );
    assert_contains_all(
        "asset facade test-budget child owns facade guard",
        &sources.asset_test_facade,
        &["use super::*;", guards.asset_facade_guard.as_str()],
    );
    assert_contains_all(
        "asset project test-budget child owns project guards",
        &sources.asset_test_project,
        &[
            "use super::*;",
            guards.asset_project_zmeta_guard.as_str(),
            guards.asset_project_manager_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset material test-budget child owns material guard",
        &sources.asset_test_material,
        &["use super::*;", guards.asset_material_guard.as_str()],
    );
    assert_contains_all(
        "asset glTF importer test-budget child owns glTF importer guard",
        &sources.asset_gltf_importer,
        &["use super::*;", guards.asset_gltf_importer_guard.as_str()],
    );
    assert_contains_all(
        "asset glTF primitive fixture test-budget child owns fixture guard",
        &sources.asset_gltf_primitive_fixtures,
        &[
            "use super::*;",
            guards.asset_gltf_primitive_fixtures_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset importer test-budget child owns importer guard",
        &sources.asset_importer,
        &["use super::*;", guards.asset_importer_guard.as_str()],
    );
    assert_contains_all(
        "asset project flow sample test-budget child owns project flow guard",
        &sources.asset_project_flow_sample,
        &[
            "use super::*;",
            guards.asset_project_flow_sample_guard.as_str(),
        ],
    );
    assert_contains_all(
        "asset scene test-budget child owns scene guard",
        &sources.asset_scene,
        &["use super::*;", guards.asset_scene_guard.as_str()],
    );
}
