fn frontmatter_status(source: &str) -> Option<&str> {
    frontmatter_value(source, "status:")
}

fn frontmatter_last_refined(source: &str) -> Option<&str> {
    frontmatter_value(source, "last_refined:")
}

fn frontmatter_value<'a>(source: &'a str, prefix: &str) -> Option<&'a str> {
    let mut lines = source.lines();
    if lines.next() != Some("---") {
        return None;
    }

    for line in lines {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix(prefix) {
            return Some(value.trim());
        }
    }

    None
}

fn runtime_plan_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root")
        .join("docs")
        .join("plans")
        .join("zircon_runtime")
        .join("runtime")
}

fn runtime_subplan_sources() -> Vec<(String, String)> {
    let plan_dir = runtime_plan_dir();
    let mut sources = Vec::new();

    for entry in std::fs::read_dir(&plan_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", plan_dir.display()))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("failed to read runtime plan entry: {error}"));
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("index.md")
        {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| panic!("runtime plan path should be utf-8: {}", path.display()))
            .to_owned();
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        sources.push((filename, source));
    }

    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources
}

fn max_iso_date(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut max_date = None;

    for start in 0..bytes.len().saturating_sub(9) {
        let candidate_bytes = &bytes[start..start + 10];
        let is_iso_date = candidate_bytes[0..4].iter().all(u8::is_ascii_digit)
            && candidate_bytes[4] == b'-'
            && candidate_bytes[5..7].iter().all(u8::is_ascii_digit)
            && candidate_bytes[7] == b'-'
            && candidate_bytes[8..10].iter().all(u8::is_ascii_digit);
        if is_iso_date {
            let candidate = std::str::from_utf8(candidate_bytes)
                .expect("ASCII date candidate should be valid utf-8");
            if match max_date {
                Some(date) => candidate > date,
                None => true,
            } {
                max_date = Some(candidate);
            }
        }
    }

    max_date
}

fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    for anchor in anchors {
        assert!(
            source.contains(anchor),
            "{label} should keep runtime plan-status anchor `{anchor}`"
        );
    }
}

fn runtime_index_row_for<'a>(index_source: &'a str, filename: &str) -> &'a str {
    index_source
        .lines()
        .find(|line| line.contains(filename))
        .unwrap_or_else(|| panic!("runtime index should include subplan row for `{filename}`"))
}

fn index_section_between<'a>(source: &'a str, start_anchor: &str, end_anchor: &str) -> &'a str {
    let start = source
        .find(start_anchor)
        .unwrap_or_else(|| panic!("runtime index should include section `{start_anchor}`"));
    let section = &source[start..];
    section
        .find(end_anchor)
        .map(|end| &section[..end])
        .unwrap_or(section)
}

fn first_backtick_value(source: &str) -> Option<&str> {
    let (_, tail) = source.split_once('`')?;
    let (value, _) = tail.split_once('`')?;
    Some(value)
}

fn leading_plan_id(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() {
        Some(&source[..2])
    } else {
        None
    }
}

fn referenced_plan_ids(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut ids = Vec::new();

    for start in 0..bytes.len().saturating_sub(1) {
        let candidate = &bytes[start..start + 2];
        if candidate.iter().all(u8::is_ascii_digit) {
            let id = std::str::from_utf8(candidate)
                .expect("ASCII plan id candidate should be valid utf-8");
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    ids
}

fn runtime_absorption_guard_modules() -> Vec<&'static str> {
    include_str!("mod.rs")
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.strip_prefix("mod ")
                .and_then(|module| module.strip_suffix(';'))
        })
        .collect()
}

fn runtime_absorption_plan_status_support_files() -> Vec<String> {
    let tests_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests");
    let plan_status_dir = tests_root.join("runtime_absorption").join("plan_status");
    let mut files = Vec::new();

    collect_rust_files_relative_to(&plan_status_dir, &tests_root, &mut files);
    files.sort();
    files
}

