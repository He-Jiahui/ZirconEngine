use crate::lifecycle::LifecycleAction;
use crate::startup::StartupAction;
use crate::tray_state::MenuEnablement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuCommand {
    OpenConsole,
    Refresh,
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
            Self::Refresh => "refresh",
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
    pending_startup: Option<StartupAction>,
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
            command: MenuCommand::Refresh,
            label: "刷新托盘状态".into(),
            enabled: enablement.refresh,
        },
        MenuEntry {
            command: MenuCommand::Start,
            label: "启动服务".into(),
            enabled: enablement.start,
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
            enabled: pending.is_some() || pending_startup.is_some(),
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
            label: startup_label(StartupAction::Install, pending_startup),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::StartupUpdate,
            label: startup_label(StartupAction::Update, pending_startup),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::StartupRemove,
            label: startup_label(StartupAction::Remove, pending_startup),
            enabled: true,
        },
        MenuEntry {
            command: MenuCommand::ExitTray,
            label: "退出托盘（服务保持运行）".into(),
            enabled: enablement.exit_tray,
        },
    ]
}

fn startup_label(action: StartupAction, pending: Option<StartupAction>) -> String {
    let label = match action {
        StartupAction::Install => "安装启动项",
        StartupAction::Update => "更新启动项",
        StartupAction::Remove => "移除启动项",
        StartupAction::Query => "查询启动项",
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
    use crate::startup::StartupAction;

    #[test]
    fn healthy_menu_exposes_refresh_without_global_lifecycle_commands() {
        let entries = menu_model(
            MenuEnablement {
                open_console: true,
                refresh: true,
                diagnostics: true,
                exit_tray: true,
                ..MenuEnablement::default()
            },
            None,
            None,
            None,
            false,
            false,
        );

        assert!(entries
            .iter()
            .find(|entry| entry.command == MenuCommand::Refresh)
            .is_some_and(|entry| entry.enabled));
        for command in [
            MenuCommand::Drain,
            MenuCommand::Resume,
            MenuCommand::Stop,
            MenuCommand::Restart,
            MenuCommand::ForceStop,
        ] {
            assert!(!entries.iter().any(|entry| entry.command == command));
        }
    }

    #[test]
    fn pending_legacy_lifecycle_keeps_only_the_cancel_command() {
        let entries = menu_model(
            MenuEnablement {
                stop: true,
                diagnostics: true,
                exit_tray: true,
                ..MenuEnablement::default()
            },
            Some(LifecycleAction::Stop),
            None,
            None,
            false,
            false,
        );
        assert!(!entries
            .iter()
            .any(|entry| entry.command == MenuCommand::Stop));
        assert!(entries
            .iter()
            .find(|entry| entry.command == MenuCommand::CancelPending)
            .is_some_and(|entry| entry.enabled));
    }

    #[test]
    fn startup_management_exposes_query_install_update_and_remove() {
        let entries = menu_model(MenuEnablement::default(), None, None, None, false, true);
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
    fn pending_startup_mutation_requires_a_second_explicit_click() {
        let entries = menu_model(
            MenuEnablement::default(),
            None,
            None,
            Some(StartupAction::Remove),
            false,
            false,
        );
        let remove = entries
            .iter()
            .find(|entry| entry.command == MenuCommand::StartupRemove)
            .unwrap();
        assert!(remove.enabled);
        assert!(remove.label.starts_with("确认："));
        assert!(entries
            .iter()
            .find(|entry| entry.command == MenuCommand::CancelPending)
            .is_some_and(|entry| entry.enabled));
    }

    #[test]
    fn legacy_active_lifecycle_remains_cancellable_without_being_reoffered() {
        let entries = menu_model(
            MenuEnablement::default(),
            None,
            Some(LifecycleAction::Restart),
            None,
            true,
            false,
        );
        let cancel = entries
            .iter()
            .find(|entry| entry.command == MenuCommand::CancelActive)
            .unwrap();
        assert!(cancel.enabled);
        assert!(cancel.label.contains("service.restart"));
        assert!(!entries
            .iter()
            .any(|entry| entry.command == MenuCommand::Restart));
    }
}
