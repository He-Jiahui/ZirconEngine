use slint::SharedString;

use crate::settings::HubLanguage;
use crate::state::{HubPage, ProjectFilterMode, ProjectSortMode};

use super::UiTextData;

pub(super) fn ui_text(language: HubLanguage) -> UiTextData {
    UiTextData {
        game_engine: text(language, "Game Engine", "游戏引擎"),
        local_user: text(language, "Local User", "本地用户"),
        local_user_initials: text(language, "LU", "本"),
        engine_status: text(language, "Engine Status", "引擎状态"),
        check_for_updates: text(language, "Check for Updates", "检查更新"),
        collapse: text(language, "Collapse", "折叠"),
        source_engines: text(language, "Source Engines", "源码引擎"),
        registered: text(language, "Registered", "已注册"),
        no_source_engines: text(
            language,
            "No source engines registered.",
            "尚未注册源码引擎。",
        ),
        open_editor: text(language, "Open Editor", "打开编辑器"),
        back_to_projects: text(language, "Back to Projects", "返回项目"),
        placeholder_reserved: text(
            language,
            "This section is wired into the Hub shell and reserved for the next feature slice.",
            "该页面已接入 Hub 外壳，后续功能切片会继续完善。",
        ),
        projects_empty_title: text(language, "No recent projects", "暂无最近项目"),
        projects_empty_detail: text(
            language,
            "Create a project or enter a project path to open an existing one.",
            "创建新项目，或输入项目路径打开已有项目。",
        ),
        project_list: text(language, "Project List", "项目列表"),
        recent_projects: text(language, "Recent Projects", "最近项目"),
        name_column: text(language, "Name", "名称"),
        engine_version_column: text(language, "Engine Version", "引擎版本"),
        last_modified_column: text(language, "Last Modified", "最近修改"),
        location_column: text(language, "Location", "位置"),
        quick_actions: text(language, "Quick Actions", "快捷操作"),
        open_path: text(language, "Open Path", "打开路径"),
        view_all_projects: text(language, "View All Projects", "查看全部项目"),
        create_project: text(language, "New Project", "新建项目"),
        import_project: text(language, "Import Project", "导入项目"),
        project_root_path: text(language, "Project root path", "项目根目录"),
        project_name: text(language, "Project name", "项目名称"),
        location: text(language, "Location", "位置"),
        browse: text(language, "Browse", "浏览"),
        open: text(language, "Open", "打开"),
        create: text(language, "Create", "创建"),
        search_projects: text(language, "Search projects...", "搜索项目..."),
        no_projects_match: text(
            language,
            "No projects match the current search.",
            "没有项目匹配当前搜索。",
        ),
        show_more_projects: text(language, "Show More", "展示更多"),
        collapse_projects: text(language, "Collapse", "收起"),
        modified_prefix: text(language, "Modified ", "修改于 "),
        editor_actions: text(language, "Engine Actions", "引擎操作"),
        save_source: text(language, "Save Source", "保存源码"),
        build: text(language, "Build", "构建"),
        open_output: text(language, "Open Output", "打开输出"),
        source_engine: text(language, "Source Engine", "源码引擎"),
        active_engine_name: text(language, "Active engine name", "当前引擎名称"),
        rename: text(language, "Rename", "重命名"),
        source_checkout_path: text(language, "Source checkout path", "源码检出路径"),
        staged_output_directory: text(language, "Staged output directory", "生成产物目录"),
        build_history: text(language, "Build History", "构建历史"),
        no_build_history: text(
            language,
            "No build history has been recorded for the active source engine.",
            "当前源码引擎还没有构建历史。",
        ),
        toolchain: text(language, "Toolchain", "工具链"),
        python_executable: text(language, "Python executable", "Python 指令"),
        cargo_executable: text(language, "Cargo executable", "Cargo 指令"),
        rustup_executable: text(language, "Rustup executable", "Rustup 指令"),
        build_defaults: text(language, "Build Defaults", "构建默认值"),
        build_profile: text(language, "Build profile", "构建配置"),
        debug: text(language, "Debug", "调试"),
        release: text(language, "Release", "发布"),
        build_jobs: text(language, "Build jobs", "构建任务数"),
        language: text(language, "Language", "语言"),
        english: text(language, "English", "英文"),
        chinese: text(language, "Chinese", "中文"),
        default_paths: text(language, "Default Paths", "默认路径"),
        default_project_directory: text(language, "Default project directory", "默认项目目录"),
        default_source_directory: text(language, "Default source directory", "默认源码目录"),
        default_staged_output_directory: text(
            language,
            "Default staged output directory",
            "默认生成产物目录",
        ),
        default_device_install_directory: text(
            language,
            "Default device install directory",
            "默认设备安装目录",
        ),
        configuration_health: text(language, "Configuration Health", "配置健康状态"),
        save_settings: text(language, "Save Settings", "保存设置"),
        source_build: text(language, "Source Build", "源码构建"),
        build_now: text(language, "Build Now", "立即构建"),
        source_prefix: text(language, "Source: ", "源码："),
        output_prefix: text(language, "Output: ", "输出："),
        last_build_prefix: text(language, "Last build: ", "最近构建："),
        jobs_suffix: text(language, " jobs", " 任务"),
        build_controls: text(language, "Build Controls", "构建控制"),
        build_editor: text(language, "Build Editor", "构建编辑器"),
        build_pipeline: text(language, "Build Pipeline", "构建流水线"),
        validate_source: text(language, "Validate Source", "验证源码"),
        compile_editor: text(language, "Compile Editor", "编译编辑器"),
        stage_runtime: text(language, "Stage Runtime", "暂存运行时"),
        package_project: text(language, "Package Project", "打包项目"),
        reserved_project_export: text(language, "Reserved for project export", "预留给项目导出"),
        unavailable: text(language, "Unavailable", "不可用"),
        current_task: text(language, "Current Task", "当前任务"),
        profile_prefix: text(language, "Profile ", "配置 "),
        asset_catalog: text(language, "Asset Catalog", "资产目录"),
        assets_found: text(language, "assets found", "个资产"),
        no_assets_found: text(
            language,
            "No assets were found in recent project folders or the active source checkout.",
            "最近项目目录或当前源码检出中没有发现资产。",
        ),
        asset_kind: text(language, "Kind", "类型"),
        asset_source: text(language, "Source", "来源"),
        asset_size: text(language, "Size", "大小"),
        learn_library: text(language, "Learning Library", "学习资料库"),
        learn_resources_found: text(language, "resources found", "个资源"),
        no_learn_resources_found: text(
            language,
            "No local documentation was found in the active source checkout.",
            "当前源码检出中没有发现本地文档。",
        ),
        learn_category: text(language, "Category", "分类"),
        learn_open: text(language, "Open", "打开"),
        team_workspace: text(language, "Local Team", "本地团队"),
        team_members_found: text(language, "contributors found", "个贡献者"),
        no_team_members_found: text(
            language,
            "No local Git contributors were found for this workspace.",
            "当前工作区没有发现本地 Git 贡献者。",
        ),
        team_git_identity: text(language, "Git Identity", "Git 身份"),
        team_repository: text(language, "Repository", "仓库"),
        team_local_only: text(
            language,
            "Local Git data only; no account or cloud service is connected.",
            "仅显示本地 Git 数据；未连接账号或云服务。",
        ),
        team_commits: text(language, "commits", "次提交"),
        cloud_overview: text(language, "Local Cloud Overview", "本地云服务概览"),
        cloud_local_only: text(
            language,
            "Offline local mode; no account, network, or remote service is connected.",
            "离线本地模式；未连接账号、网络或远程服务。",
        ),
        cloud_account: text(language, "Account", "账号"),
        cloud_output: text(language, "Build Output", "构建输出"),
        cloud_device_install: text(language, "Device Install", "设备安装"),
        cloud_packages: text(language, "Packages", "包"),
        cloud_services: text(language, "Service Slots", "服务槽位"),
        cloud_service_slots: text(language, "reserved slots", "个预留槽位"),
        plugin_catalog: text(language, "Plugin Catalog", "插件目录"),
        plugins_found: text(language, "plugins found", "个插件"),
        no_plugins_found: text(
            language,
            "No plugin manifests were found in the active source checkout.",
            "当前源码检出中没有发现插件清单。",
        ),
        plugin_packaging: text(language, "Packaging", "打包"),
        plugin_modules: text(language, "Modules", "模块"),
        plugin_maturity: text(language, "Maturity", "成熟度"),
    }
}