fn collect_rust_files_relative_to(
    directory: &std::path::Path,
    relative_root: &std::path::Path,
    files: &mut Vec<String>,
) {
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.unwrap_or_else(|error| panic!("failed to read source entry: {error}")))
        .collect();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files_relative_to(&path, relative_root, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let relative_path = path.strip_prefix(relative_root).unwrap_or_else(|error| {
                panic!(
                    "failed to make {} relative to {}: {error}",
                    path.display(),
                    relative_root.display()
                )
            });
            let anchor = relative_path
                .components()
                .map(|component| {
                    component.as_os_str().to_str().unwrap_or_else(|| {
                        panic!("source path should be utf-8: {}", path.display())
                    })
                })
                .collect::<Vec<_>>()
                .join("/");
            files.push(anchor);
        }
    }
}

fn markdown_frontmatter_and_body(source: &str) -> (&str, &str) {
    let source = source
        .strip_prefix("---")
        .expect("markdown document should start with YAML frontmatter");
    let frontmatter_end = source
        .find("\n---")
        .expect("markdown document should close YAML frontmatter");
    let frontmatter = &source[..frontmatter_end];
    let body = &source[frontmatter_end + "\n---".len()..];
    (frontmatter, body)
}

fn markdown_table_cells(row: &str) -> Vec<&str> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

#[test]
fn runtime_index_subplan_map_covers_existing_plan_files_without_stale_rows() {
    let index_source = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
        "runtime index subplan map should exactly cover runtime 01-14 subplan files"
    );
}

#[test]
fn runtime_index_problem_rows_reference_existing_subplans() {
    let index_source = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
    let problem_section =
        index_section_between(index_source, "| # | 问题 | 证据 | 子计划 |", "### 2.3");
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
    let index_source = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
fn runtime_index_status_map_matches_subplan_frontmatter() {
    let index_source = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

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
    let index_source = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
    let index_source = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
            ["owner", "backlog", "触发", "立项", "需求", "稳定"]
                .iter()
                .any(|marker| cells[2].contains(marker)),
            "runtime backlog gap `{}` should name an owner or trigger condition",
            cells[0]
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

#[test]
fn runtime_05_closeout_status_waits_for_full_scene_cargo_gate() {
    let source = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );

    assert_eq!(
        frontmatter_status(source),
        Some("in_progress"),
        "Runtime 05 should not be completed until the full scene:: Cargo gate closes"
    );
    for required_anchor in [
        "pending_full_scene_cargo",
        "cargo test -p zircon_runtime --lib scene:: --locked",
        "frontmatter 从 `completed` 修正为 `in_progress`",
    ] {
        assert!(
            source.contains(required_anchor),
            "Runtime 05 closeout plan should record `{required_anchor}`"
        );
    }
}

mod cargo_gates;
mod recent_static_guards;

#[test]
fn runtime_architecture_review_documents_all_absorption_guards() {
    let review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let (review_frontmatter, review_body) = markdown_frontmatter_and_body(review);
    let guard_modules = runtime_absorption_guard_modules();
    let harness_short_anchor = "runtime_absorption/mod.rs";
    let harness_full_anchor = "zircon_runtime/src/tests/runtime_absorption/mod.rs";

    assert!(
        !guard_modules.is_empty(),
        "runtime_absorption/mod.rs should expose at least one guard module"
    );
    assert!(
        review_frontmatter.contains(harness_full_anchor),
        "runtime architecture review frontmatter should list absorption harness `{harness_full_anchor}`"
    );
    assert!(
        review_body.contains(harness_short_anchor),
        "runtime architecture review body should document absorption harness `{harness_short_anchor}`"
    );
    for module in guard_modules {
        let short_anchor = format!("runtime_absorption/{module}.rs");
        let full_anchor = format!("zircon_runtime/src/tests/{short_anchor}");
        assert!(
            review_frontmatter.contains(&full_anchor),
            "runtime architecture review frontmatter should list guard module `{full_anchor}`"
        );
        assert!(
            review_body.contains(&short_anchor),
            "runtime architecture review body should document guard module `{short_anchor}`"
        );
    }
    for short_anchor in runtime_absorption_plan_status_support_files() {
        let full_anchor = format!("zircon_runtime/src/tests/{short_anchor}");
        assert!(
            review_frontmatter.contains(&full_anchor),
            "runtime architecture review frontmatter should list plan-status support file `{full_anchor}`"
        );
        assert!(
            review_body.contains(&short_anchor),
            "runtime architecture review body should document plan-status support file `{short_anchor}`"
        );
    }
}
