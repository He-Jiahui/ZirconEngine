use super::*;

pub(super) fn assert_pre_runtime_15_maps(sources: &TopLevelMapSources) {
    assert_contains_all(
        "pre-Runtime-15 status expected-slice parent delegates legacy status literals",
        &sources.status_pre_runtime_15,
        &[
            "pub(super) fn expected_status_for_slice(slice: &str) -> &'static str",
            "#[path = \"pre_runtime_15/runtime_01_05.rs\"]",
            "mod runtime_01_05;",
            "#[path = \"pre_runtime_15/runtime_06_10.rs\"]",
            "mod runtime_06_10;",
            "#[path = \"pre_runtime_15/runtime_11_14.rs\"]",
            "mod runtime_11_14;",
            "runtime_01_05::expected_status_for_slice(slice)",
            "runtime_06_10::expected_status_for_slice(slice)",
            "runtime_11_14::expected_status_for_slice(slice)",
            "mirror_docs_static_passed_cargo_pending",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 status expected-slice children own legacy status literals",
        &format!(
            "{}\n{}\n{}",
            sources.status_pre_runtime_15_runtime_01_05,
            sources.status_pre_runtime_15_runtime_06_10,
            sources.status_pre_runtime_15_runtime_11_14
        ),
        &[
            "pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str>",
            "Runtime 14 Cargo 验证窗口探测",
            "Runtime 05 plan-status Cargo attempt 状态审计",
            "Runtime 11 full-lib default after graphics exposure retry",
            "Runtime 12 input boundary grouped manager import guard repair",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 date expected-slice parent delegates legacy date literals",
        &sources.date_pre_runtime_15,
        &[
            "pub(super) fn expected_date_for_slice(slice: &str) -> &'static str",
            "#[path = \"pre_runtime_15/runtime_01_05.rs\"]",
            "mod runtime_01_05;",
            "#[path = \"pre_runtime_15/runtime_06_10.rs\"]",
            "mod runtime_06_10;",
            "#[path = \"pre_runtime_15/runtime_11_14.rs\"]",
            "mod runtime_11_14;",
            "runtime_01_05::expected_date_for_slice(slice)",
            "runtime_06_10::expected_date_for_slice(slice)",
            "runtime_11_14::expected_date_for_slice(slice)",
            "2026-06-14",
        ],
    );
    assert_contains_all(
        "pre-Runtime-15 date expected-slice children own legacy date literals",
        &format!(
            "{}\n{}\n{}",
            sources.date_pre_runtime_15_runtime_01_05,
            sources.date_pre_runtime_15_runtime_06_10,
            sources.date_pre_runtime_15_runtime_11_14
        ),
        &[
            "pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str>",
            "Runtime 10 F18 asset manager resolution return shape",
            "Runtime 14 animation runtime-status focused recheck timeout",
            "Runtime 11 full-lib default after graphics exposure retry",
            "Runtime 12 input boundary grouped manager import guard repair",
        ],
    );
}
