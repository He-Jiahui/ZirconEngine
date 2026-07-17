//! Static contracts for React/MUI Hub page copy and runtime labels.

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

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

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
            "{source_name} should contain page-copy snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete page-copy snippet {snippet:?}"
        );
    }
}

#[test]
fn rust_localization_and_view_model_own_page_subtitles_status_and_quick_action_labels() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let display = read_crate_file("src/tauri_app/view_model/display.rs");
    let localized = read_crate_file("src/tauri_app/view_model/localized.rs");
    let project_messages = read_crate_file("src/state/hub_message/project.rs");
    let engine_messages = read_crate_file("src/state/hub_message/engine.rs");
    let delivery_messages = read_crate_file("src/state/hub_message/delivery.rs");
    let process_messages = read_crate_file("src/state/hub_message/process.rs");
    let settings_messages = read_crate_file("src/state/hub_message/settings.rs");
    let learn_messages = read_crate_file("src/state/hub_message/learn.rs");
    let shell_messages = read_crate_file("src/state/hub_message/shell.rs");
    let action_history = read_crate_file("src/tauri_app/view_model/action_history.rs");
    let project_templates = read_crate_file("src/tauri_app/view_model/project_templates.rs");
    let ui_text = read_crate_file("src/tauri_app/view_model/ui_text.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let task_status = read_crate_file("src/state/task_status.rs");

    assert_contains_all(
        "navigation.rs",
        &navigation,
        &[
            "pub fn id(self) -> &'static str",
            "Self::Projects => \"projects\"",
            "Self::Editor => \"editor\"",
            "Self::Assets => \"assets\"",
            "Self::Builds => \"builds\"",
            "Self::Plugins => \"plugins\"",
            "Self::Cloud => \"cloud\"",
            "Self::Team => \"team\"",
            "Self::Learn => \"learn\"",
            "Self::Settings => \"settings\"",
        ],
    );
    assert_not_contains_any(
        "navigation.rs",
        &navigation,
        &["pub fn title(self)", "pub fn subtitle(self)"],
    );
    assert_contains_all(
        "display.rs",
        &display,
        &[
            "pub(crate) fn path_text(path: &Path, language: HubLanguage) -> String",
            ".pair(\"Not configured\", \"未配置\")",
            "pub(crate) fn relative_time(now_ms: u64, then_ms: u64, language: HubLanguage) -> String",
        ],
    );
    assert_contains_all(
        "localized.rs",
        &localized,
        &[
            "pub(crate) fn page_title(self, page: HubPage) -> &'static str",
            "HubLanguage::English => match page",
            "HubPage::Projects => \"Projects\"",
            "HubPage::Cloud => \"Local Delivery\"",
            "HubLanguage::Chinese => match page",
            "HubPage::Projects => \"项目\"",
            "pub(crate) fn page_subtitle(self, page: HubPage) -> &'static str",
            "HubPage::Projects => \"Manage your projects and start building worlds.\"",
            "HubPage::Settings => \"Configure toolchains, source paths, and defaults.\"",
            "HubPage::Projects => \"管理本地项目并启动世界构建流程。\"",
            "HubPage::Settings => \"配置工具链、源码路径、构建默认值和语言。\"",
            "pub(crate) fn status_label(self, label: &str) -> String",
            "\"Ready\" => \"就绪\"",
            "\"Action failed\" => \"操作失败\"",
            "\"Import cancelled\" => \"已取消导入\"",
            "\"Projects filtered\" => \"项目已筛选\"",
            "\"Projects sorted\" => \"项目已排序\"",
            "pub(crate) fn operation_target(self, target: &str) -> String",
            "\"Output Folder\" => \"输出文件夹\"",
            "\"Hub settings\" => \"Hub 设置\"",
            "pub(crate) fn render_message(self, message: &HubMessage) -> String",
            "message.render(self.language)",
        ],
    );
    assert_contains_all(
        "project.rs",
        &project_messages,
        &[
            "TemplateComingSoon",
            "项目模板尚未开放：{0}",
            "未在 {0} 找到 zircon-project.toml",
            "项目目录已创建于 {0}，但 Hub 记录失败：{1}",
            "项目未绑定源码引擎：{0}",
            "已创建 {0}",
            "已导入 {0}",
            "显示{0}",
            "按{0}排序",
        ],
    );
    assert_contains_all(
        "engine.rs",
        &engine_messages,
        &[
            "UnknownSourceEngine",
            "未知源码引擎：{0}",
            "源码检出目录缺失",
            "已暂存编辑器/运行时载荷",
        ],
    );
    assert_contains_all(
        "delivery.rs",
        &delivery_messages,
        &[
            "FileCountDetail",
            "{0} -> {1}（{2} 个文件）",
            "已打包 {0} 到 {1}（{2} 个文件）",
            "输出文件夹不存在：{0}",
            "包目录已存在：{0}",
            "设备安装已存在：{0}",
        ],
    );
    assert_contains_all(
        "process.rs",
        &process_messages,
        &[
            "OpeningTargetProcess",
            "编辑器可执行文件不可用：{0}",
            "已启动进程 {0}",
            "正在打开 {0}（进程 {1}）",
        ],
    );
    assert_contains_all(
        "settings.rs",
        &settings_messages,
        &[
            "UnknownLanguage",
            "未知 Hub 语言：{0}",
            "检查设置值后重新保存",
            "草稿已恢复为内置默认值",
        ],
    );
    assert_contains_all(
        "learn.rs",
        &learn_messages,
        &[
            "ResourceFileDoesNotExist",
            "资源文件不存在：{0}",
            "需要打开资源目标",
        ],
    );
    assert_contains_all(
        "shell.rs",
        &shell_messages,
        &[
            "OpenedPath",
            "Hub 已就绪",
            "已打开 {0}",
            "检查操作目标后重试",
        ],
    );
    assert_contains_all(
        "action_history.rs",
        &action_history,
        &[
            "let detail = text.render_message(&record.detail);",
            "let log_excerpt = text.render_message(&record.log_excerpt);",
            "let detail_rows = action_history_detail_rows(",
            ".map(|recovery| text.render_message(recovery))",
            "fn action_history_row_localizes_log_excerpt()",
        ],
    );
    assert_contains_all(
        "ui_text.rs",
        &ui_text,
        &[
            "title: text.pair(\"Projects\", \"项目\").to_string()",
            "browser_title: text.pair(\"Project Browser\", \"项目浏览器\").to_string()",
            "detail_title: text.pair(\"Project Detail\", \"项目详情\").to_string()",
            "search_placeholder: text.pair(\"Search projects...\", \"搜索项目...\").to_string()",
            "open_editor: text.pair(\"Open Editor\", \"打开编辑器\").to_string()",
            "Choose a project from the browser, or open an empty editor.",
            "从项目浏览器选择项目，或直接打开空编辑器。",
            "package_project: text.pair(\"Package Project\", \"打包项目\").to_string()",
            "install_to_device: text.pair(\"Install to Device\", \"安装到设备\").to_string()",
        ],
    );
    assert_not_contains_any(
        "ui_text.rs",
        &ui_text,
        &[
            "button_states",
            "button_state_primary",
            "Button States",
            "按钮状态",
            "project_menu_label",
            "Project menu",
            "项目菜单",
        ],
    );
    assert_contains_all(
        "project_templates.rs",
        &project_templates,
        &[
            "pub(crate) struct HubProjectTemplate",
            "pub option_label: String",
            "pub(super) fn project_template_label(",
            "option_label: template_option_label(&title, &status, template.enabled, language)",
            "HubLanguage::Chinese => format!(\"{title}（{status}）\")",
            "HubLanguage::English => format!(\"{title} ({status})\")",
            "disabled_template_option_label_is_localized_before_react_renders_it",
            "assert_eq!(template.option_label, \"2D 场景（敬请期待）\")",
        ],
    );
    assert_not_contains_any(
        "ui_text.rs",
        &ui_text,
        &[
            "Select a project before launching the editor.",
            "启动编辑器前先选择一个项目。",
        ],
    );
    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "mod display;",
            "mod project_templates;",
            "use project_templates::{project_template_label, project_template_rows, HubProjectTemplate};",
            "let text = HubTextBundle::new(snapshot.settings.language);",
            "page_title: text.page_title(snapshot.selected_page).to_string()",
            "page_subtitle: text.page_subtitle(snapshot.selected_page).to_string()",
            "project_templates: project_template_rows(snapshot.settings.language)",
            "format!(\"{scope}: {}\", text.operation_target(target))",
            "fn task_summary_localizes_backend_operation_targets()",
            "relative_time(now_unix_ms(), project.last_opened_unix_ms, language)",
            "HubLanguage::Chinese => format!(\"修改于 {modified_relative}\")",
            ".pair(\"Available\", \"可用\")",
            ".pair(\"Missing\", \"缺失\")",
            "quick_action_detail(",
            "build_detail_for_project(",
            "Build selected project",
            "Build latest recent project",
            "\"Bind a Source Engine to",
            "\"Install to Device\"",
            "Install selected project",
            "Install latest recent project",
            "\"Package Project\"",
            "Package selected project",
            "Package latest recent project",
            "\"Select a project before packaging\"",
            "\"Open in Editor\"",
            "format!(\"Open {name} in Editor\")",
            "\"Open Editor without a project\"",
            "repository_available: !team.repository_path.as_os_str().is_empty()",
            "commits_label: commit_count_label(member.commits, language)",
        ],
    );
    assert_contains_all(
        "task_status.rs",
        &task_status,
        &[
            "label: \"Ready\".to_string()",
            "detail: HubMessage::new(HubMessageId::Shell(ShellMessageId::HubReady))",
            "pub recovery: Option<HubMessage>",
            "TaskStatus::success(\"Project selected\", HubMessage::raw_text(\"Game\"))",
            "operation_summary",
        ],
    );
}

