from __future__ import annotations

import re
import unittest
from importlib import import_module
from pathlib import Path
from unittest.mock import Mock


ROOT = Path(__file__).resolve().parents[2]
IMPLEMENTATION = ROOT / "tools" / "dev" / "session_workflow_handoff.py"
CANONICAL_ROOT = r"E:\Git\ZirconEngine"
DRAFT_PR_URL = "https://github.com/example/ZirconEngine/pull/136"
CALLER = "handoff-caller"
LEGACY_TREE = "D:/Fixture/worktrees/legacy"
NESTED_TREE = "E:/Git/ZirconEngine/.agents/worktrees/stale"
WAIT_ZH = "Jenkins 连接被拒绝，受管入口不可用，无法创建 Draft PR 或提交；等 Jenkins 恢复后再继续。"
WAIT_EN = "The managed Jenkins entrypoint refused the connection. I cannot commit until Jenkins is back."


def _worktree(path: str = LEGACY_TREE, **flags: bool) -> dict:
    return {"path": path, "stale": True, "locked": False, "prunable": False, **flags}


def _session(session_id: str, *, cwd: str = LEGACY_TREE, text: str = WAIT_ZH, **fields) -> dict:
    return {
        "sessionId": session_id,
        "cwd": cwd,
        "archived": False,
        "running": False,
        "lastAssistantText": text,
        **fields,
    }


class CheckoutGuidanceRegression(unittest.TestCase):
    def test_canonical_checkout_has_no_live_jenkins_prerequisite(self) -> None:
        # Exercise a real checked-out input independently of the Python implementation.
        # Match a live rule at the start of a bullet sentence, not an inline quote.
        live_rule = re.compile(r"(?:^|[.!?]\s+)(?:Use the independent Jenkins coordinator|Jenkins still owns)\b", re.I)
        rules = (ROOT / "AGENTS.md").read_text(encoding="utf-8").splitlines()
        violations = []
        for number, line in enumerate(rules, 1):
            if line.startswith("- ") and (match := live_rule.search(line[2:])):
                violations.append(f"AGENTS.md:{number}: {match.group().lstrip('.!? ')}")
        self.assertEqual([], violations, "The canonical guide still imposes a retired Jenkins workflow")


