use super::*;
use crate::state::{
    DeliveryMessageId, EngineMessageId, HubMessage, HubMessageId, LearnMessageId, ProcessMessageId,
    ProjectMessageId, SettingsMessageId, ShellMessageId,
};

#[test]
fn chinese_bundle_localizes_page_status_and_action_copy() {
    let bundle = HubTextBundle::new(HubLanguage::Chinese);

    assert_eq!(bundle.page_title(HubPage::Settings), "设置");
    assert_eq!(bundle.status_label("Ready"), "就绪");
    assert_eq!(
        bundle.render_message(&HubMessage::new(HubMessageId::Shell(
            ShellMessageId::HubReady,
        ))),
        "Hub 已就绪"
    );
    assert_eq!(bundle.operation_scope(TaskOperationKind::Settings), "设置");
    assert_eq!(
        bundle.action_label(HubActionKind::BuildEditorRuntime),
        "构建编辑器/运行时"
    );
    assert_eq!(bundle.action_status_label(HubActionStatus::Failed), "失败");
    assert_eq!(bundle.status_label("Import cancelled"), "已取消导入");
    assert_eq!(bundle.status_label("Save Settings failed"), "保存设置失败");
    assert_eq!(bundle.status_label("Projects filtered"), "项目已筛选");
    assert_eq!(bundle.operation_target("Output Folder"), "输出文件夹");
    assert_eq!(bundle.operation_target("Hub settings"), "Hub 设置");
    assert_eq!(
        bundle.operation_target("Settings source checkout"),
        "设置中的源码检出目录"
    );
    assert_eq!(
        HubTextBundle::new(HubLanguage::English).operation_target("Settings source checkout"),
        "Settings source checkout"
    );
}

#[test]
fn render_message_localizes_dynamic_project_engine_and_process_templates() {
    let bundle = HubTextBundle::new(HubLanguage::Chinese);

    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::TemplateComingSoon),
            ["3d-scene"],
        )),
        "项目模板尚未开放：3d-scene"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::ManifestNotFound),
            ["C:\\Projects\\Missing"],
        )),
        "未在 C:\\Projects\\Missing 找到 zircon-project.toml"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::FolderCreatedButRecordFailed),
            ["C:\\Projects\\Game", "save failed"],
        )),
        "项目目录已创建于 C:\\Projects\\Game，但 Hub 记录失败：save failed"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::NoBoundSourceEngine),
            ["Game"],
        )),
        "项目未绑定源码引擎：Game"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Engine(EngineMessageId::UnknownSourceEngine),
            ["source-missing"],
        )),
        "未知源码引擎：source-missing"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Process(ProcessMessageId::OpeningTargetProcess),
            ["Game", "42"],
        )),
        "正在打开 Game（进程 42）"
    );
}

#[test]
fn render_message_localizes_delivery_settings_learn_and_shell_templates() {
    let bundle = HubTextBundle::new(HubLanguage::Chinese);

    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::FileCountDetail),
            ["Game", "C:\\Packages\\Game", "2"],
        )),
        "Game -> C:\\Packages\\Game（2 个文件）"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::PackageLogExcerpt),
            ["Game", "C:\\Packages\\Game", "2"],
        )),
        "已打包 Game 到 C:\\Packages\\Game（2 个文件）"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::OutputFolderNotRecorded),
            ["C:\\Users\\attacker"],
        )),
        "输出文件夹不是 Hub 已记录的输出：C:\\Users\\attacker"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Learn(LearnMessageId::ResourceFileDoesNotExist),
            ["C:\\Docs\\guide.md"],
        )),
        "资源文件不存在：C:\\Docs\\guide.md"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Shell(ShellMessageId::OpenedPath),
            ["C:\\Packages\\Game"],
        )),
        "已打开 C:\\Packages\\Game"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::with_params(
            HubMessageId::Settings(SettingsMessageId::UnknownLanguage),
            ["Klingon"],
        )),
        "未知 Hub 语言：Klingon"
    );
    assert_eq!(
        bundle.render_message(&HubMessage::new(HubMessageId::Settings(
            SettingsMessageId::DraftRestoredDefaults,
        ))),
        "草稿已恢复为内置默认值"
    );
}