#[test]
fn projects_dashboard_and_browser_copy_match_the_reference_surface() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let toolbar = read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx");
    let create_dialog = read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "const text = state.ui.projects;",
            "const actionText = state.ui.actions;",
            "<Typography variant=\"h4\">{text.title}</Typography>",
            "<ProjectsToolbar",
            "title={text.noProjectsFound}",
            "detail={text.searchFiltersEmpty}",
            "title={text.recentProjects}",
            "{actionText.viewAllProjects}",
            "<HubPanel title={text.quickActions}>",
            "<CreateProjectDialog",
        ],
    );
    assert_contains_all(
        "ProjectsToolbar.tsx",
        &toolbar,
        &[
            "placeholder={text.searchPlaceholder}",
            "{ value: \"all\", label: text.filterAll }",
            "{ value: \"last-modified\", label: text.sortLastModified }",
            "{ value: \"grid\", label: text.gridView",
            "{ value: \"list\", label: text.listView",
        ],
    );
    assert_contains_all(
        "CreateProjectDialog.tsx",
        &create_dialog,
        &[
            "title={text.newProjectDialog}",
            "label={text.projectName}",
            "templates.map((projectTemplate) =>",
            "label: projectTemplate.optionLabel",
            "placeholder={text.sourceEngine}",
            "options={sourceEngines.map((engine) => ({",
            "engineId: engineId || null",
        ],
    );
    assert_not_contains_any(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "`${projectTemplate.title} - ${projectTemplate.status}`",
            "projectTemplate.enabled ? projectTemplate.title",
            "engineId: state.activeSourceEngineId,",
            "ButtonStatesPanel",
            "text.buttonStates",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "const text = state.ui.projects;",
            "const actionText = state.ui.actions;",
            "<Typography variant=\"h4\">{text.browserTitle}</Typography>",
            "{actionText.dashboard}",
            "{actionText.newProject}",
            "placeholder={text.searchPlaceholder}",
            "{ value: \"all\", label: text.filterAll }",
            "{ value: \"existing\", label: text.filterExisting }",
            "{ value: \"missing\", label: text.filterMissing }",
            "<HubPanel title={text.allProjects}>",
            "EmptyStateBlock title={text.noProjectsFound} detail={text.noRecentProjectMatches}",
            "<HubPanel title={text.quickActions}>",
            "<HubPanel title={text.sourceEngines}>",
        ],
    );
}