class SessionWorkflowHandoffTests(unittest.TestCase):
    def setUp(self) -> None:
        self.assertTrue(IMPLEMENTATION.is_file(), "tools/dev/session_workflow_handoff.py is not implemented")
        self.handoff = import_module("tools.dev.session_workflow_handoff")

    def test_recognizes_both_legacy_guidance_variants(self) -> None:
        for rule in (
            "- Full Access changes command execution permissions. Jenkins still owns Git, branches, worktrees and validation.",
            "- The old local coordinator is retired. Use the independent Jenkins coordinator through the coordination skill.",
        ):
            with self.subTest(rule=rule):
                self.assertTrue(self.handoff.is_stale_guidance(rule))

    def test_retirement_and_quoted_history_are_not_live_requirements(self) -> None:
        for guidance in (
            "- Use native Git on the authorized branch. Jenkins workflows are retired; do not restore legacy services.",
            "- Jenkins workflows are retired. Historical wording only: `Jenkins still owns Git`. This no longer applies.",
        ):
            with self.subTest(guidance=guidance):
                self.assertFalse(self.handoff.is_stale_guidance(guidance))

    def test_only_selects_exact_registered_stale_and_available_worktrees(self) -> None:
        worktrees = [
            _worktree(),
            _worktree(NESTED_TREE),
            _worktree("E:/Git/ZirconEngine", stale=False),
            _worktree("D:/Fixture/worktrees/initializing", locked=True),
            _worktree("D:/Fixture/worktrees/prunable", prunable=True),
        ]
        rows = [
            _session("external", cwd="d:\\fixture\\WORKTREES\\legacy\\", origin="ordinary"),
            _session("nested", cwd=NESTED_TREE, text=WAIT_EN),  # An omitted origin is ordinary.
            _session("clean", cwd="E:/Git/ZirconEngine"),
            _session("locked", cwd="D:/Fixture/worktrees/initializing"),
            _session("prunable", cwd="D:/Fixture/worktrees/prunable"),
            _session("other-repo", cwd="E:/Git/OtherRepo"),
            _session("unregistered-under-root", cwd=NESTED_TREE + "/unregistered"),
            _session("prefix-only", cwd=LEGACY_TREE + "-not-registered"),
        ]
        selected = self.handoff.select_sessions(rows, worktrees, CALLER)
        self.assertIsInstance(selected, list)
        self.assertCountEqual([rows[0], rows[1]], selected)

    def test_never_targets_the_caller_archived_sessions_or_nonordinary_origins(self) -> None:
        eligible = _session("ordinary", origin="ordinary")
        rows = [
            eligible,
            _session(CALLER),
            _session("archived", archived=True),
            _session("child", origin="subagent"),
            _session("scheduled", origin="scheduled"),
        ]
        self.assertEqual([eligible], self.handoff.select_sessions(rows, [_worktree()], CALLER))

    def test_history_and_retirement_discussion_do_not_trigger_delivery(self) -> None:
        rows = [
            _session("retirement", text="Jenkins 已退役；不恢复服务，不等待 Jenkins，按当前授权使用原生 Git。"),
            _session("quoted", text=f"历史交接引用：『{WAIT_ZH}』。目前不受其阻塞，继续原生 Git。"),
            _session("historical-only", text=f"Prior handoff (historical): ‘{WAIT_EN}’"),
            _session("analysis", text="正在讨论撤下 Jenkins 受管入口；连接拒绝只是退役现象，不是提交前置条件。"),
            _session("unrelated", text="Waiting for user acceptance before merging; Jenkins is retired."),
        ]
        self.assertEqual([], self.handoff.select_sessions(rows, [_worktree()], CALLER))

    def test_notice_keeps_retirement_authorization_draft_order_and_confirmation(self) -> None:
        notice = self.handoff.build_notice(CANONICAL_ROOT, DRAFT_PR_URL)
        self.assertIsInstance(notice, str)
        self.assertIn(DRAFT_PR_URL, notice)
        self.assertIn(CANONICAL_ROOT.replace("\\", "/").casefold(), notice.replace("\\", "/").casefold())
        flat = " ".join(notice.split())
        clauses = {
            "retired service": r"(?i)jenkins.*(?:退役|retired)|(?:退役|retired).*jenkins",
            "no revival": r"(?i)(?:不[得要再]?|禁止)(?:恢复|重启|启动)|(?:do not|never)\s+(?:restore|restart|start)",
            "no wait": r"(?i)不[得要再需必]?(?:(?:恢复|重启)[、，/或及和与 ]+)?等待|(?:do not|never)\s+(?:(?:restore|restart)\s+or\s+)?wait",
            "authorized native Git": r"(?i)(?:授权|authori[sz]).{0,56}(?:原生\s*git|native\s+git)|(?:原生\s*git|native\s+git).{0,56}(?:授权|authori[sz])",
            "Draft first": r"(?i)(?:先|\bfirst\b).{0,36}draft\s+pr|draft\s+pr.{0,20}\bfirst\b",
            "independent Draft": r"(?i)(?:独立|单独|separate|independent|isolated|dedicated).{0,24}draft\s+pr",
            "both changes in the same PR": r"(?i)(?:同一|\bsame)\s*(?:draft\s+)?pr",
            "regression": r"(?i)回归|\bregression\b",
            "fix": r"(?i)修复|\bfix(?:es)?\b",
            "acceptance before merge": r"(?i)(?:验收|通过|审核).{0,18}(?:后|前).{0,8}合并|(?:merge only after|merge (?:until|before)).{0,32}(?:accept|review|verif|approv)",
            "exclude BD02": r"(?i)(?:不[得要]?(?:夹带|触碰|修改)|exclude|do not (?:touch|include)).{0,34}\bBD02\b|\bBD02\b.{0,20}(?:不在.*范围|out of scope)",
            "no new authorization": r"(?i)不[得会要]?(?:新增|扩大|增加|授予).{0,36}(?:权限|授权)|no (?:new|additional) (?:authori[sz]ation|permission)|does not (?:authori[sz]e|grant)",
            "read canonical guide": r"(?i)(?:读取|阅读|查阅|read|review).{0,80}agents\.md",
            "confirm adoption": r"(?i)(?:确认|回报).{0,16}(?:采纳|采用)|confirm.{0,36}(?:adopt|appl)",
        }
        for label, pattern in clauses.items():
            with self.subTest(clause=label):
                self.assertRegex(flat, pattern)

    def test_plan_has_stable_distinct_ids_and_modes_for_only_eligible_sessions(self) -> None:
        rows = [
            _session("active", running=True),
            _session("idle", text=WAIT_EN),
            _session("archived", archived=True),
        ]
        args = ([_worktree()], CALLER, CANONICAL_ROOT, DRAFT_PR_URL)
        plan = self.handoff.plan_handoffs([*rows, rows[0]], *args)
        retry = self.handoff.plan_handoffs(list(reversed(rows)), *args)
        self.assertCountEqual(["active", "idle"], [item["sessionId"] for item in plan])
        self.assertEqual({item["sessionId"]: item for item in plan}, {item["sessionId"]: item for item in retry})
        stopped = self.handoff.plan_handoffs([{**rows[0], "running": False}], *args)
        active = next(item for item in plan if item["sessionId"] == "active")
        self.assertEqual("queue", stopped[0]["mode"])
        self.assertEqual(active["requestId"], stopped[0]["requestId"])
        self.assertEqual({"active": "steer", "idle": "queue"}, {item["sessionId"]: item["mode"] for item in plan})
        ids = [item["requestId"] for item in plan]
        for request_id in ids:
            self.assertIsInstance(request_id, str)
            self.assertTrue(request_id.strip())
        self.assertEqual(len(ids), len(set(ids)))
        for item in plan:
            self.assertIn(DRAFT_PR_URL, item["text"])

    def test_explicit_delivery_receipts_do_not_mark_adoption(self) -> None:
        for accepted, state in ((True, "accepted"), (False, "rejected")):
            with self.subTest(state=state):
                self._assert_delivery(Mock(return_value=accepted), state)

    def test_ambiguous_delivery_keeps_original_request_and_never_retries(self) -> None:
        for error in (TimeoutError("no receipt"), ConnectionError("channel disappeared")):
            with self.subTest(error=type(error).__name__):
                self._assert_delivery(Mock(side_effect=error), "unknown")

    def _assert_delivery(self, prompt: Mock, state: str) -> None:
        plan = {"sessionId": "active", "requestId": "original-001", "mode": "steer", "text": "notice"}
        result = self.handoff.deliver_handoff(plan, prompt)
        prompt.assert_called_once_with(plan)
        self.assertEqual("active", result["sessionId"])
        self.assertEqual("original-001", result["requestId"])
        self.assertEqual(state, result["deliveryState"])
        self.assertIs(False, result["adopted"])


if __name__ == "__main__":
    unittest.main()
