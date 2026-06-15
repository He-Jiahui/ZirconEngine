use std::collections::BTreeSet;

use super::support::{markdown_table_cells, runtime_subplan_sources};

#[path = "status_output_tables/expected_slices.rs"]
mod expected_slices;
#[path = "status_output_tables/expected_status_rows.rs"]
mod expected_status_rows;

const STATUS_OUTPUT_HEADING: &str = "## 状态与产出记录";
const STATUS_OUTPUT_HEADER: &str = "| 里程碑 | 切片 | 状态 |";

#[test]
fn runtime_plan_status_output_tables_cover_index_and_all_subplans() {
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");

    assert_status_output_table("runtime index", runtime_index);
    for (filename, source) in runtime_subplan_sources() {
        assert_status_output_table(&filename, &source);
    }
}

#[test]
fn runtime_index_status_output_records_recent_cross_plan_slices() {
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_rows = status_output_rows(runtime_index);
    let expected_status_output_slices =
        expected_status_rows::expected_status_output_slices().collect::<Vec<_>>();

    let guarded_slices = expected_status_output_slices
        .iter()
        .map(|(slice, _)| *slice)
        .collect::<BTreeSet<_>>();
    for row in &status_rows {
        let slice = row[1];
        assert!(
            guarded_slices.contains(slice),
            "runtime index status row `{slice}` should be covered by status-output guard expectations"
        );
    }

    for (slice, required_anchors) in expected_status_output_slices {
        let row = status_rows
            .iter()
            .find(|cells| cells.get(1) == Some(&slice))
            .unwrap_or_else(|| panic!("runtime index status table should record `{slice}`"));
        let expected_status = expected_slices::expected_status_for_slice(slice);
        assert_eq!(
            row[2], expected_status,
            "runtime index status row `{slice}` should keep the expected pending-Cargo status"
        );
        let expected_date = expected_slices::expected_date_for_slice(slice);
        assert_eq!(
            row[3], expected_date,
            "runtime index status row `{slice}` should keep the slice completion date"
        );
        for anchor in required_anchors {
            assert!(
                row[4].contains(anchor),
                "runtime index status row `{slice}` should keep evidence anchor `{anchor}`"
            );
        }
    }
}

fn assert_status_output_table(label: &str, source: &str) {
    assert!(
        source.contains(STATUS_OUTPUT_HEADING),
        "{label} should keep `{STATUS_OUTPUT_HEADING}`"
    );
    assert!(
        source.contains(STATUS_OUTPUT_HEADER),
        "{label} should keep status/output table header `{STATUS_OUTPUT_HEADER}`"
    );
    assert!(
        !status_output_rows(source).is_empty(),
        "{label} should keep at least one status/output record"
    );
}

fn status_output_rows(source: &str) -> Vec<Vec<&str>> {
    let mut status_section_started = false;
    let mut rows = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == STATUS_OUTPUT_HEADING {
            status_section_started = true;
            continue;
        }
        if !status_section_started {
            continue;
        }
        if trimmed.starts_with("## ") {
            break;
        }

        let cells = markdown_table_cells(line);
        if cells.len() == 5
            && cells[0] != "里程碑"
            && !cells[0].chars().all(|character| character == '-')
        {
            rows.push(cells);
        }
    }

    assert!(
        status_section_started,
        "document should contain `{STATUS_OUTPUT_HEADING}` as a heading line"
    );

    rows
}