#[test]
fn project_detail_copy_targets_selected_project_and_actions() {
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let metrics = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");
    let sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");

    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "const text = state.ui.projects;",
            "const actionText = state.ui.actions;",
            "project?.name ?? text.detailTitle",
            "{actionText.browser}",
            "{actionText.openEditor}",
            "EmptyStateBlock title={text.noProjectSelected} detail={text.chooseProjectFromBrowser}",
            "<ProjectMetricsGrid",
            "<ProjectDetailSidebar",
            "{ value: \"overview\", label: text.overview }",
            "{ value: \"files\", label: text.files }",
            "{ value: \"actions\", label: text.actions }",
            "HubPanel title={text.projectOverview}",
            "HubPanel title={text.projectTree}",
            "HubPanel title={text.projectActions}",
        ],
    );
    assert_contains_all(
        "ProjectMetricsGrid.tsx",
        &metrics,
        &[
            "MetricCard label={text.status}",
            "text.pathUnavailable",
            "MetricCard label={text.engine}",
            "text.projectBinding",
            "MetricCard label={text.lastModified}",
            "MetricCard label={text.projectPin}",
            "detail={project.templateLabel}",
        ],
    );
    assert_contains_all(
        "ProjectDetailSidebar.tsx",
        &sidebar,
        &[
            "HubPanel title={text.quickActions}",
            "HubPanel title={text.sourceEngines}",
            "HubPanel title={text.package}",
            "actionText.packageProject",
            "actionText.installToDevice",
        ],
    );
    assert_not_contains_any(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "Current Project",
            "active project",
            "Build Controls",
            "project.templateId ?? text.notRecorded",
            "project.templateId ?? text.noTemplate",
        ],
    );
}

