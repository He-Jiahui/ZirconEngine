use super::super::support::{
    assert_contains_all, first_backtick_value, index_section_between, leading_plan_id,
    markdown_table_cells, runtime_subplan_sources,
};

fn runtime_15_status_sources() -> [(&'static str, &'static str); 8] {
    [
        (
            "Runtime 15 subplan",
            include_str!(
                "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
            ),
        ),
        (
            "engine code structure convention",
            include_str!("../../../../../../docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "engine code review findings",
            include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "module convention doc",
            include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "runtime implementation session note",
            include_str!(
                "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
            ),
        ),
        (
            "Runtime 15 status row data",
            include_str!(
                "../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/index_baseline.rs"
            ),
        ),
        (
            "Runtime 15 expected status map",
            include_str!(
                "../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/index_baseline_maps.rs"
            ),
        ),
        (
            "Runtime 15 expected date map",
            include_str!(
                "../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/index_baseline_maps.rs"
            ),
        ),
    ]
}

#[test]
fn runtime_index_subplan_map_covers_existing_plan_files_without_stale_rows() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let subplan_filenames: Vec<_> = runtime_subplan_sources()
        .into_iter()
        .map(|(filename, _)| filename)
        .collect();
    let subplan_section = index_section_between(
        index_source,
        "## 3. 子计划地图与执行顺序",
        "### 3.1 已知但暂不立项的缺口",
    );
    let mut mapped_filenames = Vec::new();

    for line in subplan_section.lines() {
        let cells = markdown_table_cells(line);
        if cells.len() == 4
            && cells[0] != "计划"
            && !cells[0].chars().all(|character| character == '-')
        {
            let filename = first_backtick_value(cells[1])
                .unwrap_or_else(|| panic!("runtime index subplan row `{line}` should link a file"));
            let plan_id = leading_plan_id(filename).unwrap_or_else(|| {
                panic!("runtime subplan filename `{filename}` should start with a two-digit id")
            });
            assert!(
                cells[0].starts_with(plan_id),
                "runtime index subplan row `{}` should use plan id `{plan_id}` from `{filename}`",
                cells[0]
            );
            assert!(
                subplan_filenames.iter().any(|subplan| subplan == filename),
                "runtime index subplan row should not reference stale file `{filename}`"
            );
            mapped_filenames.push(filename.to_owned());
        }
    }

    mapped_filenames.sort();
    assert_eq!(
        mapped_filenames, subplan_filenames,
        "runtime index subplan map should exactly cover runtime 01-15 subplan files"
    );
}

#[test]
fn runtime_15_runtime_index_subplan_map_covers_01_15_status_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let boundary_script = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py"
    );

    let subplan_section = index_section_between(
        index_source,
        "## 3. 子计划地图与执行顺序",
        "### 3.1 已知但暂不立项的缺口",
    );
    let mut mapped_ids = Vec::new();

    for line in subplan_section.lines() {
        let cells = markdown_table_cells(line);
        if cells.len() == 4
            && cells[0] != "计划"
            && !cells[0].chars().all(|character| character == '-')
        {
            let filename = first_backtick_value(cells[1])
                .unwrap_or_else(|| panic!("runtime index subplan row `{line}` should link a file"));
            let plan_id = leading_plan_id(filename).unwrap_or_else(|| {
                panic!("runtime subplan filename `{filename}` should start with a two-digit id")
            });
            mapped_ids.push(plan_id.to_owned());
        }
    }

    assert_eq!(
        mapped_ids,
        (1..=15).map(|id| format!("{id:02}")).collect::<Vec<_>>(),
        "runtime index subplan map should keep contiguous runtime 01-15 rows"
    );

    let status_anchors = [
        "Runtime 15 M3 runtime index subplan map 01-15 sync",
        "runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_index_subplan_map_covers_01_15_status_locked",
        "14-runtime-module-family-closeout.md",
        "15-code-structure-and-module-conventions.md",
        "EXPECTED_SUBPLAN_COUNT = 15",
    ];
    for (label, source) in runtime_15_status_sources() {
        assert_contains_all(label, source, &status_anchors[..3]);
    }
    assert_contains_all(
        "runtime index",
        index_source,
        &[
            "`14-runtime-module-family-closeout.md`",
            "`15-code-structure-and-module-conventions.md`",
            "Runtime 14",
            "Runtime 15",
            status_anchors[0],
            status_anchors[1],
        ],
    );
    assert_contains_all(
        "runtime plan-status boundary audit",
        boundary_script,
        &status_anchors[5..],
    );
}

#[test]
fn runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let boundary_sources = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_sources.py"
    );
    let boundary_audit = include_str!(
        "../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py"
    );

    let problem_section = index_section_between(index_source, "### 2.2 问题清单", "### 2.3");
    let observed_problem_ids: Vec<_> = problem_section
        .lines()
        .filter_map(|line| {
            let cells = markdown_table_cells(line);
            (cells.len() == 4
                && cells[0].starts_with('P')
                && cells[0][1..]
                    .chars()
                    .all(|character| character.is_ascii_digit()))
            .then(|| cells[0].to_owned())
        })
        .collect();

    assert_eq!(
        observed_problem_ids,
        (1..=17)
            .map(|problem_id| format!("P{problem_id}"))
            .collect::<Vec<_>>(),
        "runtime index problem-row parser should count aligned P1-P17 rows, not just P10+ rows"
    );

    let status_anchors = [
        "Runtime 15 M3 runtime index problem-row parser P01-P17 sync",
        "runtime_15_runtime_index_problem_row_parser_p01_p17_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_index_problem_row_parser_covers_p01_p17_status_locked",
    ];
    for (label, source) in runtime_15_status_sources() {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "runtime index",
        index_source,
        &[
            "P1",
            "P17",
            "Runtime 15 M3 runtime index problem-row parser P01-P17 sync",
            "runtime_15_runtime_index_problem_row_parser_p01_p17_sync_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "runtime plan-status source parser",
        boundary_sources,
        &[
            "def runtime_index_problem_rows",
            "markdown_table_cells(line)",
            "cells[0].startswith(\"P\")",
        ],
    );
    assert_contains_all(
        "runtime plan-status boundary audit",
        boundary_audit,
        &["EXPECTED_PROBLEM_ROW_COUNT = 17"],
    );
}
