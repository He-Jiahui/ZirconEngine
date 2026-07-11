use crate::tray_state::MenuEnablement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuCommand {
    OpenConsole,
    Start,
    Drain,
    Resume,
    Stop,
    Restart,
    ForceStop,
    Diagnostics,
    Startup,
    ExitTray,
}

impl MenuCommand {
    pub fn id(self) -> &'static str {
        match self {
            Self::OpenConsole => "open-console",
            Self::Start => "start",
            Self::Drain => "drain",
            Self::Resume => "resume",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::ForceStop => "force-stop",
            Self::Diagnostics => "diagnostics",
            Self::Startup => "startup",
            Self::ExitTray => "exit-tray",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuEntry {
    pub command: MenuCommand,
    pub label: &'static str,
    pub enabled: bool,
}

pub fn menu_model(enablement: MenuEnablement) -> Vec<MenuEntry> {
    vec![
        MenuEntry {
            command: MenuCommand::OpenConsole,
            label: "打开工作流控制台",
            enabled: enablement.open_console,
        },
        MenuEntry {
            command: MenuCommand::Start,
            label: "启动服务",
            enabled: enablement.start,
        },
        MenuEntry {
            command: MenuCommand::Drain,
            label: "暂停新写入（排空）",
            enabled: enablement.drain,
        },
        MenuEntry {
            command: MenuCommand::Resume,
            label: "恢复新写入",
            enabled: enablement.resume,
        },
        MenuEntry {
            command: MenuCommand::Stop,
            label: "停止服务",
            enabled: enablement.stop,
        },
        MenuEntry {
            command: MenuCommand::Restart,
            label: "重启服务",
            enabled: enablement.restart,
        },
        MenuEntry {
            command: MenuCommand::ForceStop,
            label: "高级恢复：强制停止",
            enabled: enablement.force_stop,
        },
        MenuEntry {
            command: MenuCommand::Diagnostics,
            label: "复制诊断信息",
            enabled: enablement.diagnostics,
        },
        MenuEntry {
            command: MenuCommand::Startup,
            label: "启动项管理",
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::ExitTray,
            label: "退出托盘（服务保持运行）",
            enabled: enablement.exit_tray,
        },
    ]
}