#[test]
fn workspace_copy_stays_local_selected_project_and_component_focused() {
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let team = read_crate_file("web/src/pages/TeamPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");

    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "MetricCard label={text.buildProfile}",
            "MetricCard label={text.outputRoot}",
            "MetricCard",
            "label={text.recentWorkflows}",
            "{ value: \"workflow\", label: common.workflow }",
            "{ value: \"history\", label: common.history }",
            "{ value: \"outputs\", label: common.outputs }",
            "HubPanel title={text.buildWorkflow}",
            "HubPanel title={common.selectedProject}",
            "workflowProject ?",
            "EmptyStateBlock title={common.noProjectSelected} detail={text.noProjectSelectedDetail}",
            "HubPanel title={text.buildHistory}",
            "EmptyStateBlock title={text.noBuildHistory} detail={text.noBuildHistoryDetail}",
            "BuildActionDetail",
            "<HubList items={action.detailRows} />",
            "HubPanel title={text.outputTree}",
        ],
    );
    assert_not_contains_any(
        "BuildsPage.tsx",
        &builds,
        &[
            "detail: action.commandLine.length > 0 ? action.commandLine.join(\" \") : common.noCommandRecorded,",
            "action.logExcerpt || common.noLogExcerpt",
            "action.outputDir ?? common.noOutputDirectory",
        ],
    );
    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &[
            "placeholder={`${text.searchPlaceholderPrefix}${text.searchPlaceholderSeparator}${state.pageTitle}${text.searchPlaceholderSuffix}`}",
            "MetricCard label={text.entries}",
            "MetricCard label={text.categories}",
            "MetricCard label={text.scopes}",
            "HubPanel title={catalogPanelTitle(mode, text)}",
            "HubPanel title={text.selectedEntry}",
            "EmptyStateBlock title={text.noEntriesFound} detail={text.noEntriesFoundDetail}",
            "EmptyStateBlock title={text.noCatalogEntrySelected} detail={text.noCatalogEntrySelectedDetail}",
        ],
    );
    assert_not_contains_any(
        "CatalogPage.tsx",
        &catalog,
        &[
            "${state.pageTitle}...",
            "${text.searchPlaceholderPrefix} ${state.pageTitle}",
            "`${state.pageTitle} ${text.catalogSuffix}`",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "MetricCard label={text.packageRoot}",
            "MetricCard label={text.deviceInstall}",
            "MetricCard label={text.serviceSlots}",
            "{ value: \"packages\", label: common.packages }",
            "{ value: \"installs\", label: common.installs }",
            "{ value: \"services\", label: common.services }",
            "HubPanel title={text.packageOutputs}",
            "EmptyStateBlock title={text.noPackagesRecorded} detail={text.noPackagesRecordedDetail}",
            "HubPanel title={text.installReadiness}",
            "HubPanel title={text.reservedServices}",
            "text.localPackageHandoff",
        ],
    );
    assert_contains_all(
        "TeamPage.tsx",
        &team,
        &[
            "MetricCard",
            "label={text.repository}",
            "value={state.team.repositoryAvailable ? common.connected : common.notConfigured}",
            "label={text.identity}",
            "label={text.contributors}",
            "{ value: \"overview\", label: common.overview }",
            "{ value: \"activity\", label: common.activity }",
            "{ value: \"toolchain\", label: common.toolchain }",
            "HubPanel title={text.teamMembers}",
            "EmptyStateBlock title={text.noTeamMembersFound} detail={text.noTeamMembersFoundDetail}",
            "HubPanel title={text.actionHistory}",
            "EmptyStateBlock title={text.noRecentActions} detail={text.noRecentActionsDetail}",
            "<HubList items={action.detailRows} />",
            "meta: member.commitsLabel",
        ],
    );
    assert_not_contains_any(
        "TeamPage.tsx",
        &team,
        &[
            "detail: action.commandLine.length > 0 ? action.commandLine.join(\" \") : common.noCommandRecorded,",
            "action.logExcerpt || common.noLogExcerpt",
            "action.outputDir ?? common.noOutputDirectory",
        ],
    );
    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "<Typography variant=\"h4\">{settingsText.heading}</Typography>",
            "{settingsText.saveButton}",
            "{settingsText.discardButton}",
            "{settingsText.restoreDefaultsButton}",
            "void onAction(HUB_ACTION.discardSettingsDraft)",
            "void onAction(HUB_ACTION.restoreDefaultSettings)",
            "MetricCard label={settingsText.sourceEnginesPanel}",
            "MetricCard label={labels.buildProfile}",
            "MetricCard label={labels.language}",
            "MetricCard label={settingsText.configurationHealthPanel}",
            "HubTabs value={tab} onChange={setTab} options={settingsText.tabs}",
            "<SettingsSection",
        ],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "options={settingsText.buildProfileOptions}",
            "options={settingsText.languageOptions}",
            "HubPanel title={settingsText.buildDefaultsPanel}",
            "HubPanel title={settingsText.configurationHealthPanel}",
            "HubPanel title={settingsText.activeSourceEnginePanel}",
        ],
    );
}

