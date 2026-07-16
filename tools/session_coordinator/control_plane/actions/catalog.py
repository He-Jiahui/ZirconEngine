from __future__ import annotations

from ...models import CoordinatorError, WebControlRole
from .models import (
    ActionKind,
    ActionParameters,
    ActionRisk,
    ActionSpec,
    SessionParameters,
    GoalCloseoutParameters,
    LifecycleParameters,
    MaintenanceCleanupParameters,
    MilestoneCommitParameters,
    MilestoneReconciliationParameters,
    MilestoneParameters,
    ValidationCancelParameters,
    ValidationStartParameters,
    TopologyRefreshParameters,
)


def _spec(
    kind: ActionKind,
    title: str,
    risk: ActionRisk,
    role: WebControlRole,
    parameters: type[ActionParameters] = SessionParameters,
    *,
    enabled: bool = True,
    session_bound: bool = True,
    preview_only: bool = False,
    warnings: tuple[str, ...] = (),
) -> ActionSpec:
    return ActionSpec(
        kind,
        title,
        risk,
        role,
        parameters,
        enabled,
        session_bound,
        preview_only,
        warnings,
    )


_SPECS = (
    _spec(ActionKind.SESSION_HEARTBEAT, "刷新 Session 心跳", ActionRisk.YELLOW, WebControlRole.OPERATOR),
    _spec(ActionKind.SESSION_ACTIVATE, "激活 Session", ActionRisk.YELLOW, WebControlRole.OPERATOR),
    _spec(ActionKind.LEASE_CLAIM, "领取 Session 写入范围", ActionRisk.YELLOW, WebControlRole.OPERATOR),
    _spec(ActionKind.LEASE_RELEASE, "释放自有租约", ActionRisk.YELLOW, WebControlRole.OPERATOR),
    _spec(ActionKind.PATCH_PROCESS, "处理自有延迟补丁", ActionRisk.YELLOW, WebControlRole.OPERATOR),
    _spec(
        ActionKind.VALIDATION_START,
        "启动托管验证",
        ActionRisk.YELLOW,
        WebControlRole.OPERATOR,
        ValidationStartParameters,
        warnings=("验证只使用服务端模板，并在盘符根 cargo-targets 下创建临时副本。",),
    ),
    _spec(
        ActionKind.VALIDATION_CANCEL,
        "取消托管验证",
        ActionRisk.YELLOW,
        WebControlRole.OPERATOR,
        ValidationCancelParameters,
    ),
    _spec(ActionKind.FAILURE_REFRESH, "刷新 Failure 图", ActionRisk.YELLOW, WebControlRole.OPERATOR),
    _spec(
        ActionKind.TOPOLOGY_REFRESH,
        "刷新计划拓扑/导入独立评审",
        ActionRisk.YELLOW,
        WebControlRole.OPERATOR,
        TopologyRefreshParameters,
    ),
    _spec(
        ActionKind.DRAIN_PREVIEW,
        "预览服务排空",
        ActionRisk.YELLOW,
        WebControlRole.OPERATOR,
        preview_only=True,
        warnings=("M5 前仅提供影响预览，不执行进程生命周期变更。",),
    ),
    _spec(
        ActionKind.SERVICE_DRAIN,
        "排空服务写入",
        ActionRisk.YELLOW,
        WebControlRole.OPERATOR,
        LifecycleParameters,
        session_bound=False,
        warnings=("排空后拒绝新写入；读取与已经进入关键区的工作继续完成。",),
    ),
    _spec(
        ActionKind.SERVICE_RESUME,
        "恢复服务写入",
        ActionRisk.YELLOW,
        WebControlRole.OPERATOR,
        LifecycleParameters,
        session_bound=False,
    ),
    _spec(
        ActionKind.SERVICE_STOP,
        "停止服务",
        ActionRisk.RED,
        WebControlRole.MAINTAINER,
        LifecycleParameters,
        session_bound=False,
        warnings=("服务先排空全部关键区；超时只会失败，不会静默强杀进程。",),
    ),
    _spec(
        ActionKind.MILESTONE_COMMIT,
        "提交里程碑",
        ActionRisk.RED,
        WebControlRole.COMMITTER,
        MilestoneCommitParameters,
        warnings=("提交前将重新验证拓扑、尝试、Failure、评审、清单和 Git 基线；必须给出具体变更摘要。",),
    ),
    _spec(
        ActionKind.MILESTONE_RECONCILE,
        "对账同拓扑已验收里程碑",
        ActionRisk.RED,
        WebControlRole.MAINTAINER,
        MilestoneReconciliationParameters,
        session_bound=False,
        warnings=("仅导入同计划、同拓扑且提交仍位于 HEAD 历史中的已验收证据；不会暂存或重建历史文件。",),
    ),
    _spec(
        ActionKind.SESSION_COMPLETE,
        "完成 Goal 和 Session",
        ActionRisk.RED,
        WebControlRole.COMMITTER,
        GoalCloseoutParameters,
        warnings=("仅当所有里程碑完成且 Session 自有范围干净时允许关闭。",),
    ),
    _spec(
        ActionKind.SERVICE_RESTART,
        "重启服务",
        ActionRisk.RED,
        WebControlRole.MAINTAINER,
        LifecycleParameters,
        session_bound=False,
        warnings=("重启意图由后继 daemon 实例恢复并写回同一 action 终态。",),
    ),
    _spec(
        ActionKind.SERVICE_FORCE_STOP,
        "强制停止服务",
        ActionRisk.RED,
        WebControlRole.MAINTAINER,
        LifecycleParameters,
        session_bound=False,
        warnings=("仅用于高级恢复；托盘终止前必须再次核验完整进程身份。",),
    ),
    _spec(
        ActionKind.MAINTENANCE_CLEANUP,
        "清理共享 Git 暂存区",
        ActionRisk.RED,
        WebControlRole.MAINTAINER,
        MaintenanceCleanupParameters,
        session_bound=False,
        warnings=("仅将共享 index 重置到 HEAD；不修改工作树、不创建提交。",),
    ),
    _spec(
        ActionKind.CODEX_RECONCILE,
        "重新同步 Codex Sessions",
        ActionRisk.YELLOW,
        WebControlRole.MAINTAINER,
        ActionParameters,
        session_bound=False,
        warnings=("只唤醒单飞同步 worker；不接受路径、thread ID 或原始 Hook 数据。",),
    ),
)

ACTION_CATALOG: dict[str, ActionSpec] = {spec.kind.value: spec for spec in _SPECS}


def action_spec(kind: str) -> ActionSpec:
    try:
        return ACTION_CATALOG[kind]
    except KeyError as error:
        raise CoordinatorError("action_kind_unknown", "Action kind is not in the closed catalog") from error
