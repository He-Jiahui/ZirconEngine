use super::*;

#[test]
fn runtime_index_problem_rows_reference_existing_subplans() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let subplan_ids: Vec<_> = runtime_subplan_sources()
        .into_iter()
        .map(|(filename, _)| {
            leading_plan_id(&filename)
                .unwrap_or_else(|| {
                    panic!("runtime subplan filename `{filename}` should start with a two-digit id")
                })
                .to_owned()
        })
        .collect();
    let problem_section = index_section_between(index_source, "### 2.2 问题清单", "### 2.3");
    let mut observed_problem_ids = Vec::new();

    for line in problem_section.lines() {
        let cells = markdown_table_cells(line);
        if cells.len() == 4
            && cells[0] != "#"
            && !cells[0].chars().all(|character| character == '-')
        {
            assert!(
                cells[0].starts_with('P'),
                "runtime problem row `{line}` should keep a P-number"
            );
            let plan_id = leading_plan_id(cells[3]).unwrap_or_else(|| {
                panic!(
                    "runtime problem row `{}` should point at a two-digit runtime subplan",
                    cells[0]
                )
            });
            assert!(
                subplan_ids.iter().any(|subplan| subplan == plan_id),
                "runtime problem row `{}` points at missing subplan `{plan_id}`",
                cells[0]
            );
            observed_problem_ids.push(cells[0].to_owned());
        }
    }

    assert!(
        observed_problem_ids.len() >= subplan_ids.len(),
        "runtime problem table should keep at least one problem/status row per subplan family"
    );
    assert!(
        observed_problem_ids
            .windows(2)
            .all(|pair| pair[0] != pair[1]),
        "runtime problem table should not duplicate adjacent problem ids"
    );
}

#[test]
fn runtime_index_execution_dependencies_reference_existing_subplans() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let subplan_ids: Vec<_> = runtime_subplan_sources()
        .into_iter()
        .map(|(filename, _)| {
            leading_plan_id(&filename)
                .unwrap_or_else(|| {
                    panic!("runtime subplan filename `{filename}` should start with a two-digit id")
                })
                .to_owned()
        })
        .collect();
    let subplan_section = index_section_between(
        index_source,
        "## 3. 子计划地图与执行顺序",
        "### 3.1 已知但暂不立项的缺口",
    );
    let mut checked_dependency_rows = 0usize;

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
            let dependency_cell = cells[2];
            let dependency_ids = referenced_plan_ids(dependency_cell);
            if dependency_cell != "无" && !dependency_cell.contains("无（") {
                assert!(
                    !dependency_ids.is_empty(),
                    "runtime index dependency cell `{dependency_cell}` for plan `{plan_id}` should reference an existing subplan id or explicitly say `无`"
                );
            }
            for dependency_id in dependency_ids {
                assert!(
                    subplan_ids.iter().any(|subplan| subplan == dependency_id),
                    "runtime index dependency cell `{dependency_cell}` for plan `{plan_id}` references missing subplan `{dependency_id}`"
                );
            }
            checked_dependency_rows += 1;
        }
    }

    assert_eq!(
        checked_dependency_rows,
        subplan_ids.len(),
        "runtime index execution order should declare one dependency cell per runtime subplan"
    );
}

#[test]
fn runtime_index_status_map_matches_subplan_frontmatter() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");

    for (filename, source) in runtime_subplan_sources() {
        let status = frontmatter_status(&source)
            .unwrap_or_else(|| panic!("runtime plan {filename} should keep a frontmatter status"));
        let row = runtime_index_row_for(index_source, &filename);
        let expected_status_cell = format!("| {status}");
        assert!(
            row.contains(&expected_status_cell),
            "runtime index row for `{filename}` should mirror frontmatter status `{status}`"
        );
    }
}

#[test]
fn runtime_index_in_progress_rows_record_remaining_gate() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let remaining_gate_markers = [
        "Cargo",
        "cargo",
        "待",
        "pending",
        "owner",
        "active lane",
        "阻塞",
        "超时",
        "窗口",
    ];

    for (filename, source) in runtime_subplan_sources() {
        let status = frontmatter_status(&source)
            .unwrap_or_else(|| panic!("runtime plan {filename} should keep a frontmatter status"));
        if status != "in_progress" {
            continue;
        }

        let row = runtime_index_row_for(index_source, &filename);
        assert!(
            remaining_gate_markers
                .iter()
                .any(|marker| row.contains(marker)),
            "runtime index row for in-progress plan `{filename}` should record the remaining gate or blocker"
        );
    }
}

#[test]
fn runtime_known_backlog_gaps_keep_owner_and_trigger_columns() {
    let index_source = include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let backlog_start = index_source
        .find("### 3.1 已知但暂不立项的缺口")
        .expect("runtime index should keep the known-backlog section");
    let backlog_section = &index_source[backlog_start..];
    let backlog_section = backlog_section
        .split_once("阶段划分:")
        .map(|(section, _)| section)
        .expect("runtime index known-backlog section should end before the phase split");
    let mut backlog_rows = Vec::new();

    for line in backlog_section.lines() {
        let cells = markdown_table_cells(line);
        if cells.len() == 3
            && cells[0] != "缺口"
            && !cells[0].chars().all(|character| character == '-')
        {
            backlog_rows.push(cells);
        }
    }

    for expected_gap in [
        "网络复制 runtime 侧",
        "音频 runtime 服务面",
        r#"FFI panic 安全（extern "C" 边界 catch_unwind 审计）"#,
        "输入录制/回放",
        "脚本调试器/断点面",
        "存档/会话持久化语义",
        "本地化/i18n",
    ] {
        assert!(
            backlog_rows.iter().any(|cells| cells[0] == expected_gap),
            "runtime index backlog should keep known gap `{expected_gap}`"
        );
    }

    for cells in backlog_rows {
        assert!(
            !cells[1].is_empty() && !cells[2].is_empty(),
            "runtime backlog gap `{}` should keep evidence and owner/trigger cells",
            cells[0]
        );
        assert!(
            ["owner", "backlog", "归 ", "触发", "立项", "需求", "稳定"]
                .iter()
                .any(|marker| cells[2].contains(marker)),
            "runtime backlog gap `{}` should name an owner or trigger condition",
            cells[0]
        );
    }
}