#[test]
fn settings_page_displays_localized_option_labels_without_changing_stable_payload_values() {
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");
    let settings_options = read_crate_file("web/src/settings/options.ts");

    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "import { settingsJobCountLabel, settingsOptionLabel } from \"../settings/options\";",
            "const buildProfileLabel = settingsOptionLabel(settingsText.buildProfileOptions, draft.buildProfile);",
            "const languageLabel = settingsOptionLabel(settingsText.languageOptions, draft.language);",
            "const draftJobsLabel = settingsJobCountLabel(settingsText, draft.jobs);",
            "MetricCard label={labels.buildProfile} value={buildProfileLabel} detail={draftJobsLabel}",
            "MetricCard label={labels.language} value={languageLabel}",
            "detail: languageLabel",
            "void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })",
        ],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "HubSwitch checked={draft.buildProfile === \"release\"} label={labels.releaseBuild} detail={buildProfileLabel}",
            "HubCheckbox checked={draft.language === \"Chinese\"} label={labels.localizedUi} detail={languageLabel}",
            "value={draft.buildProfile}",
            "onChange={(value) => updateDraft(\"buildProfile\", value)}",
            "value={draft.language}",
            "onChange={(value) => updateDraft(\"language\", value)}",
        ],
    );
    assert_contains_all(
        "options.ts",
        &settings_options,
        &[
            "import type { HubSettingsOptionText, HubSettingsText } from \"../types/hub\";",
            "export function settingsOptionLabel(options: HubSettingsOptionText[], value: string): string {",
            "return options.find((option) => option.value === value)?.label ?? value;",
            "export function settingsJobCountLabel(text: HubSettingsText, jobs: number): string {",
            "const normalizedJobs = Number.isFinite(jobs) ? Math.max(1, Math.trunc(jobs)) : 1;",
            "const template = normalizedJobs === 1 ? text.jobCountSingularTemplate : text.jobCountPluralTemplate;",
            "return template.replace(\"{jobs}\", `${normalizedJobs}`);",
        ],
    );
    assert_not_contains_any(
        "SettingsPage.tsx",
        &settings,
        &[
            "MetricCard label={labels.buildProfile} value={draft.buildProfile}",
            "MetricCard label={labels.language} value={draft.language}",
            "detail={`${draft.jobs}`}",
            "detail: draft.language",
            "detail={draft.buildProfile}",
            "detail={draft.language}",
        ],
    );
    assert_not_contains_any(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "MetricCard label={labels.buildProfile} value={draft.buildProfile}",
            "MetricCard label={labels.language} value={draft.language}",
            "detail={`${draft.jobs}`}",
            "detail: draft.language",
            "detail={draft.buildProfile}",
            "detail={draft.language}",
        ],
    );
}

