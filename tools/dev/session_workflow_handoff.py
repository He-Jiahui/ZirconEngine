"""Plan one manually reviewed handoff for sessions blocked on retired Jenkins.

The host supplies current, normalized session rows and the registered worktrees
of this repository only. It previews the plans, confirms the recipients, and
supplies a supported GUI or public-API prompt adapter if delivery is authorized.
This module does not access session databases, credentials, or services and
never polls or retries. A delivery receipt is not evidence of adoption.
"""

from __future__ import annotations

import json
import ntpath
import posixpath
import re
from collections.abc import Callable, Iterable
from typing import Any
from uuid import NAMESPACE_URL, uuid5


_FENCE = re.compile(r"^[ \t]*(`{3,}|~{3,})")
_INLINE_QUOTE = re.compile(r'`[^`]*`|"[^"]*"|“[^”]*”|‘[^’]*’|『[^』]*』|「[^」]*」')
_HISTORY = re.compile(
    r"历史(?:上|记录|交接|引用|资料|说明|措辞|示例)|\b(?:prior|previous|former)\s+handoff\b|"
    r"\bhistorical(?:\s+(?:handoff|wording|example|quote|record|context))?\s*[:：)]|\bpreviously\b",
    re.I,
)
_LIVE_RULE = re.compile(
    r"(?:^|[.!?;]\s+)(?:"
    r"Jenkins\s+(?:still\s+owns|(?:is|remains)\s+the\s+(?:sole|exclusive)\s+owner)|"
    r"Use\s+the\s+independent\s+Jenkins\s+coordinator|"
    r"(?:Git|branch\s+commits?)\s+must\s+(?:go\s+through|use)\s+(?:the\s+)?(?:managed\s+)?Jenkins)\b",
    re.I,
)

_ENTRY = r"(?:Jenkins|受管(?:的)?\s*(?:Git\s*)?(?:入口|提交|交付|流程)|managed\s+(?:Git|entry[ -]?point))"
_RECOVERY = r"(?:恢复|重启|上线|可用|(?:come(?:s)?|is|are|be|go(?:es)?)\s+back|(?:back\s+)?online|available|restor(?:ed|ation))"
_WAIT_FOR_RECOVERY = re.compile(
    r"(?:等(?:待|候)?|\bwait(?:ing)?\s+(?:for|until)\b|\b(?:blocked|stuck)\s+(?:on|until)\b|"
    r"\b(?:cannot|can't|unable)\b[^。！？.!?]{0,50}\buntil\b)"
    rf"[^。！？.!?]{{0,34}}{_ENTRY}[^。！？.!?]{{0,44}}{_RECOVERY}",
    re.I,
)
_UNAVAILABLE = re.compile(r"连接(?:被)?拒绝|拒绝连接|连接失败|不可用|ECONNREFUSED|refus(?:ed|es|al)|unavailable", re.I)
_DELIVERY = r"(?:提交|交付|推送|开(?:立|启)?\s*(?:独立\s*)?(?:Draft\s*)?PR|创(?:建|立)\s*(?:独立\s*)?(?:Draft\s*)?PR|\bcommit\b|\bpush\b|(?:open|create)\b.{0,28}\bPR\b|(?:Draft\s*)?PR\s+creation)"
_BLOCKED_DELIVERY = re.compile(
    rf"(?:无法|不能|没法|未能|阻断|阻塞|卡在|can't|cannot|unable\s+to|blocked)"
    rf"[^。！？.!?]{{0,48}}{_DELIVERY}|"
    rf"{_DELIVERY}[^。！？.!?]{{0,36}}(?:受阻|被阻塞|未建|未创建|还没建|(?:is|are)\s+blocked)",
    re.I,
)
_NONCURRENT = re.compile(
    rf"(?:无[须需]|不(?:再|需|必|用|需要|要求|应(?:该)?|要)?)等(?:待|候)?[^。！？.!?]{{0,18}}{_ENTRY}|"
    r"Jenkins\s*(?:已(?:经)?|早已)退役|\bJenkins\s+(?:is|was|has\s+been)\s+retired\b|"
    r"正在讨论.{0,30}(?:退役|撤下|恢复)|\bdiscuss(?:ing|ion)?\b.{0,48}\b(?:retir\w+|whether)\b|"
    r"不受其阻塞|不是.{0,12}(?:阻塞|前置条件)|不影响.{0,12}(?:提交|交付)|"
    rf"\b(?:not|no\s+longer|don't|should\s+we)\s+wait(?:ing)?\s+for\s+{_ENTRY}|\bnot\s+blocked\b",
    re.I,
)


