use serde::{Deserialize, Serialize};

/// 主窗口关闭后的进程退出策略，由 App 的窗口事件处理器执行。
/// 当前入口只管理一个主原生窗口，`OnAllClosed` 在该入口等价于主窗口关闭。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowExitCondition {
    OnPrimaryClosed,
    #[default]
    OnAllClosed,
    DontExit,
}

/// 将关闭请求是否销毁窗口，与销毁后是否退出事件循环分开配置。
/// 关闭请求先发给运行时；只有事件分发成功后，App 才按此策略释放表面并退出。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowLifecyclePolicy {
    pub exit_condition: WindowExitCondition,
    pub close_when_requested: bool,
}

impl WindowLifecyclePolicy {
    pub fn with_exit_condition(mut self, exit_condition: WindowExitCondition) -> Self {
        self.exit_condition = exit_condition;
        self
    }

    pub fn with_close_when_requested(mut self, close_when_requested: bool) -> Self {
        self.close_when_requested = close_when_requested;
        self
    }

    pub fn should_close_on_request(self) -> bool {
        self.close_when_requested
    }

    /// 仅供当前单主窗口入口在关闭或销毁事件后判断退出；多窗口宿主应按实际剩余窗口重新判定。
    pub fn should_exit_after_primary_close(self) -> bool {
        self.close_when_requested
            && matches!(
                self.exit_condition,
                WindowExitCondition::OnPrimaryClosed | WindowExitCondition::OnAllClosed
            )
    }

    pub fn diagnostic_lines(self) -> [String; 2] {
        [
            exit_condition_diagnostic_line(self.exit_condition),
            close_when_requested_diagnostic_line(self.close_when_requested),
        ]
    }
}

fn exit_condition_diagnostic_line(exit_condition: WindowExitCondition) -> String {
    const PREFIX: &str = "window.exit_condition=";
    let value = match exit_condition {
        WindowExitCondition::OnPrimaryClosed => "OnPrimaryClosed",
        WindowExitCondition::OnAllClosed => "OnAllClosed",
        WindowExitCondition::DontExit => "DontExit",
    };
    let mut line = String::with_capacity(PREFIX.len() + value.len());
    line.push_str(PREFIX);
    line.push_str(value);
    line
}

fn close_when_requested_diagnostic_line(close_when_requested: bool) -> String {
    const PREFIX: &str = "window.close_when_requested=";
    let value = if close_when_requested {
        "true"
    } else {
        "false"
    };
    let mut line = String::with_capacity(PREFIX.len() + value.len());
    line.push_str(PREFIX);
    line.push_str(value);
    line
}

impl Default for WindowLifecyclePolicy {
    fn default() -> Self {
        Self {
            exit_condition: WindowExitCondition::OnAllClosed,
            close_when_requested: true,
        }
    }
}

#[cfg(test)]
#[path = "tests/lifecycle_policy_optimization_batch_fh_tests.rs"]
mod optimization_batch_fh_tests;