#[test]
fn workspace_pages_display_localized_saved_settings_option_labels() {
    let settings_dto = read_crate_file("src/tauri_app/view_model/settings_dto.rs");
    let types = read_crate_file("web/src/types/hub.ts");
    let data = read_crate_file("web/src/data/hubData.ts");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let workspace = read_crate_file("web/src/pages/WorkspacePage.tsx");

    assert_contains_all(
        "settings_dto.rs",
        &settings_dto,
        &[
            "pub build_profile_label: String",
            "pub language_label: String",
            "pub jobs_label: String",
            "pub build_profile_detail: String",
            "pub build_workflow_detail: String",
            "option(\"Chinese\", \"中文\")",
            "option(\"English\", \"English\")",
            "fn job_count_label(jobs: u16, language: HubLanguage) -> String",
            "settings_language_options_keep_native_names_across_ui_languages",
            "settings_summary_projects_saved_option_labels_for_react_consumers",
        ],
    );
    assert_not_contains_any(
        "settings_dto.rs",
        &settings_dto,
        &["option(\"Chinese\", text.pair(\"Chinese\", \"中文\"))"],
    );
    assert_contains_all(
        "hub.ts",
        &types,
        &[
            "buildProfileLabel: string;",
            "languageLabel: string;",
            "jobsLabel: string;",
            "buildProfileDetail: string;",
            "buildWorkflowDetail: string;",
        ],
    );
    assert_contains_all(
        "hubData.ts",
        &data,
        &[
            "buildProfileLabel: \"Debug\"",
            "languageLabel: \"中文\"",
            "jobsLabel: \"1 任务\"",
            "buildProfileDetail: \"Debug / 1 任务\"",
            "buildWorkflowDetail: \"使用当前构建默认值编译编辑器/运行时目标：Debug\"",
        ],
    );

    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "detail: state.settings.buildWorkflowDetail",
            "{ id: \"profile\", label: text.profile, detail: state.settings.buildProfileLabel }",
            "{ id: \"jobs\", label: text.jobs, detail: state.settings.jobsLabel }",
            "MetricCard label={text.buildProfile} value={state.settings.buildProfileLabel} detail={state.settings.jobsLabel}",
        ],
    );
    assert_not_contains_any(
        "BuildsPage.tsx",
        &builds,
        &[
            "import { settingsOptionLabel } from \"../settings/options\";",
            "const buildProfileLabel = settingsOptionLabel(",
            "detail: `${text.compileDetail}:",
            "detail: `${state.settings.jobs}`",
            "detail={`${state.settings.jobs} ${common.jobs}`}",
            "detail: `${text.compileDetail}: ${state.settings.buildProfile}`",
            "{ id: \"profile\", label: text.profile, detail: state.settings.buildProfile }",
            "MetricCard label={text.buildProfile} value={state.settings.buildProfile}",
        ],
    );

    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "{ id: \"profile\", title: state.ui.builds.buildProfile, detail: state.settings.buildProfileDetail }",
        ],
    );
    assert_not_contains_any(
        "CloudPage.tsx",
        &cloud,
        &[
            "import { settingsOptionLabel } from \"../settings/options\";",
            "const buildProfileLabel = settingsOptionLabel(",
            "`${buildProfileLabel} / ${state.settings.jobs} ${common.jobs}`",
            "{ id: \"profile\", title: state.ui.builds.buildProfile, detail: `${state.settings.buildProfile} / ${state.settings.jobs} ${common.jobs}` }",
        ],
    );

    assert_contains_all(
        "WorkspacePage.tsx",
        &workspace,
        &[
            "const activePageLabel = state.ui.shell.navItems.find((item) => item.id === state.activePage)?.label ?? state.pageTitle;",
            "{ id: \"build-profile\", title: labels.buildProfile, detail: state.settings.buildProfileDetail }",
            "detail: activePageLabel,",
            "MetricCard label={state.ui.shell.workspaceProfile} value={state.pageTitle} detail={activePageLabel}",
            "MetricCard label={labels.buildProfile} value={state.settings.buildProfileLabel} detail={state.settings.jobsLabel}",
            "HubSwitch checked={state.settings.buildProfile === \"release\"} label={labels.releaseBuild} detail={state.settings.buildProfileLabel} disabled",
            "HubCheckbox checked={state.settings.language === \"Chinese\"} label={labels.localizedUi} detail={state.settings.languageLabel} disabled",
        ],
    );
    assert_not_contains_any(
        "WorkspacePage.tsx",
        &workspace,
        &[
            "import { settingsOptionLabel } from \"../settings/options\";",
            "const buildProfileLabel = settingsOptionLabel(",
            "const languageLabel = settingsOptionLabel(",
            "`${buildProfileLabel} / ${state.settings.jobs} ${common.jobs}`",
            "detail={`${state.settings.jobs} ${common.jobs}`}",
            "{ id: \"build-profile\", title: labels.buildProfile, detail: `${state.settings.buildProfile} / ${state.settings.jobs} ${common.jobs}` }",
            "MetricCard label={labels.buildProfile} value={state.settings.buildProfile}",
            "label={labels.releaseBuild} detail={state.settings.buildProfile}",
            "label={labels.localizedUi} detail={state.settings.language}",
            "HubCheckbox checked={state.settings.language === \"English\"} label={labels.localizedUi}",
            "detail: state.activePage",
            "detail={state.activePage}",
        ],
    );
}

