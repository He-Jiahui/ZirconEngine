use super::support::{
    frontmatter_last_refined, frontmatter_status, markdown_table_cells, max_iso_date,
    runtime_subplan_sources,
};

#[test]
fn runtime_plan_last_refined_covers_latest_recorded_date() {
    for (filename, source) in runtime_subplan_sources() {
        let last_refined = frontmatter_last_refined(&source).unwrap_or_else(|| {
            panic!("runtime plan {filename} should keep a last_refined frontmatter date")
        });
        if let Some(max_date) = max_iso_date(&source) {
            assert!(
                last_refined >= max_date,
                "runtime plan {filename} has last_refined `{last_refined}` before latest recorded date `{max_date}`"
            );
        }
    }
}

#[test]
fn runtime_plan_status_does_not_claim_completed_while_validation_is_pending() {
    let pending_markers = [
        "code_static_pending_cargo",
        "pending_full_scene_cargo",
        "Cargo 待",
        "Cargo待",
        "待 active",
        "待活动",
        "待编译通道",
        "待共享测试窗口",
        "待跑",
        "待重跑",
    ];

    for (filename, source) in runtime_subplan_sources() {
        let status = frontmatter_status(&source)
            .unwrap_or_else(|| panic!("runtime plan {filename} should keep a frontmatter status"));

        if status == "completed" {
            for marker in pending_markers {
                assert!(
                    !source.contains(marker),
                    "runtime plan {filename} is marked completed but still contains pending validation marker `{marker}`"
                );
            }
        }
    }
}

#[test]
fn runtime_plan_frontmatter_status_uses_known_lifecycle_values() {
    for (filename, source) in runtime_subplan_sources() {
        let status = frontmatter_status(&source)
            .unwrap_or_else(|| panic!("runtime plan {filename} should keep a frontmatter status"));
        assert!(
            matches!(status, "planned" | "in_progress" | "completed"),
            "runtime plan {filename} uses unknown frontmatter status `{status}`"
        );
    }
}

#[test]
fn runtime_subplans_keep_status_and_evidence_tables() {
    for (filename, source) in runtime_subplan_sources() {
        for required_anchor in ["## 状态与产出记录", "| 里程碑 | 切片 | 状态 |"] {
            assert!(
                source.contains(required_anchor),
                "runtime plan {filename} should keep the status/evidence anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_subplan_status_records_keep_non_empty_evidence() {
    for (filename, source) in runtime_subplan_sources() {
        let status_start = source
            .find("## 状态与产出记录")
            .unwrap_or_else(|| panic!("runtime plan {filename} should keep a status section"));
        let status_section = &source[status_start..];
        let status_section = status_section
            .find("\n## ")
            .map(|next_heading| &status_section[..next_heading])
            .unwrap_or(status_section);
        let mut record_rows = Vec::new();

        for line in status_section.lines() {
            let cells = markdown_table_cells(line);
            if cells.len() == 5
                && cells[0] != "里程碑"
                && !cells[0].chars().all(|character| character == '-')
            {
                record_rows.push(cells);
            }
        }

        assert!(
            !record_rows.is_empty(),
            "runtime plan {filename} should keep at least one status/evidence record"
        );
        for cells in record_rows {
            for (index, label) in [
                "里程碑",
                "切片",
                "状态",
                "完成日期",
                "证据（命令输出 / 文件 / 测试名）",
            ]
            .iter()
            .enumerate()
            {
                assert!(
                    !cells[index].is_empty() && cells[index] != "TBD" && cells[index] != "待定",
                    "runtime plan {filename} status row `{}` should keep non-placeholder `{label}`",
                    cells[1]
                );
            }
            if cells[2] != "待开始" {
                assert!(
                    cells[4].len() > 6 && cells[4] != "无" && cells[4] != "—",
                    "runtime plan {filename} status row `{}` should record concrete evidence once it has started",
                    cells[1]
                );
            }
            if cells[2].contains("完成") {
                assert!(
                    max_iso_date(cells[3]).is_some(),
                    "runtime plan {filename} completed status row `{}` should record an ISO completion date",
                    cells[1]
                );
            }
        }
    }
}