pub(super) fn page_title(page: HubPage, language: HubLanguage) -> SharedString {
    match page {
        HubPage::Projects => text(language, "Projects", "项目"),
        HubPage::Editor => text(language, "Editor", "编辑器"),
        HubPage::Assets => text(language, "Assets", "资产"),
        HubPage::Builds => text(language, "Builds", "构建"),
        HubPage::Plugins => text(language, "Plugins", "插件"),
        HubPage::Cloud => text(language, "Cloud", "云服务"),
        HubPage::Team => text(language, "Team", "团队"),
        HubPage::Learn => text(language, "Learn", "学习"),
        HubPage::Settings => text(language, "Settings", "设置"),
    }
}

pub(super) fn page_subtitle(page: HubPage, language: HubLanguage) -> SharedString {
    match page {
        HubPage::Projects => text(
            language,
            "Manage your projects and start building worlds.",
            "管理项目并开始构建世界。",
        ),
        HubPage::Editor => text(
            language,
            "Manage source installs and launch the editor.",
            "管理源码引擎并启动编辑器。",
        ),
        HubPage::Assets => text(
            language,
            "Browse discovered project and engine assets.",
            "浏览已发现的项目和引擎资产。",
        ),
        HubPage::Builds => text(
            language,
            "Build and package workflows for the active project.",
            "当前项目的构建与打包工作流。",
        ),
        HubPage::Plugins => text(
            language,
            "Plugin discovery and project extensions.",
            "插件发现和项目扩展。",
        ),
        HubPage::Cloud => text(
            language,
            "Local cloud readiness and reserved service slots.",
            "本地云服务准备状态和预留服务槽位。",
        ),
        HubPage::Team => text(
            language,
            "Local Git identity and recent contributors.",
            "本地 Git 身份与最近贡献者。",
        ),
        HubPage::Learn => text(
            language,
            "Guides, samples, and local documentation.",
            "指南、示例和本地文档。",
        ),
        HubPage::Settings => text(
            language,
            "Configure toolchains, source paths, and defaults.",
            "配置工具链、源码路径和默认值。",
        ),
    }
}