#[test]
fn workspace_pages_use_localized_count_templates_instead_of_page_suffix_joins() {
    let ui_text = read_crate_file("src/tauri_app/view_model/ui_text.rs");
    let types = read_crate_file("web/src/types/hub.ts");
    let data = read_crate_file("web/src/data/hubData.ts");
    let count_text = read_crate_file("web/src/text/counts.ts");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let team = read_crate_file("web/src/pages/TeamPage.tsx");
    let workspace = read_crate_file("web/src/pages/WorkspacePage.tsx");

    assert_contains_all(
        "ui_text.rs",
        &ui_text,
        &[
            "pub entry_count_template: String",
            "pub available_count_template: String",
            "pub reserved_count_template: String",
            "pub member_count_template: String",
            "pub action_count_template: String",
            "entry_count_template: text",
            ".pair(\"{count} entries\", \"{count} 个条目\")",
            "available_count_template: text",
            ".pair(\"{count} available\", \"{count} 个可用\")",
            "reserved_count_template: text",
            ".pair(\"{count} reserved\", \"{count} 个预留\")",
            "member_count_template: text",
            ".pair(\"{count} members\", \"{count} 位成员\")",
            "action_count_template: text",
            ".pair(\"{count} actions\", \"{count} 次操作\")",
            "pub module_count_template: String",
            "module_count_template: text",
            ".pair(\"{count} modules\", \"{count} 个模块\")",
            "pub package_action_count_template: String",
            "package_action_count_template: text",
            ".pair(\"{count} package actions\", \"{count} 次打包操作\")",
            "pub recent_action_count_template: String",
            "recent_action_count_template: text",
            ".pair(\"{count} recent actions\", \"{count} 次最近操作\")",
        ],
    );
    assert_contains_all(
        "hub.ts",
        &types,
        &[
            "entryCountTemplate: string;",
            "availableCountTemplate: string;",
            "reservedCountTemplate: string;",
            "memberCountTemplate: string;",
            "actionCountTemplate: string;",
            "moduleCountTemplate: string;",
            "packageActionCountTemplate: string;",
            "recentActionCountTemplate: string;",
        ],
    );
    assert_contains_all(
        "hubData.ts",
        &data,
        &[
            "entryCountTemplate: \"{count} 个条目\"",
            "availableCountTemplate: \"{count} 个可用\"",
            "reservedCountTemplate: \"{count} 个预留\"",
            "memberCountTemplate: \"{count} 位成员\"",
            "actionCountTemplate: \"{count} 次操作\"",
            "moduleCountTemplate: \"{count} 个模块\"",
            "packageActionCountTemplate: \"{count} 次打包操作\"",
            "recentActionCountTemplate: \"{count} 次最近操作\"",
        ],
    );
    assert_contains_all(
        "counts.ts",
        &count_text,
        &[
            "export function formatCountText(template: string, count: number): string",
            "return template.replace(\"{count}\", `${normalizedCount}`);",
        ],
    );

    for (label, source) in [
        ("BuildsPage.tsx", builds.as_str()),
        ("CatalogPage.tsx", catalog.as_str()),
        ("CloudPage.tsx", cloud.as_str()),
        ("EditorPage.tsx", editor.as_str()),
        ("TeamPage.tsx", team.as_str()),
        ("WorkspacePage.tsx", workspace.as_str()),
    ] {
        assert_contains_all(label, source, &["formatCountText("]);
    }

    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &["formatCountText(text.moduleCountTemplate, plugin.moduleCount)"],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &["formatCountText(text.packageActionCountTemplate, packageActions.length)"],
    );
    assert_contains_all(
        "TeamPage.tsx",
        &team,
        &["formatCountText(text.recentActionCountTemplate, state.actionHistory.length)"],
    );

    for (label, source) in [
        ("BuildsPage.tsx", builds.as_str()),
        ("CatalogPage.tsx", catalog.as_str()),
        ("CloudPage.tsx", cloud.as_str()),
        ("EditorPage.tsx", editor.as_str()),
        ("TeamPage.tsx", team.as_str()),
        ("WorkspacePage.tsx", workspace.as_str()),
    ] {
        assert_not_contains_any(
            label,
            source,
            &[
                " ${common.entries}`",
                " ${common.available}`",
                " ${common.reserved}`",
                " ${common.members}`",
                " ${common.actions}`",
                " ${text.editorPlugins}`",
                " ${text.packageActionsSuffix}`",
                " ${text.recentActions}`",
                " ${state.ui.catalog.moduleCountSuffix}`",
            ],
        );
    }
}

#[test]
fn cloud_local_delivery_history_rows_show_output_directories_from_action_history() {
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let hub_list = read_crate_file("web/src/components/data/HubList.tsx");

    assert_contains_all(
        "HubList.tsx",
        &hub_list,
        &["secondaryDetail?: string;", "item.secondaryDetail"],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "items={packageActions.map((action) => ({",
            "detail: action.detail,",
            "secondaryDetail: action.outputDir ?? common.noOutputDirectory,",
            "meta: action.finished,",
            "items={installActions.map((action) => ({",
            "onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, item.id, { historyId: item.id })}",
        ],
    );
    assert_not_contains_any(
        "CloudPage.tsx",
        &cloud,
        &[
            "secondaryDetail: actionOutputDetail(action),",
            "function actionOutputDetail(action: HubActionHistoryItem)",
            "return action.detailRows.find((row) => row.id === \"output\")?.detail;",
        ],
    );
}

