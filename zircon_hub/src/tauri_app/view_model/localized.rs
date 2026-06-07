use crate::settings::HubLanguage;
use crate::state::{HubActionKind, HubActionStatus, HubPage, TaskOperationKind};

#[derive(Clone, Copy)]
pub(crate) struct HubTextBundle {
    language: HubLanguage,
}

impl HubTextBundle {
    pub(crate) fn new(language: HubLanguage) -> Self {
        Self { language }
    }

    pub(crate) fn page_title(self, page: HubPage) -> &'static str {
        match self.language {
            HubLanguage::English => match page {
                HubPage::Projects => "Projects",
                HubPage::Editor => "Editor",
                HubPage::Assets => "Assets",
                HubPage::Builds => "Builds",
                HubPage::Plugins => "Plugins",
                HubPage::Cloud => "Local Delivery",
                HubPage::Team => "Team",
                HubPage::Learn => "Learn",
                HubPage::Settings => "Settings",
            },
            HubLanguage::Chinese => match page {
                HubPage::Projects => "项目",
                HubPage::Editor => "编辑器",
                HubPage::Assets => "资产",
                HubPage::Builds => "构建",
                HubPage::Plugins => "插件",
                HubPage::Cloud => "本地交付",
                HubPage::Team => "团队",
                HubPage::Learn => "学习",
                HubPage::Settings => "设置",
            },
        }
    }

    pub(crate) fn page_subtitle(self, page: HubPage) -> &'static str {
        match self.language {
            HubLanguage::English => match page {
                HubPage::Projects => "Manage your projects and start building worlds.",
                HubPage::Editor => "Manage source installs and launch the editor.",
                HubPage::Assets => "Browse selected project and Source Engine assets.",
                HubPage::Builds => "Build and package workflows for the selected project.",
                HubPage::Plugins => {
                    "Browse selected project plugin manifests and Source Engine plugins."
                }
                HubPage::Cloud => "Local packages, installs, and reserved service slots.",
                HubPage::Team => "Local Git identity and recent contributors.",
                HubPage::Learn => "Guides, samples, and local documentation.",
                HubPage::Settings => "Configure toolchains, source paths, and defaults.",
            },
            HubLanguage::Chinese => match page {
                HubPage::Projects => "管理本地项目并启动世界构建流程。",
                HubPage::Editor => "管理源码引擎安装并启动编辑器。",
                HubPage::Assets => "浏览已选项目和源码引擎中的资产。",
                HubPage::Builds => "为已选项目执行构建、打包和安装工作流。",
                HubPage::Plugins => "浏览已选项目和源码引擎中的插件清单。",
                HubPage::Cloud => "本地包输出、设备安装和预留服务入口。",
                HubPage::Team => "查看本地 Git 身份和近期贡献者。",
                HubPage::Learn => "本地指南、示例和文档资源。",
                HubPage::Settings => "配置工具链、源码路径、构建默认值和语言。",
            },
        }
    }

    pub(crate) fn status_label(self, label: &str) -> String {
        if self.language == HubLanguage::English {
            return label.to_string();
        }

        match label {
            "Ready" => "就绪",
            "Running" => "运行中",
            "Success" => "成功",
            "Warning" => "警告",
            "Error" => "错误",
            "Building" => "构建中",
            "Packaging" => "打包中",
            "Installing" => "安装中",
            "Opening Editor" => "正在打开编辑器",
            "Building failed" => "构建失败",
            "Packaging failed" => "打包失败",
            "Installing failed" => "安装失败",
            "Opening Editor failed" => "打开编辑器失败",
            "Build failed" => "构建失败",
            "Build editor/runtime failed" => "构建编辑器/运行时失败",
            "Build complete" => "构建完成",
            "Source Engine invalid" => "源码引擎无效",
            "Project created" => "项目已创建",
            "Project imported" => "项目已导入",
            "Import cancelled" => "已取消导入",
            "Project pinned" => "项目已置顶",
            "Project unpinned" => "项目已取消置顶",
            "Project removed from Hub" => "项目已从 Hub 移除",
            "Delete requested" => "已请求删除",
            "Delete cancelled" => "已取消删除",
            "Project deleted" => "项目已删除",
            "Create Project failed" => "创建项目失败",
            "Import Project failed" => "导入项目失败",
            "Remove Project failed" => "移除项目失败",
            "Delete Project failed" => "删除项目失败",
            "Package Project failed" => "打包项目失败",
            "Install to Device failed" => "安装到设备失败",
            "Package created" => "包已创建",
            "Installed to device" => "已安装到设备",
            "Open Editor failed" => "打开编辑器失败",
            "Editor launched" => "编辑器已启动",
            "Resource opened" => "资源已打开",
            "Open Resource failed" => "打开资源失败",
            "Output folder opened" => "输出文件夹已打开",
            "Open Output failed" => "打开输出失败",
            "Settings saved" => "设置已保存",
            "Save Settings failed" => "保存设置失败",
            "Folder selected" => "已选择文件夹",
            "Folder selection cancelled" => "已取消选择文件夹",
            "Browse folder failed" => "浏览文件夹失败",
            "Projects filtered" => "项目已筛选",
            "Projects sorted" => "项目已排序",
            "Project selected" => "项目已选择",
            "All projects" => "全部项目",
            "Engine selected" => "源码引擎已选择",
            "Loading Hub state" => "正在加载 Hub 状态",
            "Action failed" => "操作失败",
            "Live updates unavailable" => "实时更新不可用",
            _ => label,
        }
        .to_string()
    }

    pub(crate) fn status_detail(self, detail: &str) -> String {
        if self.language == HubLanguage::English {
            return detail.to_string();
        }

        if let Some(template) = detail.strip_prefix("Project template is coming soon: ") {
            return format!("项目模板尚未开放：{template}");
        }
        if let Some(path) = detail.strip_prefix("Project folder does not exist: ") {
            return format!("项目文件夹不存在：{path}");
        }
        if let Some(path) = detail.strip_prefix("zircon-project.toml was not found in ") {
            return format!("未在 {path} 找到 zircon-project.toml");
        }
        if let Some(path) = detail.strip_prefix("Project root is not valid: ") {
            return format!("项目根目录无效：{path}");
        }
        if let Some(project) = detail.strip_prefix("Project has no bound Source Engine: ") {
            return format!("项目未绑定源码引擎：{project}");
        }
        if let Some(binding) = detail.strip_prefix("Project bound Source Engine is unavailable: ") {
            return format!("项目绑定的源码引擎不可用：{binding}");
        }
        if let Some(engine_id) = detail.strip_prefix("Unknown Source Engine: ") {
            return format!("未知源码引擎：{engine_id}");
        }
        if let Some(path) = detail.strip_prefix("Created ") {
            return format!("已创建 {path}");
        }
        if let Some(path) = detail.strip_prefix("Imported ") {
            return format!("已导入 {path}");
        }
        if let Some(path) = detail.strip_prefix("Output folder does not exist: ") {
            return format!("输出文件夹不存在：{path}");
        }
        if let Some(path) = detail.strip_prefix("Resource file does not exist: ") {
            return format!("资源文件不存在：{path}");
        }
        if let Some(path) = detail.strip_prefix("Opened ") {
            return format!("已打开 {path}");
        }
        if let Some(path) = detail.strip_prefix("Editor executable is not available: ") {
            return format!("编辑器可执行文件不可用：{path}");
        }
        if let Some(process_id) = detail.strip_prefix("Started process ") {
            return format!("已启动进程 {process_id}");
        }
        if let Some(opening) = detail.strip_prefix("Opening ") {
            if let Some((target, process_id)) = opening
                .strip_suffix(')')
                .and_then(|body| body.rsplit_once(" (process "))
            {
                return format!("正在打开 {target}（进程 {process_id}）");
            }
        }
        if let Some(process_id) = detail.strip_prefix("Process ") {
            return format!("进程 {process_id}");
        }
        if let Some(filter) = detail
            .strip_prefix("Showing ")
            .and_then(localize_project_filter)
        {
            return format!("显示{filter}");
        }
        if let Some(sort) = detail.strip_prefix("Sorting by ") {
            return format!("按{}排序", localize_project_sort(sort));
        }
        if let Some(localized) = localize_delivery_log_excerpt(detail) {
            return localized;
        }
        if let Some(localized) = localize_file_count_suffix(detail) {
            return localized;
        }
        if let Some(path) = detail.strip_prefix("Device install already exists: ") {
            return format!("设备安装已存在：{path}");
        }
        if let Some(language) = detail.strip_prefix("Unknown Hub language: ") {
            return format!("未知 Hub 语言：{language}");
        }
        if let Some(profile) = detail.strip_prefix("Unknown build profile: ") {
            return format!("未知构建配置：{profile}");
        }
        if let Some(field) = detail.strip_prefix("Unknown settings folder field: ") {
            return format!("未知设置文件夹字段：{field}");
        }
        if let Some((action, target)) = detail
            .strip_prefix("Unknown recent project target for ")
            .and_then(|body| body.split_once(": "))
        {
            return format!("未知最近项目目标（{action}）：{target}");
        }

        match detail {
            "Hub is ready" => "Hub 已就绪",
            "Refreshing projects, source engines, and build workflows" => {
                "正在刷新项目、源码引擎和构建工作流"
            }
            "Source engine is ready" => "源码引擎已就绪",
            "Source checkout directory is missing" => "源码检出目录缺失",
            "Source checkout is missing Cargo.toml" => "源码检出缺少 Cargo.toml",
            "Source checkout is missing tools/zircon_build.py" => {
                "源码检出缺少 tools/zircon_build.py"
            }
            "No recovery action is required" => "无需恢复操作",
            "Locate an existing ZirconEngine checkout or update Settings > Source Checkout" => {
                "定位已有 ZirconEngine 检出，或更新设置 > 源码检出"
            }
            "Select the ZirconEngine repository root that contains the workspace Cargo.toml" => {
                "选择包含工作区 Cargo.toml 的 ZirconEngine 仓库根目录"
            }
            "Select a complete ZirconEngine checkout with tools/zircon_build.py before building" => {
                "构建前选择包含 tools/zircon_build.py 的完整 ZirconEngine 检出"
            }
            "Showing all recent projects" => "显示全部最近项目",
            "Running tools/zircon_build.py" => "正在运行 tools/zircon_build.py",
            "Staged editor/runtime payload" => "已暂存编辑器/运行时载荷",
            "Copying project into package output" => "正在复制项目到包输出目录",
            "Preparing package and copying to local device install directory" => {
                "正在准备包并复制到本地设备安装目录"
            }
            "Launching staged editor process" => "正在启动暂存编辑器进程",
            "Select a valid project with a bound Source Engine before building" => {
                "构建前先选择一个已绑定源码引擎的有效项目"
            }
            "Check Python, Cargo, and Source Checkout settings before retrying" => {
                "重试前检查 Python、Cargo 和源码检出设置"
            }
            "Open Build History and fix the first reported error before retrying" => {
                "打开构建历史并修复第一条错误后再重试"
            }
            "No recent project is available to build" => "没有可用于构建的最近项目",
            "Selected project is no longer available to build" => "已选项目不再可用于构建",
            "No recent project is available to package" => "没有可用于打包的最近项目",
            "Selected project is no longer available to package" => "已选项目不再可用于打包",
            "No recent project is available to install" => "没有可用于安装的最近项目",
            "Selected project is no longer available to install" => "已选项目不再可用于安装",
            "Select an available project before packaging" => "打包前先选择一个可用项目",
            "Select a valid project and package it before installing to a device" => {
                "安装到设备前先选择一个有效项目并完成打包"
            }
            "Check that the selected project directory contains a Zircon project manifest" => {
                "检查已选项目目录是否包含 Zircon 项目清单"
            }
            "Project root is not available for packaging" => "项目根目录不可用于打包",
            "Package output root is required" => "需要包输出根目录",
            "Package output root must be outside the project directory" => {
                "包输出根目录必须位于项目目录外"
            }
            "Check that the project root exists and the package output is outside the project" => {
                "检查项目根目录是否存在，并确保包输出目录位于项目外"
            }
            "Package directory is not available" => "包目录不可用",
            "Device install directory is required" => "需要设备安装目录",
            "Device install directory must be outside the package directory" => {
                "设备安装目录必须位于包目录外"
            }
            "Check the package output and configured local device install directory before retrying" => {
                "重试前检查包输出和已配置的本地设备安装目录"
            }
            "Select an available project or launch Editor without a project" => {
                "选择一个可用项目，或不带项目启动编辑器"
            }
            "Choose a valid Zircon project before opening it in Editor" => {
                "在编辑器中打开前先选择一个有效的 Zircon 项目"
            }
            "Build the editor/runtime payload or fix Source Engine settings before opening the project" => {
                "打开项目前先构建编辑器/运行时载荷，或修复源码引擎设置"
            }
            "Build the editor/runtime payload or fix Source Engine settings before launching" => {
                "启动前先构建编辑器/运行时载荷，或修复源码引擎设置"
            }
            "Verify the staged zircon_editor executable exists and the project path is accessible" => {
                "确认暂存的 zircon_editor 可执行文件存在，且项目路径可访问"
            }
            "Verify the staged zircon_editor executable exists" => {
                "确认暂存的 zircon_editor 可执行文件存在"
            }
            "Fill in a project name, location, enabled template, and Source Engine before creating" => {
                "创建前填写项目名称、位置、已启用模板和源码引擎"
            }
            "Choose the Renderable Empty template for v1 local project creation" => {
                "v1 本地项目创建请选择“可渲染空项目”模板"
            }
            "Register or select a Source Engine before creating the project" => {
                "创建项目前先注册或选择源码引擎"
            }
            "Choose an empty target folder and retry project creation" => {
                "选择一个空目标文件夹后重试项目创建"
            }
            "Choose a folder containing zircon-project.toml" => {
                "选择包含 zircon-project.toml 的文件夹"
            }
            "No project folder was selected" => "未选择项目文件夹",
            "Run Import Project again and choose a Zircon project folder" => {
                "重新运行导入项目并选择 Zircon 项目文件夹"
            }
            "Register or select a Source Engine before importing the project" => {
                "导入项目前先注册或选择源码引擎"
            }
            "Removed project from Hub recent list" => "已从 Hub 最近项目列表移除",
            "Project files were left on disk" => "项目文件已保留在磁盘上",
            "Confirm delete to move the project to the Windows Recycle Bin" => {
                "确认删除会将项目移动到 Windows 回收站"
            }
            "Cancel delete to leave the project unchanged" => "取消删除以保持项目不变",
            "Project was left unchanged" => "项目保持不变",
            "Moved project to Windows Recycle Bin" => "已将项目移动到 Windows 回收站",
            "The project remains in Hub; fix the filesystem issue or cancel delete" => {
                "项目仍保留在 Hub 中；修复文件系统问题或取消删除"
            }
            "Choose a resource from the current Learn catalog" => {
                "从当前学习目录中选择资源"
            }
            "Open Resource target is required" => "需要打开资源目标",
            "Resource is not present in the current Learn catalog" => {
                "资源不在当前学习目录中"
            }
            "Refresh the Learn catalog or choose an existing local document" => {
                "刷新学习目录或选择已有本地文档"
            }
            "Refresh the Learn catalog and choose an existing local document" => {
                "刷新学习目录并选择已有本地文档"
            }
            "Open Output target is required" => "需要打开输出目标",
            "Open the containing folder from the file system and verify shell integration" => {
                "从文件系统打开所在文件夹，并检查系统外壳集成"
            }
            "Choose a recorded package, install, or build output before opening the folder" => {
                "打开文件夹前先选择已记录的包、安装或构建输出"
            }
            "Run the build, package, or install workflow again and then open its output folder" => {
                "重新运行构建、打包或安装工作流后再打开输出文件夹"
            }
            "Open the folder manually from the file system and verify shell integration" => {
                "从文件系统手动打开文件夹，并检查系统外壳集成"
            }
            "Review the action target and retry from Hub" => "检查操作目标后从 Hub 重试",
            "State will still refresh after each command completes" => "命令完成后仍会刷新状态",
            "Check the action target and try again" => "检查操作目标后重试",
            "No folder was selected" => "未选择文件夹",
            "Choose a folder or keep the current setting" => "选择文件夹或保留当前设置",
            "Choose an existing local folder or type the path manually" => {
                "选择已有本地文件夹或手动输入路径"
            }
            "Check Settings values and save again" => "检查设置值后重新保存",
            "Settings folder field is required" => "需要设置文件夹字段",
            "Python executable is required" => "需要 Python 可执行文件",
            "Cargo executable is required" => "需要 Cargo 可执行文件",
            "Rustup executable is required" => "需要 Rustup 可执行文件",
            "Default project directory is required" => "需要默认项目目录",
            "Default source directory is required" => "需要默认源码目录",
            "Default build output directory is required" => "需要默认构建输出目录",
            "Default device install directory is required" => "需要默认设备安装目录",
            _ => detail,
        }
        .to_string()
    }

    pub(crate) fn operation_scope(self, operation: TaskOperationKind) -> &'static str {
        match self.language {
            HubLanguage::English => match operation {
                TaskOperationKind::Project => "Project",
                TaskOperationKind::SourceEngine => "Source Engine",
                TaskOperationKind::Build => "Build",
                TaskOperationKind::Process => "Process",
                TaskOperationKind::Settings => "Settings",
                TaskOperationKind::Hub => "Hub",
            },
            HubLanguage::Chinese => match operation {
                TaskOperationKind::Project => "项目",
                TaskOperationKind::SourceEngine => "源码引擎",
                TaskOperationKind::Build => "构建",
                TaskOperationKind::Process => "进程",
                TaskOperationKind::Settings => "设置",
                TaskOperationKind::Hub => "Hub",
            },
        }
    }

    pub(crate) fn operation_target(self, target: &str) -> String {
        if self.language == HubLanguage::English {
            return target.to_string();
        }

        match target {
            "Projects" => "项目",
            "Project" => "项目",
            "Import Project" => "导入项目",
            "Source Engine" => "源码引擎",
            "Device Install" => "设备安装",
            "Editor" => "编辑器",
            "Output Folder" => "输出文件夹",
            "Hub settings" => "Hub 设置",
            "Settings folder" => "设置文件夹",
            "Hub action" => "Hub 操作",
            "Visual verification" => "视觉验证",
            _ => target,
        }
        .to_string()
    }

    pub(crate) fn action_label(self, action: HubActionKind) -> &'static str {
        match self.language {
            HubLanguage::English => action.label(),
            HubLanguage::Chinese => match action {
                HubActionKind::CreateProject => "创建项目",
                HubActionKind::ImportProject => "导入项目",
                HubActionKind::RemoveProject => "移除项目",
                HubActionKind::DeleteProject => "删除项目",
                HubActionKind::BuildEditorRuntime => "构建编辑器/运行时",
                HubActionKind::OpenEditor => "打开编辑器",
                HubActionKind::PackageProject => "打包项目",
                HubActionKind::InstallProject => "安装到设备",
                HubActionKind::OpenResource => "打开资源",
                HubActionKind::OpenOutput => "打开输出",
            },
        }
    }

    pub(crate) fn action_status_label(self, status: HubActionStatus) -> &'static str {
        match self.language {
            HubLanguage::English => status.label(),
            HubLanguage::Chinese => match status {
                HubActionStatus::Success => "成功",
                HubActionStatus::Failed => "失败",
                HubActionStatus::Cancelled => "已取消",
            },
        }
    }

    pub(crate) fn pair(self, english: &'static str, chinese: &'static str) -> &'static str {
        match self.language {
            HubLanguage::English => english,
            HubLanguage::Chinese => chinese,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_bundle_localizes_page_and_status_copy() {
        let bundle = HubTextBundle::new(HubLanguage::Chinese);

        assert_eq!(bundle.page_title(HubPage::Settings), "设置");
        assert_eq!(bundle.status_label("Ready"), "就绪");
        assert_eq!(bundle.status_detail("Hub is ready"), "Hub 已就绪");
        assert_eq!(bundle.operation_scope(TaskOperationKind::Settings), "设置");
        assert_eq!(
            bundle.action_label(HubActionKind::BuildEditorRuntime),
            "构建编辑器/运行时"
        );
        assert_eq!(bundle.action_status_label(HubActionStatus::Failed), "失败");
        assert_eq!(
            bundle.status_detail("Select an available project before packaging"),
            "打包前先选择一个可用项目"
        );
        assert_eq!(
            bundle.status_detail("Project template is coming soon: 3d-scene"),
            "项目模板尚未开放：3d-scene"
        );
        assert_eq!(
            bundle.status_detail("Project folder does not exist: C:\\Projects\\Missing"),
            "项目文件夹不存在：C:\\Projects\\Missing"
        );
        assert_eq!(
            bundle.status_detail("zircon-project.toml was not found in C:\\Projects\\Missing"),
            "未在 C:\\Projects\\Missing 找到 zircon-project.toml"
        );
        assert_eq!(
            bundle.status_detail("Project root is not valid: C:\\Projects\\Broken"),
            "项目根目录无效：C:\\Projects\\Broken"
        );
        assert_eq!(
            bundle.status_detail("Project has no bound Source Engine: Game"),
            "项目未绑定源码引擎：Game"
        );
        assert_eq!(
            bundle.status_detail("Project bound Source Engine is unavailable: Game -> source-old"),
            "项目绑定的源码引擎不可用：Game -> source-old"
        );
        assert_eq!(
            bundle.status_detail("Unknown Source Engine: source-missing"),
            "未知源码引擎：source-missing"
        );
        assert_eq!(
            bundle.status_detail("Created C:\\Projects\\Game"),
            "已创建 C:\\Projects\\Game"
        );
        assert_eq!(
            bundle.status_detail("Imported C:\\Projects\\Imported"),
            "已导入 C:\\Projects\\Imported"
        );
        assert_eq!(
            bundle.status_detail("Output folder does not exist: C:\\Packages\\Missing"),
            "输出文件夹不存在：C:\\Packages\\Missing"
        );
        assert_eq!(
            bundle.status_detail("Resource file does not exist: C:\\Docs\\guide.md"),
            "资源文件不存在：C:\\Docs\\guide.md"
        );
        assert_eq!(
            bundle.status_detail("Open Resource target is required"),
            "需要打开资源目标"
        );
        assert_eq!(
            bundle.status_detail("Opened C:\\Packages\\Game"),
            "已打开 C:\\Packages\\Game"
        );
        assert_eq!(
            bundle.status_detail("Open Output target is required"),
            "需要打开输出目标"
        );
        assert_eq!(
            bundle
                .status_detail("Editor executable is not available: C:\\Zircon\\zircon_editor.exe"),
            "编辑器可执行文件不可用：C:\\Zircon\\zircon_editor.exe"
        );
        assert_eq!(bundle.status_detail("Started process 42"), "已启动进程 42");
        assert_eq!(
            bundle.status_detail("Opening Game (process 42)"),
            "正在打开 Game（进程 42）"
        );
        assert_eq!(bundle.status_detail("Process 42"), "进程 42");
        assert_eq!(
            bundle.status_detail("Source checkout directory is missing"),
            "源码检出目录缺失"
        );
        assert_eq!(
            bundle.status_detail("Source checkout is missing Cargo.toml"),
            "源码检出缺少 Cargo.toml"
        );
        assert_eq!(
            bundle.status_detail("Source checkout is missing tools/zircon_build.py"),
            "源码检出缺少 tools/zircon_build.py"
        );
        assert_eq!(
            bundle.status_detail("Staged editor/runtime payload"),
            "已暂存编辑器/运行时载荷"
        );
        assert_eq!(
            bundle.status_detail("Game -> C:\\Packages\\Game (2 files)"),
            "Game -> C:\\Packages\\Game（2 个文件）"
        );
        assert_eq!(
            bundle.status_detail("Project root is not available for packaging"),
            "项目根目录不可用于打包"
        );
        assert_eq!(
            bundle.status_detail("Package output root is required"),
            "需要包输出根目录"
        );
        assert_eq!(
            bundle.status_detail("Package output root must be outside the project directory"),
            "包输出根目录必须位于项目目录外"
        );
        assert_eq!(
            bundle.status_detail("Package directory is not available"),
            "包目录不可用"
        );
        assert_eq!(
            bundle.status_detail("Device install directory is required"),
            "需要设备安装目录"
        );
        assert_eq!(
            bundle.status_detail("Device install directory must be outside the package directory"),
            "设备安装目录必须位于包目录外"
        );
        assert_eq!(
            bundle.status_detail("Device install already exists: C:\\Devices\\Game"),
            "设备安装已存在：C:\\Devices\\Game"
        );
        assert_eq!(
            bundle.status_detail("Unknown Hub language: Klingon"),
            "未知 Hub 语言：Klingon"
        );
        assert_eq!(
            bundle.status_detail("Unknown build profile: shipping"),
            "未知构建配置：shipping"
        );
        assert_eq!(
            bundle.status_detail("Check Settings values and save again"),
            "检查设置值后重新保存"
        );
        assert_eq!(bundle.status_label("Import cancelled"), "已取消导入");
        assert_eq!(bundle.status_label("Save Settings failed"), "保存设置失败");
        assert_eq!(bundle.status_label("Projects filtered"), "项目已筛选");
        assert_eq!(bundle.operation_target("Output Folder"), "输出文件夹");
        assert_eq!(bundle.operation_target("Hub settings"), "Hub 设置");
        assert_eq!(bundle.status_detail("Showing Missing"), "显示缺失项目");
        assert_eq!(bundle.status_label("Projects sorted"), "项目已排序");
        assert_eq!(bundle.status_detail("Sorting by Name"), "按名称排序");
        assert_eq!(
            bundle.status_detail("Showing all recent projects"),
            "显示全部最近项目"
        );
        assert_eq!(
            bundle.status_detail("Packaged Game to C:\\Packages\\Game (2 files)"),
            "已打包 Game 到 C:\\Packages\\Game（2 个文件）"
        );
        assert_eq!(
            bundle.status_detail("Installed Game to C:\\Devices\\Game (3 files)"),
            "已安装 Game 到 C:\\Devices\\Game（3 个文件）"
        );
    }
}

