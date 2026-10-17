//! 启动邮箱将终态结果绑定到一次 Hub 启动会话；文件路径及生命周期由宿主负责。
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    HubEditorLaunchOutcomeV1, HubEditorReadyReceiptV1, HubEditorStartupFailureCodeV1,
    HubProtocolVersionV1, HubSessionToken,
};

/// File-mailbox payload shared by Hub and Editor for a single launch session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Hub 读取后还须把 launch_session 与当前受监督启动令牌逐项比对。
pub struct HubEditorMailboxV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub launch_session: HubSessionToken,
    pub outcome: HubEditorLaunchOutcomeV1,
}

impl HubEditorMailboxV1 {
    pub fn ready(launch_session: HubSessionToken, receipt: HubEditorReadyReceiptV1) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            launch_session,
            outcome: HubEditorLaunchOutcomeV1::Ready { receipt },
        }
    }

    pub fn failed(launch_session: HubSessionToken, code: HubEditorStartupFailureCodeV1) -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            launch_session,
            outcome: HubEditorLaunchOutcomeV1::Failed { code },
        }
    }

    pub fn ready_receipt(&self) -> Option<&HubEditorReadyReceiptV1> {
        match &self.outcome {
            HubEditorLaunchOutcomeV1::Ready { receipt } => Some(receipt),
            HubEditorLaunchOutcomeV1::Failed { .. } => None,
        }
    }

    /// Hub 解码邮箱后用受监控子进程的 token 核对，避免旧会话结果冒充当前启动。
    /// 仅当载荷中的会话令牌与本次启动令牌一致时返回成功。
    pub fn validate_launch_session(
        &self,
        expected: HubSessionToken,
    ) -> Result<(), HubEditorMailboxSessionError> {
        if self.launch_session == expected {
            Ok(())
        } else {
            Err(HubEditorMailboxSessionError)
        }
    }
}

/// Opaque mismatch result for a mailbox not bound to the launch being supervised.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HubEditorMailboxSessionError;

impl fmt::Display for HubEditorMailboxSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Hub Editor mailbox launch session does not match the supervised child")
    }
}

impl std::error::Error for HubEditorMailboxSessionError {}
