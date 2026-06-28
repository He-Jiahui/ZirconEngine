use std::path::PathBuf;

use crate::engines::SourceEngineInstall;
use crate::projects::{project_metadata_key, ProjectMetadata};
use crate::settings::{HubLanguage, HubSettings};
use crate::state::{
    DeliveryMessageId, HubMessage, HubMessageId, HubPage, ProjectFilterMode, ProjectMessageId,
    ProjectSortMode, ProjectSubpage, ProjectViewMode, SettingsMessageId, TaskStatus,
    TASK_PROGRESS_PREPARED_PERCENT,
};
use crate::team::TeamOverview;

use super::display::{MILLIS_PER_DAY, MILLIS_PER_HOUR};
use super::*;

#[test]
fn view_model_projects_come_from_snapshot_filtering_and_state_ids() {
    let snapshot = HubSnapshot {
        selected_page: HubPage::Projects,
        project_filter: ProjectFilterMode::All,
        project_sort: ProjectSortMode::Name,
        project_view_mode: ProjectViewMode::List,
        project_subpage: ProjectSubpage::ProjectBrowser,
        search_query: "stellar".to_string(),
        selected_project_path: Some(PathBuf::from("E:/Projects/StellarOutpost")),
        new_project_name: String::new(),
        selected_template_id: "renderable-empty".to_string(),
        new_project_location: PathBuf::from("E:/Projects"),
        new_project_engine_id: None,
        pending_delete_project_path: None,
        task_status: TaskStatus::idle(),
        queued_background_actions: 0,
        recent_projects: vec![
            RecentProject::new("Elysium Chronicles", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ],
        project_metadata: crate::projects::ProjectMetadataMap::new(),
        assets: Vec::new(),
        learn_resources: Vec::new(),
        plugins: Vec::new(),
        team: TeamOverview::empty(),
        action_history: Vec::new(),
        engines: Vec::new(),
        active_engine_id: None,
        settings: HubSettings::default(),
        settings_draft: HubSettings::default(),
    };

    let model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(model.projects.len(), 1);
    assert_eq!(model.projects[0].name, "Stellar Outpost");
    assert_eq!(model.browser_projects.len(), 1);
    assert_eq!(model.project_view_mode, "list");
    assert_eq!(model.project_subpage, "project-browser");
    assert_eq!(
        model.selected_project_id.as_deref(),
        Some("E:/Projects/StellarOutpost")
    );
    assert_eq!(
        model
            .selected_project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("Stellar Outpost")
    );
    assert_eq!(model.task_summary.label, "就绪");
    assert_eq!(model.task_summary.operation, "Hub");
    assert_eq!(model.task_summary.progress_percent, 0);
    assert_eq!(model.assets.len(), 0);
    assert_eq!(model.plugins.len(), 0);
    assert_eq!(model.learn_resources.len(), 0);
    assert_eq!(model.team.members.len(), 0);
    assert_eq!(model.action_history.len(), 0);
}

#[test]
fn quick_actions_use_selected_project_scope_and_engine_binding() {
    let selected = PathBuf::from("E:/Projects/Game");
    let mut snapshot = snapshot_with_projects(
        Some(PathBuf::from("E:\\Projects\\Game\\")),
        vec![RecentProject::new("Game", selected.clone(), 10)],
    );
    bind_source_engine(&mut snapshot, &selected);

    let model = HubViewModel::from_snapshot(&snapshot);

    assert!(quick_action(&model, "build-project").enabled);
    assert_eq!(
        quick_action(&model, "build-project").detail,
        "构建已选项目 Game"
    );
    assert_eq!(
        quick_action(&model, "package-project").detail,
        "打包已选项目 Game"
    );
}

#[test]
fn selected_project_template_label_localizes_stable_template_metadata() {
    let selected = PathBuf::from("E:/Projects/Game");
    let mut snapshot = snapshot_with_projects(
        Some(selected.clone()),
        vec![RecentProject::new("Game", selected.clone(), 10)],
    );
    snapshot.settings.language = HubLanguage::Chinese;
    snapshot.project_metadata.insert(
        project_metadata_key(&selected),
        ProjectMetadata {
            last_selected_template: Some("renderable-empty".to_string()),
            ..ProjectMetadata::default()
        },
    );

    let model = HubViewModel::from_snapshot(&snapshot);
    let project = model
        .selected_project
        .as_ref()
        .expect("selected project detail should be projected");

    assert_eq!(project.template_id.as_deref(), Some("renderable-empty"));
    assert_eq!(project.template_label, "可渲染空项目");
}

#[test]
fn quick_actions_disable_unbound_or_stale_project_targets() {
    let selected = PathBuf::from("E:/Projects/Game");
    let unbound = HubViewModel::from_snapshot(&snapshot_with_projects(
        Some(PathBuf::from("E:\\Projects\\Game\\")),
        vec![RecentProject::new("Game", selected, 10)],
    ));

    assert!(!quick_action(&unbound, "build-project").enabled);
    assert!(quick_action(&unbound, "package-project").enabled);
    assert_eq!(
        quick_action(&unbound, "build-project").detail,
        "先为 Game 绑定源码引擎"
    );

    let stale = HubViewModel::from_snapshot(&snapshot_with_projects(
        Some(PathBuf::from("E:/Projects/Missing")),
        vec![RecentProject::new("Latest", "E:/Projects/Latest", 20)],
    ));

    assert!(!quick_action(&stale, "build-project").enabled);
    assert!(!quick_action(&stale, "package-project").enabled);
    assert_eq!(
        quick_action(&stale, "build-project").detail,
        "已选项目不再可用"
    );
    assert_eq!(
        quick_action(&stale, "open-editor").detail,
        "不带项目打开编辑器"
    );
}

#[test]
fn quick_actions_use_latest_recent_only_when_no_project_is_selected() {
    let latest = PathBuf::from("E:/Projects/Latest");
    let mut snapshot = snapshot_with_projects(
        None,
        vec![
            RecentProject::new("Old", "E:/Projects/Old", 1),
            RecentProject::new("Latest", latest.clone(), 20),
        ],
    );
    bind_source_engine(&mut snapshot, &latest);

    let model = HubViewModel::from_snapshot(&snapshot);

    assert!(quick_action(&model, "build-project").enabled);
    assert_eq!(
        quick_action(&model, "build-project").detail,
        "构建最近项目 Latest"
    );
    assert_eq!(
        quick_action(&model, "install-device").detail,
        "安装最近项目 Latest"
    );
}

#[test]
fn source_engine_rows_localize_status_labels() {
    let mut snapshot = snapshot_with_projects(None, Vec::new());
    snapshot.settings.language = HubLanguage::Chinese;
    snapshot.active_engine_id = Some("source-local".to_string());
    snapshot.engines.push(SourceEngineInstall {
        id: "source-local".to_string(),
        display_name: "Local Source".to_string(),
        source_dir: PathBuf::from("E:/Source/ZirconEngine"),
        output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    snapshot.engines.push(SourceEngineInstall {
        id: "source-backup".to_string(),
        display_name: "Backup Source".to_string(),
        source_dir: PathBuf::from("E:/Source/ZirconEngineBackup"),
        output_dir: PathBuf::from("E:/Source/ZirconEngineBackup/out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });

    let model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(model.source_engines[0].status, "当前");
    assert_eq!(model.source_engines[1].status, "已注册");
}

#[test]
fn relative_time_uses_compact_labels() {
    let now = 10 * MILLIS_PER_DAY;

    assert_eq!(relative_time(now, now, HubLanguage::English), "just now");
    assert_eq!(
        relative_time(now, now - (2 * MILLIS_PER_HOUR), HubLanguage::English),
        "2h ago"
    );
    assert_eq!(
        relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::English),
        "3d ago"
    );
    assert_eq!(relative_time(now, now, HubLanguage::Chinese), "刚刚");
    assert_eq!(
        relative_time(now, now - (2 * MILLIS_PER_HOUR), HubLanguage::Chinese),
        "2 小时前"
    );
    assert_eq!(
        relative_time(now, now - (3 * MILLIS_PER_DAY), HubLanguage::Chinese),
        "3 天前"
    );
}

#[test]
fn task_summary_localizes_running_message_detail_and_operation_scope() {
    let mut snapshot = snapshot_with_projects(None, Vec::new());
    snapshot.settings.language = HubLanguage::Chinese;
    snapshot.task_status = TaskStatus::running_operation(
        "Packaging",
        HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::CopyingProjectToPackage,
        )),
        crate::state::TaskOperationKind::Project,
        "Game",
    );

    let model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(model.task_summary.label, "打包中");
    assert_eq!(model.task_summary.detail, "正在复制项目到包输出目录");
    assert_eq!(model.task_summary.operation, "项目: Game");
}

#[test]
fn task_summary_projects_backend_progress_percent() {
    let mut snapshot = snapshot_with_projects(None, Vec::new());
    snapshot.task_status = TaskStatus::running_operation(
        "Packaging",
        HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::CopyingProjectToPackage,
        )),
        crate::state::TaskOperationKind::Project,
        "Game",
    )
    .with_progress_percent(TASK_PROGRESS_PREPARED_PERCENT);

    let model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(
        model.task_summary.progress_percent,
        TASK_PROGRESS_PREPARED_PERCENT
    );
}

#[test]
fn task_summary_localizes_import_cancelled_status() {
    let mut snapshot = snapshot_with_projects(None, Vec::new());
    snapshot.settings.language = HubLanguage::Chinese;
    snapshot.task_status = TaskStatus::warning(
        "Import cancelled",
        HubMessage::new(HubMessageId::Project(
            ProjectMessageId::NoProjectFolderSelected,
        )),
        HubMessage::new(HubMessageId::Project(ProjectMessageId::RunImportAgain)),
    )
    .with_operation(crate::state::TaskOperationKind::Project, "Import Project");

    let model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(model.task_summary.label, "已取消导入");
    assert_eq!(model.task_summary.detail, "未选择项目文件夹");
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("重新运行导入项目并选择 Zircon 项目文件夹")
    );
}