fn localize_file_count_suffix(detail: &str) -> Option<String> {
    let body = detail.strip_suffix(" files)")?;
    let (prefix, count) = body.rsplit_once(" (")?;
    if count.chars().all(|character| character.is_ascii_digit()) {
        Some(format!("{prefix}（{count} 个文件）"))
    } else {
        None
    }
}

fn localize_delivery_log_excerpt(detail: &str) -> Option<String> {
    let (verb, rest) = detail.split_once(' ')?;
    let action = match verb {
        "Packaged" => "已打包",
        "Installed" => "已安装",
        _ => return None,
    };
    let body = rest.strip_suffix(" files)")?;
    let (target_and_path, count) = body.rsplit_once(" (")?;
    if !count.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let (target, path) = target_and_path.split_once(" to ")?;
    Some(format!("{action} {target} 到 {path}（{count} 个文件）"))
}

fn localize_project_filter(filter: &str) -> Option<&'static str> {
    match filter {
        "All Projects" => Some("全部项目"),
        "Existing" => Some("存在项目"),
        "Missing" => Some("缺失项目"),
        _ => None,
    }
}

fn localize_project_sort(sort: &str) -> &'static str {
    match sort {
        "Last Modified" => "最近修改",
        "Name" => "名称",
        _ => "当前字段",
    }
}