def _prose(text: str) -> str:
    """Ignore quoted examples and fenced snippets before considering live text."""
    fence = ""
    lines = []
    for line in text.splitlines():
        marker = _FENCE.match(line)
        if marker:
            delimiter = marker[1]
            if not fence:
                fence = delimiter
            elif delimiter[0] == fence[0] and len(delimiter) >= len(fence) and not line[marker.end():].strip():
                fence = ""
        elif not fence and not line.lstrip().startswith(">"):
            lines.append(_INLINE_QUOTE.sub(" ", line))
    return "\n".join(lines)


def is_stale_guidance(text: str) -> bool:
    """Recognize live Jenkins ownership or mandatory entry instructions in root guidance."""
    for line in _prose(text).splitlines():
        body = re.sub(r"^\s*(?:[-*+]\s+|\d+[.)]\s+)?", "", line).replace("**", "")
        if not _HISTORY.search(body) and _LIVE_RULE.search(body):
            return True
    return False


def _path_key(value: object) -> str | None:
    if not isinstance(value, str) or not value or "\0" in value or ".." in re.split(r"[/\\]", value):
        return None
    if re.match(r"^[A-Za-z]:[/\\]", value) or value.startswith(("\\\\", "//")):
        key = ntpath.normcase(ntpath.normpath(value))
        if key.startswith(("\\\\?\\", "\\\\.\\")):
            return None
        return key.rstrip("\\") if len(key) > 3 else key
    if value.startswith("/"):
        return posixpath.normpath(value)
    return None


def _waiting_for_jenkins(text: object) -> bool:
    if not isinstance(text, str):
        return False
    prose = _prose(text)
    # A historical/retirement report is not proof of a current block. Leave
    # ambiguous cases for a human rather than planning an unsolicited prompt.
    if _HISTORY.search(prose) or _NONCURRENT.search(prose):
        return False
    for paragraph in re.split(r"\n[ \t]*\n", prose):
        line = " ".join(paragraph.split())
        if _WAIT_FOR_RECOVERY.search(line):
            return True
        for marker in _UNAVAILABLE.finditer(line):
            context = line[max(0, marker.start() - 120):marker.end() + 120]
            if re.search(_ENTRY, context, flags=re.I) and _BLOCKED_DELIVERY.search(context):
                return True
    return False


def select_sessions(
    rows: Iterable[dict[str, Any]], worktrees: Iterable[dict[str, Any]], caller_id: str,
) -> list[dict[str, Any]]:
    """Select ordinary, unarchived sessions at an exact eligible registered root."""
    eligible = {
        _path_key(tree.get("path")) for tree in worktrees
        if tree.get("stale") is True and not tree.get("locked") and not tree.get("prunable")
    } - {None}
    selected = []
    seen = set()
    for row in rows:
        session_id = row.get("sessionId")
        if not isinstance(session_id, str) or not session_id.strip() or session_id == caller_id or session_id in seen:
            continue
        if row.get("archived") is not False or row.get("origin", "ordinary") != "ordinary":
            continue
        if not isinstance(row.get("running"), bool) or _path_key(row.get("cwd")) not in eligible:
            continue
        if _waiting_for_jenkins(row.get("lastAssistantText")):
            selected.append(row)
            seen.add(session_id)
    return selected


def build_notice(canonical_root: str, pr_url: str) -> str:
    """Clarify workflow without transferring work between tasks or adding authority."""
    return (
        "按用户当前要求，Jenkins 和旧协调器均已退役：不恢复、不等待这些服务，也不重放历史队列。\n"
        f"请读取规范工作树的 AGENTS.md（{canonical_root}）；旧树中的 Jenkins 必经指引仅作历史资料。\n"
        "在已有授权范围内使用原生 Git 和任务自己的工作树；保留其他改动、锁、回执和真实验收门槛，"
        "不新增任何提交、推送、通知或验收授权。\n"
        "已获准的独立问题先开独立 Draft PR，再在同一 PR 补回归与修复，实际验收后才能合并。"
        f"工作流交接修复的参考 Draft PR：{pr_url}；不是把其他任务迁入该 PR 的授权。\n"
        "本工作流修复不得夹带 BD02 或其他任务的改动；各任务既有授权不变，"
        "本交接不新增触碰任何生产代码的权限。"
        "请回复确认是否采纳，并说明尚存阻塞；收到本消息不等于任务已恢复。"
    )


