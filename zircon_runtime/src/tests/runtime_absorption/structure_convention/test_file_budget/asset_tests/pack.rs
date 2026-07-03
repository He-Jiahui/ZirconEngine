use super::*;

#[test]
fn runtime_15_asset_pack_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/pack.rs");
    let basic = read_runtime_src("asset/tests/pack/basic.rs");
    let reader_validation = read_runtime_src("asset/tests/pack/reader_validation.rs");
    let delta_reader_validation = read_runtime_src("asset/tests/pack/delta_reader_validation.rs");
    let delta_pack = read_runtime_src("asset/tests/pack/delta_pack.rs");
    let delta_installer = read_runtime_src("asset/tests/pack/delta_installer.rs");
    let trim = read_runtime_src("asset/tests/pack/trim.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );

    assert_contains_all(
        "asset pack parent test module mounts",
        &parent,
        &[
            "mod basic;",
            "mod delta_installer;",
            "mod delta_pack;",
            "mod delta_reader_validation;",
            "mod reader_validation;",
            "mod trim;",
            "fn pack_asset_entry",
            "fn malformed_pack_bytes",
            "fn unique_pack_temp_dir",
        ],
    );

    for moved_test in [
        "fn pack_round_trip",
        "fn pack_reader_rejects_manifest_asset_path_schema",
        "fn delta_reader_rejects_nested_pack_manifest_asset_path_schema",
        "fn delta_pack_contains_only_changed_chunks",
        "fn delta_installer_rebuilds_target_pack_to_staging",
        "fn unreferenced_asset_trimmed_and_reported",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/pack.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/pack.rs should not keep executable tests in the parent module"
    );
    let migrated_test_count = [
        basic.as_str(),
        reader_validation.as_str(),
        delta_reader_validation.as_str(),
        delta_pack.as_str(),
        delta_installer.as_str(),
        trim.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        migrated_test_count, 42,
        "asset pack child modules should preserve the original 42 tests"
    );

    assert_contains_all(
        "asset pack basic child owns writer contracts",
        &basic,
        &[
            "use super::*;",
            "fn pack_round_trip",
            "fn duplicate_content_stored_once",
            "fn pack_writer_rejects_unsafe_asset_paths",
        ],
    );
    assert_contains_all(
        "asset pack reader validation child owns manifest contracts",
        &reader_validation,
        &[
            "use super::*;",
            "fn pack_reader_rejects_manifest_asset_path_schema",
            "fn pack_reader_rejects_manifest_asset_chunk_mismatch",
            "fn pack_reader_rejects_manifest_trailing_bytes",
        ],
    );
    assert_contains_all(
        "asset pack delta reader validation child owns delta manifest contracts",
        &delta_reader_validation,
        &[
            "use super::*;",
            "fn delta_reader_rejects_nested_pack_manifest_asset_path_schema",
            "fn delta_reader_rejects_delta_chunk_table_mismatch",
            "fn delta_reader_rejects_manifest_trailing_bytes",
        ],
    );
    assert_contains_all(
        "asset pack delta child owns delta apply contracts",
        &delta_pack,
        &[
            "use super::*;",
            "fn delta_pack_contains_only_changed_chunks",
            "fn delta_pack_applies_to_base_pack",
            "fn delta_pack_rejects_wrong_base_manifest",
        ],
    );
    assert_contains_all(
        "asset pack delta installer child owns install contracts",
        &delta_installer,
        &[
            "use super::*;",
            "fn delta_installer_rebuilds_target_pack_to_staging",
            "fn delta_installer_writes_install_receipt_from_staging_and_promotion",
            "fn delta_installer_rejects_receipt_for_mismatched_reports",
        ],
    );
    assert_contains_all(
        "asset pack trim child owns trimming contracts",
        &trim,
        &[
            "use super::*;",
            "fn unreferenced_asset_trimmed_and_reported",
            "fn asset_filter_trim_is_reported",
            "fn duplicate_trim_input_path_is_reported",
        ],
    );

    for (path, source) in [
        ("asset/tests/pack.rs", parent.as_str()),
        ("asset/tests/pack/basic.rs", basic.as_str()),
        (
            "asset/tests/pack/reader_validation.rs",
            reader_validation.as_str(),
        ),
        (
            "asset/tests/pack/delta_reader_validation.rs",
            delta_reader_validation.as_str(),
        ),
        ("asset/tests/pack/delta_pack.rs", delta_pack.as_str()),
        (
            "asset/tests/pack/delta_installer.rs",
            delta_installer.as_str(),
        ),
        ("asset/tests/pack/trim.rs", trim.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset pack test folder split",
                "runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked",
                "asset/tests/pack.rs",
                "asset/tests/pack/delta_installer.rs",
                "runtime_15_asset_pack_tests_are_folder_backed",
            ],
        );
    }
}

#[test]
fn runtime_15_asset_pack_header_readers_are_panic_free() {
    let reader = read_runtime_src("asset/pack/reader.rs");
    let delta = read_runtime_src("asset/pack/delta.rs");
    let pack_doc = read_repo("docs/zircon_runtime/asset/pack.md");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/asset_tests.rs",
    );

    assert_contains_all(
        "asset pack reader owns typed header helpers",
        &reader,
        &[
            "pub(crate) fn read_header_u32(bytes: &[u8], offset: usize) -> Result<u32, ZrPackError>",
            "pub(crate) fn read_header_u64(bytes: &[u8], offset: usize) -> Result<u64, ZrPackError>",
            "fn read_header_bytes<const N: usize>",
            "ok_or(ZrPackError::HeaderTooSmall)",
        ],
    );
    assert_contains_all(
        "asset pack delta reuses typed header helpers",
        &delta,
        &[
            "reader::{read_header_u32, read_header_u64, validate_chunk_payload_extent}",
            "let version = read_header_u32(bytes, 4)?;",
            "let manifest_offset = read_header_u64(bytes, 8)? as usize;",
            "let manifest_size = read_header_u64(bytes, 16)? as usize;",
        ],
    );
    for (label, source) in [("reader", reader.as_str()), ("delta", delta.as_str())] {
        for forbidden in [
            "expect(\"header version bytes\")",
            "expect(\"header offset bytes\")",
            "expect(\"header size bytes\")",
            "try_into().expect(\"header",
            "try_into().unwrap()",
        ] {
            assert!(
                !source.contains(forbidden),
                "asset pack {label} should not rely on panic-based header conversion `{forbidden}`"
            );
        }
    }

    for (label, source) in [
        ("pack doc", pack_doc.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 asset pack panic-free header readers",
                "runtime_15_asset_pack_header_readers_panic_free_static_passed_cargo_deferred",
                "read_header_u64",
                "runtime_15_asset_pack_header_readers_are_panic_free",
            ],
        );
    }
}
