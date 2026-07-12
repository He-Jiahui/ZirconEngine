use crate::lifecycle::LifecycleAction;
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
    CancelPending,
    CancelActive,
    StartupQuery,
    StartupInstall,
    StartupUpdate,
    StartupRemove,
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
            Self::CancelPending => "cancel-pending",
            Self::CancelActive => "cancel-active",
            Self::StartupQuery => "startup-query",
            Self::StartupInstall => "startup-install",
            Self::StartupUpdate => "startup-update",
            Self::StartupRemove => "startup-remove",
            Self::ExitTray => "exit-tray",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuEntry {
    pub command: MenuCommand,
    pub label: String,
    pub enabled: bool,
}

pub fn menu_model(
    enablement: MenuEnablement,
    pending: Option<LifecycleAction>,
    active: Option<LifecycleAction>,
    can_cancel_active: bool,
    has_error: bool,
) -> Vec<MenuEntry> {
    vec![
        MenuEntry {
            command: MenuCommand::OpenConsole,
            label: "打开工作流控制台".into(),
            enabled: enablement.open_console,
        },
        MenuEntry {
            command: MenuCommand::Start,
            label: "启动服务".into(),
            enabled: enablement.start,
        },
        MenuEntry {
            command: MenuCommand::Drain,
            label: lifecycle_label(LifecycleAction::Drain, pending),
            enabled: enablement.drain,
        },
        MenuEntry {
            command: MenuCommand::Resume,
            label: lifecycle_label(LifecycleAction::Resume, pending),
            enabled: enablement.resume,
        },
        MenuEntry {
            command: MenuCommand::Stop,
            label: lifecycle_label(LifecycleAction::Stop, pending),
            enabled: enablement.stop,
        },
        MenuEntry {
            command: MenuCommand::Restart,
            label: lifecycle_label(LifecycleAction::Restart, pending),
            enabled: enablement.restart,
        },
        MenuEntry {
            command: MenuCommand::ForceStop,
            label: lifecycle_label(LifecycleAction::ForceStop, pending),
            enabled: enablement.force_stop,
        },
        MenuEntry {
            command: MenuCommand::Diagnostics,
            label: if has_error {
                "诊断信息（含最近错误）".into()
            } else {
                "诊断信息".into()
            },
            enabled: enablement.diagnostics,
        },
        MenuEntry {
            command: MenuCommand::CancelPending,
            label: "取消待确认操作".into(),
            enabled: pending.is_some(),
        },
        MenuEntry {
            command: MenuCommand::CancelActive,
            label: active.map_or_else(
                || "取消执行中生命周期".into(),
                |action| format!("取消执行中操作：{}", action.kind()),
            ),
            enabled: active.is_some() && can_cancel_active,
        },
        MenuEntry {
            command: MenuCommand::StartupQuery,
            label: "查询启动项".into(),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::StartupInstall,
            label: "安装启动项".into(),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::StartupUpdate,
            label: "更新启动项".into(),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::StartupRemove,
            label: "移除启动项".into(),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::ExitTray,
            label: "退出托盘（服务保持运行）".into(),
            enabled: enablement.exit_tray,
        },
    ]
}

fn lifecycle_label(action: LifecycleAction, pending: Option<LifecycleAction>) -> String {
    let label = match action {
        LifecycleAction::Drain => "暂停新写入（排空）",
        LifecycleAction::Resume => "恢复新写入",
        LifecycleAction::Stop => "停止服务",
        LifecycleAction::Restart => "重启服务",
        LifecycleAction::ForceStop => "高级恢复：强制停止",
    };
    if pending == Some(action) {
        format!("确认：{label}（再次点击）")
    } else {
        label.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_lifecycle_requires_a_second_explicit_click() {
        let entries = menu_model(
            MenuEnablement {
                stop: true,
                diagnostics: true,
                exit_tray: true,
                ..MenuEnablement::default()
            },
            Some(LifecycleAction::Stop),
            None,
            false,
            false,
        );
        let stop = entries
            .iter()
            .find(|entry| entry.command == MenuCommand::Stop)
            .unwrap();
        assert!(stop.enabled);
        assert!(stop.label.starts_with("确认："));
        assert!(entries
            .iter()
            .find(|entry| entry.command == MenuCommand::CancelPending)
            .is_some_and(|entry| entry.enabled));
    }

    #[test]
    fn startup_management_exposes_query_install_update_and_remove() {
        let entries = menu_model(MenuEnablement::default(), None, None, false, true);
        for command in [
            MenuCommand::StartupQuery,
            MenuCommand::StartupInstall,
            MenuCommand::StartupUpdate,
            MenuCommand::StartupRemove,
        ] {
            assert!(entries
                .iter()
                .find(|entry| entry.command == command)
                .is_some_and(|entry| entry.enabled));
        }
        assert!(entries
            .iter()
            .find(|entry| entry.command == MenuCommand::Diagnostics)
            .is_some_and(|entry| entry.label.contains("最近错误")));
    }

    #[test]
    fn confirmed_draining_lifecycle_exposes_a_separate_cancel_command() {
        let entries = menu_model(
            MenuEnablement::default(),
            None,
            Some(LifecycleAction::Restart),
            true,
            false,
        );
        let cancel = entries
            .iter()
            .find(|entry| entry.command == MenuCommand::CancelActive)
            .unwrap();
        assert!(cancel.enabled);
        assert!(cancel.label.contains("service.restart"));
    }
}
