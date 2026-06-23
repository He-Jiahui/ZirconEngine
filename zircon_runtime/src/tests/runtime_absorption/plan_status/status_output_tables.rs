use super::support::{markdown_table_cells, runtime_subplan_sources};

const STATUS_OUTPUT_HEADING: &str = "## 状态与产出记录";
const STATUS_OUTPUT_HEADER: &str = "| 里程碑 | 切片 | 状态 |";
const INDEX_STATUS_ROUTING_HEADER: &str = "| 范围 | 记录位置 |";

#[test]
fn runtime_plan_status_output_tables_cover_all_subplans() {
    for (filename, source) in runtime_subplan_sources() {
        assert_status_output_table(&filename, &source);
    }
}

#[test]
fn runtime_index_status_section_routes_to_subplans_without_detail_table() {
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");

    assert!(
        runtime_index.contains(STATUS_OUTPUT_HEADING),
        "runtime index should keep `{STATUS_OUTPUT_HEADING}` as a routing section"
    );
    assert!(
        runtime_index.contains(INDEX_STATUS_ROUTING_HEADER),
        "runtime index status section should route readers to subplan status tables"
    );
    assert!(
        !runtime_index.contains(STATUS_OUTPUT_HEADER),
        "runtime index should not duplicate the slice-level status/output table"
    );
    for (filename, _) in runtime_subplan_sources() {
        assert!(
            runtime_index.contains(&format!("`{filename}`")),
            "runtime index status routing should point to `{filename}`"
        );
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