#[test]
fn app_level_failure_feedback_uses_localized_shell_detail_copy() {
    let ui_text = read_crate_file("src/tauri_app/view_model/ui_text.rs");
    let types = read_crate_file("web/src/types/hub.ts");
    let data = read_crate_file("web/src/data/hubData.ts");
    let app = read_crate_file("web/src/App.tsx");

    assert_contains_all(
        "ui_text.rs",
        &ui_text,
        &[
            "pub live_updates_unavailable_detail: String",
            "pub action_failed_detail: String",
            "live_updates_unavailable_detail: text",
            "\"Unable to subscribe to Hub state updates.\"",
            "\"无法订阅 Hub 状态更新。\"",
            "action_failed_detail: text",
            "\"The Hub backend could not complete this action.\"",
            "\"Hub 后端未能完成该操作。\"",
        ],
    );
    assert_contains_all(
        "hub.ts",
        &types,
        &[
            "liveUpdatesUnavailableDetail: string;",
            "actionFailedDetail: string;",
        ],
    );
    assert_contains_all(
        "hubData.ts",
        &data,
        &[
            "liveUpdatesUnavailableDetail: \"无法订阅 Hub 状态更新。\"",
            "actionFailedDetail: \"Hub 后端未能完成该操作。\"",
        ],
    );
    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const shellText = stateRef.current.ui.shell;",
            "detail: shellText.liveUpdatesUnavailableDetail,",
            "detail: shellText.actionFailedDetail,",
        ],
    );
    assert_not_contains_any(
        "App.tsx",
        &app,
        &[
            "const detail = error instanceof Error ? error.message : String(error);",
            "detail,",
        ],
    );
}

#[test]
fn page_copy_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/project_page_copy_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test project_page_copy_contract",
            "## Page Copy Contract Cutover",
            "React/MUI Hub page copy and runtime labels",
            "src/state/navigation.rs",
            "src/state/task_status.rs",
            "src/tauri_app/view_model.rs",
            "src/tauri_app/view_model/project_templates.rs",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "Project templates are projected by `src/tauri_app/view_model/project_templates.rs`",
            "React no longer joins template title/status with page-local punctuation",
            "Action-history DTOs include `detailRows` for target, finished time, output, recovery, command, and log display",
            "Builds and Team render those rows directly without page-local command/log/output fallback wording",
            "Cloud local-delivery package/install history rows read their visible output line from the backend `outputDir` field",
            "Known internal operation targets such as `Output Folder` and `Hub settings` are translated by `HubTextBundle::operation_target`",
            "fallback Workspace localized UI checkbox follows the same saved-language semantics as Settings",
            "Workspace fallback page labels use Rust-projected navigation labels instead of stable route ids",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`project_page_copy_contract.rs`",
            "React/MUI Hub page copy and runtime labels",
            "HubTextBundle`, runtime labels, import-cancel task labels, project filter/sort/view-all feedback labels",
            "New Project template option labels and selected-project template labels from `src/tauri_app/view_model/project_templates.rs`",
            "editor-launch executable/process details",
            "Editor no-project copy that allows choosing a project or opening an empty editor",
            "package/install file-count suffixes, localized delivery log excerpts, and delivery failure details",
            "task operation targets such as output folder and Hub settings",
            "fallback Workspace localized UI checkbox using `Chinese` as the checked saved-language state",
            "Projects dashboard no longer renders the button-state reference sample strip",
            "action-history `detailRows` render stable target, localized finished time, output directory, recovery hint, command line, and localized log excerpt",
            "Cloud local package/install history rows render localized action detail plus the backend `outputDir` field",
            "Workspace fallback page labels use Rust-projected navigation labels instead of stable route ids",
            "localized project modified-time/status strings",
            "localized action-history action/status/detail/log-excerpt/recovery display text",
        ],
    );
}

#[test]
fn page_copy_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/project_page_copy_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "project_page_copy_contract.rs",
        &contract,
        &[
            "src/state/navigation.rs",
            "src/tauri_app/view_model/localized.rs",
            "src/tauri_app/view_model/project_templates.rs",
            "src/tauri_app/view_model/ui_text.rs",
            "src/state/task_status.rs",
            "src/tauri_app/view_model.rs",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
        ],
    );
    assert_not_contains_any(
        "project_page_copy_contract.rs",
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