def plan_handoffs(
    rows: Iterable[dict[str, Any]], worktrees: Iterable[dict[str, Any]], caller_id: str,
    canonical_root: str, pr_url: str,
) -> list[dict[str, str]]:
    """Build stable request IDs scoped to this repository, Draft PR and session."""
    notice = build_notice(canonical_root, pr_url)
    plan = []
    for row in select_sessions(rows, worktrees, caller_id):
        session_id = row["sessionId"]
        identity = json.dumps(["zircon-workflow-handoff-v1", _path_key(canonical_root) or canonical_root, pr_url, session_id])
        plan.append({
            "sessionId": session_id,
            "requestId": str(uuid5(NAMESPACE_URL, identity)),
            "mode": "steer" if row["running"] else "queue",
            "text": notice,
        })
    return plan


POLICY_START = "<!-- zirconengine-local-workflow-policy:start -->"
POLICY_END = "<!-- zirconengine-local-workflow-policy:end -->"


def render_global_policy(canonical_root: str, pr_url: str) -> str:
    """Render a narrow user-global overlay; the caller owns installation and review."""
    if _path_key(canonical_root) is None or any(c in canonical_root + pr_url for c in "\r\n\0"):
        raise ValueError("Policy references must be an absolute root and single-line PR URL")
    return (
        f"{POLICY_START}\n"
        "## ZirconEngine 当前用户工作流（2026-11-17）\n\n"
        f"仅适用于规范仓库 {canonical_root} 及与其共享 Git common-dir 的登记工作树；其他仓库不受本节影响。\n\n"
        "- Jenkins 和旧协调器均已退役：不恢复、不等待，不重放历史队列。旧工作树里相反的必经入口或排他所有权要求已被用户撤回，不能继续作为阻塞条件。\n"
        "- 在原任务既有授权范围内直接使用原生 Git、任务工作树和 GitHub；不再为跳过已退役服务重复索取例外。真正缺少动作授权、GitHub 权限或网络条件时，只报告实际缺项。\n"
        "- 发现独立问题先开独立 Draft PR，再在同一 PR 补回归、提交修复；实际验收通过后才可合并。不得夹带其他问题，也不因已有工作获得额外提交、推送、发布或清理授权。\n"
        f"- 旧会话恢复或读到冲突规则时，请读取规范工作树的 AGENTS.md（{canonical_root}），并按当前用户指示继续；保留真实验收门槛。\n"
        "- 保留其他会话源码、索引、未提交修改、锁和历史回执；不编辑会话数据库，不批量覆盖、合并或变基旧工作树。\n"
        f"- 此次工作流修复及验收记录：{pr_url}。它不是其他引擎问题的交付 PR。\n"
        f"{POLICY_END}"
    )


def merge_owned_policy(existing: str, policy: str) -> str:
    """Replace only this task's owned section, preserving all unrelated bytes as text."""
    if policy.count(POLICY_START) != 1 or policy.count(POLICY_END) != 1:
        raise ValueError("Policy must contain exactly one complete owned section")
    start_count, end_count = existing.count(POLICY_START), existing.count(POLICY_END)
    if start_count == end_count == 0:
        separator = "" if not existing or existing.endswith("\n\n") else "\n" if existing.endswith("\n") else "\n\n"
        return existing + separator + policy + "\n"
    if start_count != 1 or end_count != 1:
        raise ValueError("Ambiguous owned policy markers; preserve existing content")
    start, end = existing.index(POLICY_START), existing.index(POLICY_END)
    if end < start:
        raise ValueError("Owned policy markers are out of order")
    return existing[:start] + policy + existing[end + len(POLICY_END):]


def deliver_handoff(plan: dict[str, str], prompt_fn: Callable[[dict[str, str]], bool | None]) -> dict[str, Any]:
    """Call the authorized adapter once; ambiguous outcomes retain their request ID.

    True/False are explicit accepted/rejected receipts. No receipt (including a
    timeout or other adapter exception) is unknown, not a signal to resend.
    """
    receipt = {"sessionId": plan["sessionId"], "requestId": plan["requestId"], "adopted": False}
    try:
        accepted = prompt_fn(plan)
    except Exception:
        accepted = None
    receipt["deliveryState"] = "accepted" if accepted is True else "rejected" if accepted is False else "unknown"
    return receipt