#[test]
fn task_summary_localizes_backend_operation_targets() {
    let mut snapshot = snapshot_with_projects(None, Vec::new());
    snapshot.settings.language = HubLanguage::Chinese;
    snapshot.task_status = TaskStatus::error(
        "Open Output failed",
        HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::OpenOutputTargetRequired,
        )),
        HubMessage::new(HubMessageId::Delivery(
            DeliveryMessageId::ChooseRecordedOutputRecovery,
        )),
    )
    .with_operation(crate::state::TaskOperationKind::Process, "Output Folder");

    let output_model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(output_model.task_summary.operation, "进程: 输出文件夹");

    snapshot.task_status = TaskStatus::success(
        "Settings saved",
        HubMessage::with_params(
            HubMessageId::Settings(SettingsMessageId::SettingsSavedPath),
            ["E:\\Git\\ZirconEngine\\zircon_hub\\hub.toml"],
        ),
    )
    .with_operation(crate::state::TaskOperationKind::Settings, "Hub settings");

    let settings_model = HubViewModel::from_snapshot(&snapshot);

    assert_eq!(settings_model.task_summary.operation, "设置: Hub 设置");
}

fn snapshot_with_projects(
    selected_project_path: Option<PathBuf>,
    recent_projects: Vec<RecentProject>,
) -> HubSnapshot {
    HubSnapshot {
        selected_page: HubPage::Projects,
        project_filter: ProjectFilterMode::All,
        project_sort: ProjectSortMode::LastModified,
        project_view_mode: ProjectViewMode::Grid,
        project_subpage: ProjectSubpage::Dashboard,
        search_query: String::new(),
        selected_project_path,
        new_project_name: String::new(),
        selected_template_id: "renderable-empty".to_string(),
        new_project_location: PathBuf::from("E:/Projects"),
        new_project_engine_id: None,
        pending_delete_project_path: None,
        task_status: TaskStatus::idle(),
        queued_background_actions: 0,
        recent_projects,
        project_metadata: crate::projects::ProjectMetadataMap::new(),
        assets: Vec::new(),
        learn_resources: Vec::new(),
        plugins: Vec::new(),
        team: TeamOverview::empty(),
        action_history: Vec::new(),
        engines: Vec::new(),
        active_engine_id: None,
        settings: HubSettings::default(),
        settings_draft: HubSettings::default(),
    }
}

fn bind_source_engine(snapshot: &mut HubSnapshot, project_path: &PathBuf) {
    snapshot.engines.push(SourceEngineInstall {
        id: "source-local".to_string(),
        display_name: "Local Source".to_string(),
        source_dir: PathBuf::from("E:/Source/ZirconEngine"),
        output_dir: PathBuf::from("E:/Source/ZirconEngine/out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    snapshot.project_metadata.insert(
        project_metadata_key(project_path),
        ProjectMetadata {
            engine_id: Some("source-local".to_string()),
            ..ProjectMetadata::default()
        },
    );
}

fn quick_action<'a>(model: &'a HubViewModel, id: &str) -> &'a HubQuickAction {
    model
        .quick_actions
        .iter()
        .find(|action| action.id == id)
        .expect("quick action id should exist")
}
