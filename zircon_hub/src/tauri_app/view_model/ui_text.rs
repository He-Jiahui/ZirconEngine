use serde::Serialize;

use crate::settings::HubLanguage;

use super::localized::HubTextBundle;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubUiText {
    pub shell: HubShellText,
    pub actions: HubActionText,
    pub projects: HubProjectsText,
    pub common: HubCommonText,
    pub editor: HubEditorText,
    pub builds: HubBuildsText,
    pub catalog: HubCatalogText,
    pub cloud: HubCloudText,
    pub team: HubTeamText,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubShellText {
    pub product_category: String,
    pub workspace_profile: String,
    pub active_engine: String,
    pub ready_fallback: String,
    pub local_defaults: String,
    pub no_source_engine_registered: String,
    pub no_fallback_engine_configured: String,
    pub manage_engines: String,
    pub source: String,
    pub build_output: String,
    pub active: String,
    pub user_account: String,
    pub user_account_detail: String,
    pub preferences: String,
    pub preferences_detail: String,
    pub documentation: String,
    pub documentation_detail: String,
    pub sign_out: String,
    pub demo_mode_badge: String,
    pub live_updates_unavailable: String,
    pub live_updates_unavailable_detail: String,
    pub action_failed: String,
    pub action_failed_detail: String,
    pub state_refresh_after_command: String,
    pub check_action_target: String,
    pub nav_items: Vec<HubNavItemText>,
    pub engine_status: String,
    pub up_to_date: String,
    pub check_for_updates: String,
    pub check_for_updates_detail: String,
    pub collapse: String,
    pub expand: String,
    pub notifications: String,
    pub help: String,
    pub settings: String,
    pub minimize: String,
    pub maximize: String,
    pub close: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubNavItemText {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubActionText {
    pub import_project: String,
    pub new_project: String,
    pub create_project: String,
    pub close: String,
    pub dashboard: String,
    pub browser: String,
    pub open_editor: String,
    pub package_project: String,
    pub install_to_device: String,
    pub view_all_projects: String,
    pub pin_project: String,
    pub unpin_project: String,
    pub remove_from_hub: String,
    pub request_delete: String,
    pub cancel_delete: String,
    pub confirm_delete: String,
    pub browse_folder: String,
    pub open_resource: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubProjectsText {
    pub title: String,
    pub browser_title: String,
    pub detail_title: String,
    pub search_placeholder: String,
    pub filter_all: String,
    pub filter_existing: String,
    pub filter_missing: String,
    pub sort_last_modified: String,
    pub sort_name: String,
    pub grid_view: String,
    pub list_view: String,
    pub no_projects_found: String,
    pub search_filters_empty: String,
    pub no_recent_project_matches: String,
    pub project_browser: String,
    pub recent_projects: String,
    pub quick_actions: String,
    pub source_engines: String,
    pub all_projects: String,
    pub new_project_dialog: String,
    pub project_name: String,
    pub location: String,
    pub no_project_selected: String,
    pub choose_project_from_browser: String,
    pub status: String,
    pub ready: String,
    pub path_unavailable: String,
    pub engine: String,
    pub project_binding: String,
    pub last_modified: String,
    pub project_pin: String,
    pub pinned: String,
    pub unpinned: String,
    pub no_template: String,
    pub overview: String,
    pub files: String,
    pub actions: String,
    pub project_overview: String,
    pub project_tree: String,
    pub project_actions: String,
    pub source_engine: String,
    pub template: String,
    pub not_recorded: String,
    pub platform: String,
    pub project_id: String,
    pub content: String,
    pub available: String,
    pub missing: String,
    pub build_output: String,
    pub device_install: String,
    pub package: String,
    pub project_management: String,
    pub delete_requested: String,
    pub delete_requested_detail: String,
    pub table_name: String,
    pub table_engine_version: String,
    pub table_last_modified: String,
    pub table_location: String,
    pub open_project_details_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubCommonText {
    pub overview: String,
    pub plugins: String,
    pub activity: String,
    pub workflow: String,
    pub history: String,
    pub outputs: String,
    pub toolchain: String,
    pub packages: String,
    pub installs: String,
    pub services: String,
    pub selected_project: String,
    pub source_engines: String,
    pub quick_actions: String,
    pub project: String,
    pub engine: String,
    pub template: String,
    pub path: String,
    pub category: String,
    pub scope: String,
    pub target: String,
    pub finished: String,
    pub output: String,
    pub recovery: String,
    pub log: String,
    pub command: String,
    pub operation: String,
    pub detail: String,
    pub status: String,
    pub none: String,
    pub no_project_selected: String,
    pub no_selected_project: String,
    pub not_configured: String,
    pub configured: String,
    pub connected: String,
    pub available: String,
    pub ready: String,
    pub local: String,
    pub reserved: String,
    pub entries: String,
    pub actions: String,
    pub members: String,
    pub jobs: String,
    pub entry_count_template: String,
    pub available_count_template: String,
    pub reserved_count_template: String,
    pub member_count_template: String,
    pub action_count_template: String,
    pub no_output_directory: String,
    pub no_recovery_needed: String,
    pub no_log_excerpt: String,
    pub no_command_recorded: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubEditorText {
    pub workspace_tree: String,
    pub editor_workspace: String,
    pub selected_project: String,
    pub source_engines: String,
    pub source_build_history: String,
    pub editor_plugins: String,
    pub launch_target: String,
    pub launch_readiness: String,
    pub editor_plugin_scope: String,
    pub editor_activity: String,
    pub plugin_coming_soon_panel: String,
    pub no_project_selected_title: String,
    pub no_project_selected_detail: String,
    pub no_editor_plugins_title: String,
    pub no_editor_plugins_detail: String,
    pub no_editor_activity_title: String,
    pub no_editor_activity_detail: String,
    pub project_available: String,
    pub source_engine_registered: String,
    pub editor_plugin_scope_status: String,
    pub editor_packaging_scope: String,
    pub choose_project: String,
    pub no_template_recorded: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubBuildsText {
    pub build_project: String,
    pub package_project: String,
    pub install_to_device: String,
    pub build_button: String,
    pub package_button: String,
    pub install_button: String,
    pub build_workflow: String,
    pub build_history: String,
    pub latest_workflow: String,
    pub output_tree: String,
    pub output_folders: String,
    pub build_profile: String,
    pub output_root: String,
    pub recent_workflows: String,
    pub profile: String,
    pub jobs: String,
    pub device_install: String,
    pub compile_detail: String,
    pub package_detail: String,
    pub install_detail: String,
    pub no_build_history: String,
    pub no_build_history_detail: String,
    pub no_project_selected_detail: String,
    pub no_workflow_selected: String,
    pub no_workflow_selected_detail: String,
    pub open_output: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubCatalogText {
    pub search_placeholder_prefix: String,
    pub search_placeholder_separator: String,
    pub search_placeholder_suffix: String,
    pub entries: String,
    pub categories: String,
    pub scopes: String,
    pub catalog_suffix: String,
    pub assets_catalog_panel_title: String,
    pub plugins_catalog_panel_title: String,
    pub learn_catalog_panel_title: String,
    pub selected_entry: String,
    pub catalog_tree: String,
    pub all: String,
    pub project: String,
    pub engine: String,
    pub guides: String,
    pub reference: String,
    pub no_catalog: String,
    pub no_scope: String,
    pub no_entries_found: String,
    pub no_entries_found_detail: String,
    pub no_catalog_entry_selected: String,
    pub no_catalog_entry_selected_detail: String,
    pub module_count_suffix: String,
    pub module_count_template: String,
    pub coming_soon_panel: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubCloudText {
    pub local_delivery_tree: String,
    pub local_delivery_tree_detail: String,
    pub package_output: String,
    pub package_outputs: String,
    pub package_target: String,
    pub package_root: String,
    pub device_install: String,
    pub device_installs: String,
    pub install_readiness: String,
    pub service_slots: String,
    pub reserved_services: String,
    pub current_status: String,
    pub local_package_handoff: String,
    pub reserved_local_services: String,
    pub no_packages_recorded: String,
    pub no_packages_recorded_detail: String,
    pub no_installs_recorded: String,
    pub no_installs_recorded_detail: String,
    pub device_install_folder: String,
    pub package_history: String,
    pub package_actions_suffix: String,
    pub package_action_count_template: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubTeamText {
    pub repository: String,
    pub identity: String,
    pub contributors: String,
    pub team_members: String,
    pub repository_identity: String,
    pub team_tree: String,
    pub action_history: String,
    pub latest_action: String,
    pub git_name: String,
    pub git_email: String,
    pub name: String,
    pub email: String,
    pub unknown_contributor: String,
    pub no_email_configured: String,
    pub commit_singular_suffix: String,
    pub commit_plural_suffix: String,
    pub recent_actions: String,
    pub recent_action_count_template: String,
    pub no_team_members_found: String,
    pub no_team_members_found_detail: String,
    pub no_recent_actions: String,
    pub no_recent_actions_detail: String,
    pub no_action_selected: String,
    pub no_action_selected_detail: String,
    pub coming_soon_panel: String,
}

pub(crate) fn ui_text(language: HubLanguage) -> HubUiText {
    let text = HubTextBundle::new(language);
    HubUiText {
        shell: HubShellText {
            product_category: text.pair("Game Engine", "游戏引擎").to_string(),
            workspace_profile: text
                .pair("Zircon Hub workspace", "Zircon Hub 工作区")
                .to_string(),
            active_engine: text.pair("Active Engine", "当前引擎").to_string(),
            ready_fallback: text.pair("Ready Fallback", "备用引擎").to_string(),
            local_defaults: text.pair("Local Defaults", "本地默认值").to_string(),
            no_source_engine_registered: text
                .pair("No Source Engine registered", "未注册源码引擎")
                .to_string(),
            no_fallback_engine_configured: text
                .pair("No fallback engine configured", "未配置备用引擎")
                .to_string(),
            manage_engines: text.pair("Manage engines...", "管理引擎...").to_string(),
            source: text.pair("Source", "源码").to_string(),
            build_output: text.pair("Build Output", "构建输出").to_string(),
            active: text.pair("Active", "当前").to_string(),
            user_account: text.pair("My Account", "我的账户").to_string(),
            user_account_detail: text
                .pair("Profile and preferences", "个人资料和偏好")
                .to_string(),
            preferences: text.pair("Preferences", "偏好设置").to_string(),
            preferences_detail: text
                .pair("Hub settings and paths", "Hub 设置和路径")
                .to_string(),
            documentation: text.pair("Documentation", "文档").to_string(),
            documentation_detail: text
                .pair("Guides and local help", "指南和本地帮助")
                .to_string(),
            sign_out: text.pair("Sign Out", "退出").to_string(),
            demo_mode_badge: text.pair("Demo Data", "演示数据").to_string(),
            live_updates_unavailable: text
                .pair("Live updates unavailable", "实时更新不可用")
                .to_string(),
            live_updates_unavailable_detail: text
                .pair(
                    "Unable to subscribe to Hub state updates.",
                    "无法订阅 Hub 状态更新。",
                )
                .to_string(),
            action_failed: text.pair("Action failed", "操作失败").to_string(),
            action_failed_detail: text
                .pair(
                    "The Hub backend could not complete this action.",
                    "Hub 后端未能完成该操作。",
                )
                .to_string(),
            state_refresh_after_command: text
                .pair(
                    "State will still refresh after each command completes",
                    "命令完成后仍会刷新状态",
                )
                .to_string(),
            check_action_target: text
                .pair(
                    "Check the action target and try again",
                    "检查操作目标后重试",
                )
                .to_string(),
            nav_items: nav_items(language),
            engine_status: text.pair("Engine Status", "引擎状态").to_string(),
            up_to_date: text.pair("Local version", "本地版本").to_string(),
            check_for_updates: text
                .pair("Update check reserved", "更新检查预留")
                .to_string(),
            check_for_updates_detail: text
                .pair(
                    "Remote update service is not enabled in local v1.",
                    "本地 v1 不启用远程更新服务。",
                )
                .to_string(),
            collapse: text.pair("Collapse", "收起").to_string(),
            expand: text.pair("Expand", "展开").to_string(),
            notifications: text.pair("Notifications", "通知").to_string(),
            help: text.pair("Help", "帮助").to_string(),
            settings: text.pair("Settings", "设置").to_string(),
            minimize: text.pair("Minimize", "最小化").to_string(),
            maximize: text.pair("Maximize", "最大化").to_string(),
            close: text.pair("Close", "关闭").to_string(),
        },
        actions: HubActionText {
            import_project: text.pair("Import Project", "导入项目").to_string(),
            new_project: text.pair("New Project", "新建项目").to_string(),
            create_project: text.pair("Create Project", "创建项目").to_string(),
            close: text.pair("Close", "关闭").to_string(),
            dashboard: text.pair("Dashboard", "仪表盘").to_string(),
            browser: text.pair("Browser", "浏览器").to_string(),
            open_editor: text.pair("Open Editor", "打开编辑器").to_string(),
            package_project: text.pair("Package Project", "打包项目").to_string(),
            install_to_device: text.pair("Install to Device", "安装到设备").to_string(),
            view_all_projects: text.pair("View All Projects", "查看全部项目").to_string(),
            pin_project: text.pair("Pin Project", "置顶项目").to_string(),
            unpin_project: text.pair("Unpin Project", "取消置顶").to_string(),
            remove_from_hub: text.pair("Remove from Hub", "从 Hub 移除").to_string(),
            request_delete: text.pair("Delete Project", "删除项目").to_string(),
            cancel_delete: text.pair("Cancel Delete", "取消删除").to_string(),
            confirm_delete: text.pair("Confirm Delete", "确认删除").to_string(),
            browse_folder: text.pair("Browse Folder", "浏览文件夹").to_string(),
            open_resource: text.pair("Open Resource", "打开资源").to_string(),
        },
        projects: HubProjectsText {
            title: text.pair("Projects", "项目").to_string(),
            browser_title: text.pair("Project Browser", "项目浏览器").to_string(),
            detail_title: text.pair("Project Detail", "项目详情").to_string(),
            search_placeholder: text.pair("Search projects...", "搜索项目...").to_string(),
            filter_all: text.pair("All Projects", "全部项目").to_string(),
            filter_existing: text.pair("Existing", "存在").to_string(),
            filter_missing: text.pair("Missing", "缺失").to_string(),
            sort_last_modified: text.pair("Last Modified", "最近修改").to_string(),
            sort_name: text.pair("Name", "名称").to_string(),
            grid_view: text.pair("Grid view", "网格视图").to_string(),
            list_view: text.pair("List view", "列表视图").to_string(),
            no_projects_found: text.pair("No projects found", "未找到项目").to_string(),
            search_filters_empty: text
                .pair(
                    "Search and filters hide every recent project",
                    "搜索和筛选隐藏了所有最近项目",
                )
                .to_string(),
            no_recent_project_matches: text
                .pair(
                    "No recent project matches the current view",
                    "没有最近项目匹配当前视图",
                )
                .to_string(),
            project_browser: text.pair("Project Browser", "项目浏览器").to_string(),
            recent_projects: text.pair("Recent Projects", "最近项目").to_string(),
            quick_actions: text.pair("Quick Actions", "快捷操作").to_string(),
            source_engines: text.pair("Source Engines", "源码引擎").to_string(),
            all_projects: text.pair("All Projects", "全部项目").to_string(),
            new_project_dialog: text.pair("New Project", "新建项目").to_string(),
            project_name: text.pair("Project Name", "项目名称").to_string(),
            location: text.pair("Location", "位置").to_string(),
            no_project_selected: text.pair("No project selected", "未选择项目").to_string(),
            choose_project_from_browser: text
                .pair(
                    "Choose a project from the browser",
                    "从项目浏览器中选择项目",
                )
                .to_string(),
            status: text.pair("Status", "状态").to_string(),
            ready: text.pair("Ready", "就绪").to_string(),
            path_unavailable: text.pair("Path unavailable", "路径不可用").to_string(),
            engine: text.pair("Engine", "引擎").to_string(),
            project_binding: text.pair("Project binding", "项目绑定").to_string(),
            last_modified: text.pair("Last Modified", "最近修改").to_string(),
            project_pin: text.pair("Project Pin", "项目置顶").to_string(),
            pinned: text.pair("Pinned", "已置顶").to_string(),
            unpinned: text.pair("Unpinned", "未置顶").to_string(),
            no_template: text.pair("No template", "无模板").to_string(),
            overview: text.pair("Overview", "概览").to_string(),
            files: text.pair("Files", "文件").to_string(),
            actions: text.pair("Actions", "操作").to_string(),
            project_overview: text.pair("Project Overview", "项目概览").to_string(),
            project_tree: text.pair("Project Tree", "项目树").to_string(),
            project_actions: text.pair("Project Actions", "项目操作").to_string(),
            source_engine: text.pair("Source Engine", "源码引擎").to_string(),
            template: text.pair("Template", "模板").to_string(),
            not_recorded: text.pair("Not recorded", "未记录").to_string(),
            platform: text.pair("Platform", "平台").to_string(),
            project_id: text.pair("Project ID", "项目 ID").to_string(),
            content: text.pair("Content", "内容").to_string(),
            available: text.pair("Available", "可用").to_string(),
            missing: text.pair("Missing", "缺失").to_string(),
            build_output: text.pair("Build Output", "构建输出").to_string(),
            device_install: text.pair("Device Install", "设备安装").to_string(),
            package: text.pair("Package", "包").to_string(),
            project_management: text.pair("Project Management", "项目管理").to_string(),
            delete_requested: text
                .pair("Delete confirmation required", "需要确认删除")
                .to_string(),
            delete_requested_detail: text
                .pair(
                    "Confirm delete moves the project folder to the Windows Recycle Bin.",
                    "确认删除会将项目文件夹移动到 Windows 回收站。",
                )
                .to_string(),
            table_name: text.pair("Name", "名称").to_string(),
            table_engine_version: text.pair("Engine Version", "引擎版本").to_string(),
            table_last_modified: text.pair("Last Modified", "最近修改").to_string(),
            table_location: text.pair("Location", "位置").to_string(),
            open_project_details_label: text
                .pair("Open project details", "打开项目详情")
                .to_string(),
        },
        common: common_text(text),
        editor: editor_text(text),
        builds: builds_text(text),
        catalog: catalog_text(text),
        cloud: cloud_text(text),
        team: team_text(text),
    }
}

fn common_text(text: HubTextBundle) -> HubCommonText {
    HubCommonText {
        overview: text.pair("Overview", "概览").to_string(),
        plugins: text.pair("Plugins", "插件").to_string(),
        activity: text.pair("Activity", "活动").to_string(),
        workflow: text.pair("Workflow", "工作流").to_string(),
        history: text.pair("History", "历史").to_string(),
        outputs: text.pair("Outputs", "输出").to_string(),
        toolchain: text.pair("Toolchain", "工具链").to_string(),
        packages: text.pair("Packages", "包").to_string(),
        installs: text.pair("Installs", "安装").to_string(),
        services: text.pair("Services", "服务").to_string(),
        selected_project: text.pair("Selected Project", "已选项目").to_string(),
        source_engines: text.pair("Source Engines", "源码引擎").to_string(),
        quick_actions: text.pair("Quick Actions", "快捷操作").to_string(),
        project: text.pair("Project", "项目").to_string(),
        engine: text.pair("Engine", "引擎").to_string(),
        template: text.pair("Template", "模板").to_string(),
        path: text.pair("Path", "路径").to_string(),
        category: text.pair("Category", "分类").to_string(),
        scope: text.pair("Scope", "范围").to_string(),
        target: text.pair("Target", "目标").to_string(),
        finished: text.pair("Finished", "完成时间").to_string(),
        output: text.pair("Output", "输出").to_string(),
        recovery: text.pair("Recovery", "恢复建议").to_string(),
        log: text.pair("Log", "日志").to_string(),
        command: text.pair("Command", "命令").to_string(),
        operation: text.pair("Operation", "操作").to_string(),
        detail: text.pair("Detail", "详情").to_string(),
        status: text.pair("Status", "状态").to_string(),
        none: text.pair("None", "无").to_string(),
        no_project_selected: text.pair("No project selected", "未选择项目").to_string(),
        no_selected_project: text.pair("No selected project", "没有已选项目").to_string(),
        not_configured: text.pair("Not configured", "未配置").to_string(),
        configured: text.pair("Configured", "已配置").to_string(),
        connected: text.pair("Connected", "已连接").to_string(),
        available: text.pair("Available", "可用").to_string(),
        ready: text.pair("Ready", "就绪").to_string(),
        local: text.pair("Local", "本地").to_string(),
        reserved: text.pair("Reserved", "预留").to_string(),
        entries: text.pair("entries", "条目").to_string(),
        actions: text.pair("actions", "操作").to_string(),
        members: text.pair("members", "成员").to_string(),
        jobs: text.pair("jobs", "任务").to_string(),
        entry_count_template: text.pair("{count} entries", "{count} 个条目").to_string(),
        available_count_template: text.pair("{count} available", "{count} 个可用").to_string(),
        reserved_count_template: text.pair("{count} reserved", "{count} 个预留").to_string(),
        member_count_template: text.pair("{count} members", "{count} 位成员").to_string(),
        action_count_template: text.pair("{count} actions", "{count} 次操作").to_string(),
        no_output_directory: text.pair("No output directory", "没有输出目录").to_string(),
        no_recovery_needed: text.pair("No recovery needed", "无需恢复操作").to_string(),
        no_log_excerpt: text.pair("No log excerpt", "没有日志摘录").to_string(),
        no_command_recorded: text.pair("No command recorded", "没有记录命令").to_string(),
    }
}

fn editor_text(text: HubTextBundle) -> HubEditorText {
    HubEditorText {
        workspace_tree: text.pair("Workspace Tree", "工作区树").to_string(),
        editor_workspace: text.pair("Editor Workspace", "编辑器工作区").to_string(),
        selected_project: text.pair("Selected Project", "已选项目").to_string(),
        source_engines: text.pair("Source Engines", "源码引擎").to_string(),
        source_build_history: text
            .pair("Source Engine Build History", "源码引擎构建历史")
            .to_string(),
        editor_plugins: text.pair("Editor Plugins", "编辑器插件").to_string(),
        launch_target: text.pair("Launch Target", "启动目标").to_string(),
        launch_readiness: text.pair("Launch Readiness", "启动就绪状态").to_string(),
        editor_plugin_scope: text
            .pair("Editor Plugin Scope", "编辑器插件范围")
            .to_string(),
        editor_activity: text.pair("Editor Activity", "编辑器活动").to_string(),
        plugin_coming_soon_panel: text
            .pair("Reserved Plugin Operations", "预留插件操作")
            .to_string(),
        no_project_selected_title: text.pair("No project selected", "未选择项目").to_string(),
        no_project_selected_detail: text
            .pair(
                "Choose a project from the browser, or open an empty editor.",
                "从项目浏览器选择项目，或直接打开空编辑器。",
            )
            .to_string(),
        no_editor_plugins_title: text
            .pair("No editor plugins found", "未找到编辑器插件")
            .to_string(),
        no_editor_plugins_detail: text
            .pair(
                "Editor-scoped plugins will appear after catalog discovery.",
                "目录发现后会显示编辑器范围插件。",
            )
            .to_string(),
        no_editor_activity_title: text
            .pair("No editor activity", "没有编辑器活动")
            .to_string(),
        no_editor_activity_detail: text
            .pair(
                "Open Editor and build actions will appear here.",
                "打开编辑器和构建操作会显示在这里。",
            )
            .to_string(),
        project_available: text.pair("Project Available", "项目可用").to_string(),
        source_engine_registered: text
            .pair("Source Engine Registered", "源码引擎已注册")
            .to_string(),
        editor_plugin_scope_status: text
            .pair("Editor Plugin Scope", "编辑器插件范围")
            .to_string(),
        editor_packaging_scope: text
            .pair("Editor packaging scope", "编辑器打包范围")
            .to_string(),
        choose_project: text.pair("Choose a project", "选择项目").to_string(),
        no_template_recorded: text.pair("No template recorded", "未记录模板").to_string(),
    }
}

fn builds_text(text: HubTextBundle) -> HubBuildsText {
    HubBuildsText {
        build_project: text.pair("Build Project", "构建项目").to_string(),
        package_project: text.pair("Package Project", "打包项目").to_string(),
        install_to_device: text.pair("Install to Device", "安装到设备").to_string(),
        build_button: text.pair("Build", "构建").to_string(),
        package_button: text.pair("Package", "打包").to_string(),
        install_button: text.pair("Install", "安装").to_string(),
        build_workflow: text.pair("Build Workflow", "构建工作流").to_string(),
        build_history: text.pair("Build History", "构建历史").to_string(),
        latest_workflow: text.pair("Latest Workflow", "最近工作流").to_string(),
        output_tree: text.pair("Output Tree", "输出树").to_string(),
        output_folders: text.pair("Output Folders", "输出文件夹").to_string(),
        build_profile: text.pair("Build Profile", "构建配置").to_string(),
        output_root: text.pair("Output Root", "输出根目录").to_string(),
        recent_workflows: text.pair("Recent Workflows", "最近工作流").to_string(),
        profile: text.pair("Profile", "配置").to_string(),
        jobs: text.pair("Jobs", "任务").to_string(),
        device_install: text.pair("Device Install", "设备安装").to_string(),
        compile_detail: text
            .pair(
                "Compile editor/runtime targets with configured build defaults",
                "使用当前构建默认值编译编辑器/运行时目标",
            )
            .to_string(),
        package_detail: text
            .pair(
                "Create distributable project output for local deployment",
                "为本地交付创建可分发项目输出",
            )
            .to_string(),
        install_detail: text
            .pair(
                "Copy the latest package into the configured device staging folder",
                "将最新包复制到配置的设备暂存目录",
            )
            .to_string(),
        no_build_history: text.pair("No build history", "没有构建历史").to_string(),
        no_build_history_detail: text
            .pair(
                "Build, package, and install actions will appear here.",
                "构建、打包和安装操作会显示在这里。",
            )
            .to_string(),
        no_project_selected_detail: text
            .pair(
                "Select a project before running build workflows.",
                "运行构建工作流前先选择项目。",
            )
            .to_string(),
        no_workflow_selected: text
            .pair("No workflow selected", "未选择工作流")
            .to_string(),
        no_workflow_selected_detail: text
            .pair(
                "Run a build workflow to populate this panel.",
                "运行构建工作流后会填充此面板。",
            )
            .to_string(),
        open_output: text.pair("Open Output", "打开输出").to_string(),
    }
}

fn catalog_text(text: HubTextBundle) -> HubCatalogText {
    HubCatalogText {
        search_placeholder_prefix: text.pair("Search", "搜索").to_string(),
        search_placeholder_separator: text.pair(" ", "").to_string(),
        search_placeholder_suffix: text.pair("...", "...").to_string(),
        entries: text.pair("Entries", "条目").to_string(),
        categories: text.pair("Categories", "分类").to_string(),
        scopes: text.pair("Scopes", "范围").to_string(),
        catalog_suffix: text.pair("Catalog", "目录").to_string(),
        assets_catalog_panel_title: text.pair("Assets Catalog", "资产目录").to_string(),
        plugins_catalog_panel_title: text.pair("Plugins Catalog", "插件目录").to_string(),
        learn_catalog_panel_title: text.pair("Learn Catalog", "学习目录").to_string(),
        selected_entry: text.pair("Selected Entry", "已选条目").to_string(),
        catalog_tree: text.pair("Catalog Tree", "目录树").to_string(),
        all: text.pair("All", "全部").to_string(),
        project: text.pair("Project", "项目").to_string(),
        engine: text.pair("Engine", "引擎").to_string(),
        guides: text.pair("Guides", "指南").to_string(),
        reference: text.pair("Reference", "参考").to_string(),
        no_catalog: text.pair("No catalog", "没有目录").to_string(),
        no_scope: text.pair("No scope", "没有范围").to_string(),
        no_entries_found: text.pair("No entries found", "未找到条目").to_string(),
        no_entries_found_detail: text
            .pair(
                "Adjust the search or catalog tab to view more entries.",
                "调整搜索或目录标签以查看更多条目。",
            )
            .to_string(),
        no_catalog_entry_selected: text
            .pair("No catalog entry selected", "未选择目录条目")
            .to_string(),
        no_catalog_entry_selected_detail: text
            .pair(
                "Catalog data will appear after discovery completes.",
                "目录发现完成后会显示数据。",
            )
            .to_string(),
        module_count_suffix: text.pair("modules", "模块").to_string(),
        module_count_template: text.pair("{count} modules", "{count} 个模块").to_string(),
        coming_soon_panel: text.pair("Reserved Capabilities", "预留能力").to_string(),
    }
}

fn cloud_text(text: HubTextBundle) -> HubCloudText {
    HubCloudText {
        local_delivery_tree: text.pair("Local Delivery Tree", "本地交付树").to_string(),
        local_delivery_tree_detail: text
            .pair("Package and install handoff", "包输出与安装交接")
            .to_string(),
        package_output: text.pair("Package Output", "包输出").to_string(),
        package_outputs: text.pair("Package Outputs", "包输出").to_string(),
        package_target: text.pair("Package Target", "打包目标").to_string(),
        package_root: text.pair("Package Root", "包根目录").to_string(),
        device_install: text.pair("Device Install", "设备安装").to_string(),
        device_installs: text.pair("Device Installs", "设备安装记录").to_string(),
        install_readiness: text.pair("Install Readiness", "安装就绪状态").to_string(),
        service_slots: text.pair("Service Slots", "服务槽位").to_string(),
        reserved_services: text.pair("Reserved Services", "预留服务").to_string(),
        current_status: text.pair("Current Status", "当前状态").to_string(),
        local_package_handoff: text.pair("Local package handoff", "本地包交付").to_string(),
        reserved_local_services: text
            .pair("Reserved local services", "预留本地服务")
            .to_string(),
        no_packages_recorded: text.pair("No packages recorded", "没有包记录").to_string(),
        no_packages_recorded_detail: text
            .pair(
                "Package Project actions will appear in this local output view.",
                "打包项目操作会显示在此本地输出视图中。",
            )
            .to_string(),
        no_installs_recorded: text
            .pair("No installs recorded", "没有安装记录")
            .to_string(),
        no_installs_recorded_detail: text
            .pair(
                "Install to Device actions will appear here.",
                "安装到设备操作会显示在这里。",
            )
            .to_string(),
        device_install_folder: text
            .pair("Device Install Folder", "设备安装文件夹")
            .to_string(),
        package_history: text.pair("Package History", "包历史").to_string(),
        package_actions_suffix: text.pair("package actions", "次打包操作").to_string(),
        package_action_count_template: text
            .pair("{count} package actions", "{count} 次打包操作")
            .to_string(),
    }
}

fn team_text(text: HubTextBundle) -> HubTeamText {
    HubTeamText {
        repository: text.pair("Repository", "仓库").to_string(),
        identity: text.pair("Identity", "身份").to_string(),
        contributors: text.pair("Contributors", "贡献者").to_string(),
        team_members: text.pair("Team Members", "团队成员").to_string(),
        repository_identity: text.pair("Repository Identity", "仓库身份").to_string(),
        team_tree: text.pair("Team Tree", "团队树").to_string(),
        action_history: text.pair("Action History", "操作历史").to_string(),
        latest_action: text.pair("Latest Action", "最近操作").to_string(),
        git_name: text.pair("Git Name", "Git 名称").to_string(),
        git_email: text.pair("Git Email", "Git 邮箱").to_string(),
        name: text.pair("Name", "名称").to_string(),
        email: text.pair("Email", "邮箱").to_string(),
        unknown_contributor: text.pair("Unknown Contributor", "未知贡献者").to_string(),
        no_email_configured: text.pair("No email configured", "未配置邮箱").to_string(),
        commit_singular_suffix: text.pair("commit", "次提交").to_string(),
        commit_plural_suffix: text.pair("commits", "次提交").to_string(),
        recent_actions: text.pair("recent actions", "最近操作").to_string(),
        recent_action_count_template: text
            .pair("{count} recent actions", "{count} 次最近操作")
            .to_string(),
        no_team_members_found: text
            .pair("No team members found", "未找到团队成员")
            .to_string(),
        no_team_members_found_detail: text
            .pair(
                "Git contribution data will appear when repository history is available.",
                "仓库历史可用时会显示 Git 贡献数据。",
            )
            .to_string(),
        no_recent_actions: text.pair("No recent actions", "没有最近操作").to_string(),
        no_recent_actions_detail: text
            .pair(
                "Build, package, install, and editor launches will appear here.",
                "构建、打包、安装和编辑器启动会显示在这里。",
            )
            .to_string(),
        no_action_selected: text.pair("No action selected", "未选择操作").to_string(),
        no_action_selected_detail: text
            .pair(
                "Run a quick action to populate this panel.",
                "运行快捷操作后会填充此面板。",
            )
            .to_string(),
        coming_soon_panel: text
            .pair("Reserved Collaboration", "预留协作能力")
            .to_string(),
    }
}

fn nav_items(language: HubLanguage) -> Vec<HubNavItemText> {
    [
        ("projects", "Projects", "项目"),
        ("editor", "Editor", "编辑器"),
        ("assets", "Assets", "资产"),
        ("builds", "Builds", "构建"),
        ("plugins", "Plugins", "插件"),
        ("cloud", "Local Delivery", "本地交付"),
        ("team", "Team", "团队"),
        ("learn", "Learn", "学习"),
        ("settings", "Settings", "设置"),
    ]
    .into_iter()
    .map(|(id, english, chinese)| HubNavItemText {
        id: id.to_string(),
        label: match language {
            HubLanguage::English => english,
            HubLanguage::Chinese => chinese,
        }
        .to_string(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::settings::HubLanguage;

    #[test]
    fn ui_text_defaults_to_chinese_shell_and_project_copy() {
        let text = super::ui_text(HubLanguage::Chinese);

        assert_eq!(text.shell.action_failed, "操作失败");
        assert_eq!(text.shell.nav_items[0].label, "项目");
        assert_eq!(text.actions.new_project, "新建项目");
        assert_eq!(text.actions.open_resource, "打开资源");
        assert_eq!(text.editor.plugin_coming_soon_panel, "预留插件操作");
        assert_eq!(text.shell.active_engine, "当前引擎");
        assert_eq!(text.shell.no_source_engine_registered, "未注册源码引擎");
        assert_eq!(text.shell.user_account, "我的账户");
        assert_eq!(text.shell.workspace_profile, "Zircon Hub 工作区");
        assert_eq!(text.shell.up_to_date, "本地版本");
        assert_eq!(text.shell.check_for_updates, "更新检查预留");
        assert_eq!(
            text.shell.check_for_updates_detail,
            "本地 v1 不启用远程更新服务。"
        );
        assert_eq!(text.shell.expand, "展开");
        assert_eq!(text.shell.demo_mode_badge, "演示数据");
        assert_eq!(text.projects.search_placeholder, "搜索项目...");
        assert_eq!(text.catalog.search_placeholder_prefix, "搜索");
        assert_eq!(text.catalog.search_placeholder_separator, "");
        assert_eq!(text.catalog.search_placeholder_suffix, "...");
    }

    #[test]
    fn ui_text_strings_are_non_empty_except_explicit_separator() {
        for language in [HubLanguage::English, HubLanguage::Chinese] {
            let value =
                serde_json::to_value(super::ui_text(language)).expect("ui text should serialize");

            assert_non_empty_strings(&value, "");
        }
    }

    fn assert_non_empty_strings(value: &serde_json::Value, path: &str) {
        match value {
            serde_json::Value::String(text) => {
                if path == "catalog.searchPlaceholderSeparator" {
                    return;
                }
                assert!(!text.trim().is_empty(), "empty UI text at {path}");
            }
            serde_json::Value::Array(values) => {
                for (index, child) in values.iter().enumerate() {
                    assert_non_empty_strings(child, &format!("{path}[{index}]"));
                }
            }
            serde_json::Value::Object(fields) => {
                for (key, child) in fields {
                    let next_path = if path.is_empty() {
                        key.to_string()
                    } else {
                        format!("{path}.{key}")
                    };
                    assert_non_empty_strings(child, &next_path);
                }
            }
            _ => {}
        }
    }
}
