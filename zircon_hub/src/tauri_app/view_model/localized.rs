use crate::settings::HubLanguage;
use crate::state::{HubActionKind, HubActionStatus, HubMessage, HubPage, TaskOperationKind};

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
            "Creating Project" => "正在创建项目",
            "Opening Editor" => "正在打开编辑器",
            "Task cancelled" => "任务已取消",
            "Building failed" => "构建失败",
            "Packaging failed" => "打包失败",
            "Installing failed" => "安装失败",
            "Creating Project failed" => "创建项目失败",
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
            "Settings draft discarded" => "已放弃设置修改",
            "Default settings restored" => "已恢复默认设置",
            "Save Hub state failed" => "保存 Hub 状态失败",
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

    pub(crate) fn render_message(self, message: &HubMessage) -> String {
        message.render(self.language)
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
            "Settings source checkout" => "设置中的源码检出目录",
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
#[path = "tests/localized.rs"]
mod tests;
