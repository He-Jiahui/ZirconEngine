//! Static contracts for shared React/MUI Hub metric-section layout policy.
//! 固定指标卡的展示职责及页面对业务数值的投影职责，并锁定不同页面指标网格的响应式约定。

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub crate should live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

/// 读取相对 Hub 包根的受审源码作为结构证据；调用方依赖仓库检出完整，读取失败应暴露契约来源缺失。
fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

/// 读取仓库级交接文档或工具证据；约定 Hub 包位于仓库根下一层，不能依赖测试启动时的工作目录。
fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}")),
    )
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain metric-section snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete metric-section snippet {snippet:?}"
        );
    }
}

/// 把指标的色调、图标和文字布局留给共享卡片，业务页面只提交已投影的数值与语义。
#[test]
fn metric_card_owns_shared_card_tone_icon_and_text_layout() {
    let metric = read_crate_file("web/src/components/data/MetricCard.tsx");
    let data_index = read_crate_file("web/src/components/data/index.ts");

    assert_contains_all(
        "MetricCard.tsx",
        &metric,
        &[
            "export interface MetricCardProps",
            "label: string;",
            "value: string;",
            "detail?: string;",
            "icon?: ReactNode;",
            "tone?: \"neutral\" | \"accent\" | \"success\" | \"warning\" | \"error\";",
            "const toneColor = {",
            "neutral: hubTokens.colors.textSoft",
            "accent: hubTokens.colors.accent",
            "success: hubTokens.colors.success",
            "warning: hubTokens.colors.warning",
            "error: hubTokens.colors.error",
            "minHeight: 86",
            "gridTemplateColumns: icon ? \"34px minmax(0, 1fr)\" : \"1fr\"",
            "borderRadius: `${hubTokens.radius.panel}px`",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "Typography variant=\"caption\" noWrap",
            "Typography variant=\"h6\" noWrap",
            "color: toneColor[tone]",
        ],
    );
    assert_contains_all(
        "components/data/index.ts",
        &data_index,
        &["export * from \"./MetricCard\";"],
    );
}

/// 固定项目详情的四项指标由共享网格承载，并按容器宽度收缩，避免页面单独重复卡片布局。
#[test]
fn project_detail_uses_four_metric_cards_then_collapses_responsively() {
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let metrics = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");

    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "ProjectMetricsGrid",
            "project={project}",
            "boundEngine={boundEngine}",
        ],
    );
    assert_contains_all(
        "ProjectMetricsGrid.tsx",
        &metrics,
        &[
            "MetricCard",
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"repeat(2, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"1fr\"",
            "MetricCard label={text.status} value={project.status}",
            "tone={project.exists ? \"success\" : \"warning\"}",
            "MetricCard label={text.engine} value={project.engineVersion}",
            "tone=\"accent\"",
            "MetricCard label={text.lastModified} value={project.modified}",
            "MetricCard label={text.projectPin} value={project.pinned ? text.pinned : text.unpinned}",
        ],
    );
}

/// 核对工作区页面使用三指标网格和共享卡片，页面根据自身 DTO 决定标签与状态。
#[test]
fn workspace_pages_use_shared_three_metric_grid_and_metric_card_atoms() {
    for (page, snippets) in [
        (
            "BuildsPage.tsx",
            vec![
                "MetricCard label={text.buildProfile}",
                "MetricCard label={text.outputRoot}",
                "MetricCard",
                "label={text.recentWorkflows}",
            ],
        ),
        (
            "CatalogPage.tsx",
            vec![
                "MetricCard label={text.entries}",
                "MetricCard label={text.categories}",
                "MetricCard label={text.scopes}",
            ],
        ),
        (
            "CloudPage.tsx",
            vec![
                "MetricCard label={text.packageRoot}",
                "MetricCard label={text.deviceInstall}",
                "MetricCard label={text.serviceSlots}",
            ],
        ),
        (
            "TeamPage.tsx",
            vec![
                "MetricCard",
                "label={text.repository}",
                "MetricCard",
                "label={text.identity}",
                "MetricCard",
                "label={text.contributors}",
            ],
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(
            page,
            &source,
            &[
                "MetricCard",
                "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
                "gridTemplateColumns: \"1fr\"",
                "@media (max-width: 980px)",
            ],
        );
        assert_contains_all(page, &source, &snippets);
        assert_not_contains_any(page, &source, &["<Card", "<Paper", "HubMetricSectionState"]);
    }
}

// BUG: [CR-HUBTESTA-0010] Cloud 页把打包和安装历史汇入共享投影器，测试仍要求两项页面内投影片段；证据：CloudPage.tsx。
/// 把历史筛选、分类统计和业务状态选择放在页面或其投影助手，指标卡只承担展示。
#[test]
fn metric_pages_keep_data_projection_in_pages_not_metric_card() {
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let team = read_crate_file("web/src/pages/TeamPage.tsx");

    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const buildHistory = useMemo(",
            "const latestAction = buildHistory[0];",
            "tone={latestAction ? metricTone(latestAction.tone) : \"neutral\"}",
        ],
    );
    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &[
            "const categoryCount = new Set(rows.map((row) => row.category)).size;",
            "const scopeCount = new Set(rows.map((row) => row.scope)).size;",
            "detail={selectedRow?.category ?? text.noCatalog}",
            "detail={selectedRow?.scope ?? text.noScope}",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const packageActions = useMemo(",
            "const installActions = useMemo(",
            "const reservedServices",
        ],
    );
    assert_contains_all(
        "TeamPage.tsx",
        &team,
        &[
            "state.team.repositoryAvailable ? common.connected : common.notConfigured",
            "value={state.team.identityName || common.notConfigured}",
            "value={`${state.team.members.length}`}",
        ],
    );
}

/// 要求文档记录指标卡职责及不同页面网格口径，便于后续响应式调整连同测试更新。
#[test]
fn metric_section_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_metric_section_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_metric_section_contract",
            "## Metric Section Contract Cutover",
            "React/MUI metric section system",
            "web/src/components/data/MetricCard.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_metric_section_contract.rs`",
            "React/MUI metric section system",
            "shared MetricCard tone/icon/text atom",
            "four-card Project Detail metrics and three-card workspace metric grids",
        ],
    );
}

/// 自读测试源码核对受审目标仍指向当前前端；禁用词分段构造，新增注释也不能携带其完整旧引用。
#[test]
fn metric_section_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_metric_section_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_metric_section_contract.rs",
        &contract,
        &[
            "web/src/components/data/MetricCard.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_metric_section_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_taffy_name.as_str(),
        ],
    );
}