pub(super) fn project_filter_label(
    filter: ProjectFilterMode,
    language: HubLanguage,
) -> SharedString {
    match filter {
        ProjectFilterMode::All => text(language, "All Projects", "全部项目"),
        ProjectFilterMode::Existing => text(language, "Existing", "存在"),
        ProjectFilterMode::Missing => text(language, "Missing", "缺失"),
    }
}

pub(super) fn project_sort_label(sort: ProjectSortMode, language: HubLanguage) -> SharedString {
    match sort {
        ProjectSortMode::LastModified => text(language, "Last Modified", "最近修改"),
        ProjectSortMode::Name => text(language, "Name", "名称"),
    }
}

pub(super) fn text(language: HubLanguage, english: &str, chinese: &str) -> SharedString {
    match language {
        HubLanguage::English => SharedString::from(english),
        HubLanguage::Chinese => SharedString::from(chinese),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localizes_core_shell_text() {
        let texts = ui_text(HubLanguage::Chinese);

        assert_eq!(texts.game_engine, SharedString::from("游戏引擎"));
        assert_eq!(
            page_title(HubPage::Projects, HubLanguage::Chinese),
            SharedString::from("项目")
        );
        assert_eq!(
            project_sort_label(ProjectSortMode::LastModified, HubLanguage::Chinese),
            SharedString::from("最近修改")
        );
    }
}
