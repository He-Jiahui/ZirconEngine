---
record_kind: implementation_plan
status: planned
owner_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
related_code:
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/codex_sync/history.py
  - tools/session_coordinator/tests/test_ai_effort_api.py
plan_sources:
  - user: 2026-07-15 AI 会话工期、挣值、日历工期与预算记录口径
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
---

# Coordinator01 AI 工期与预算账本实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` for this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在本地协调器中保存可审计的 AI 里程碑工期、质量成本、挣值预算和并行日历预测，并同时展示“累计投入”与“当前计划有效工作量”。

**Architecture:** 新增一个独立的 `ai_effort` 领域服务与 SQLite 账本；它只接受受控的结构化输入，不读取 prompt、命令或原始对话。服务把用户确认的历史汇总保存为 immutable baseline，把每个当前计划里程碑保存为独立 ledger row，并由快照/CLI 投影计算 accepted 有效工作量、repair 质量成本和三档有效并行度的日历区间。

**Tech Stack:** Python 3.14、SQLite、现有 session coordinator command/snapshot 模式、unittest。

---

## 口径与不变量

- 历史累计投入与当前计划挣值是两组并列数据，禁止用 `6,005h / 750.6 AI 工作日` 除以当前完成比例外推。
- 每条里程碑账本必须含 `plan_id`、`active_ai_hours`、`outcome`（枚举 `accepted`、`failed`、`superseded`）和 `blocked_by`；可选 `cost_class` 区分 delivery/design 与 repair/validation。
- 只有 `accepted` 的 `active_ai_hours` 计入当前计划“已完成有效工作量”；`failed` repair/validation 仅计质量成本；`superseded` 既不计有效完成，也不计可交付剩余。
- 用户提供的初始 baseline：历史 6,005h / 750.6 AI 工作日，其中 delivery/design 为 5,371h / 671.3，repair/validation 为 634h / 79.3；7 月可追溯为 911.7h / 114 AI 工作日。
- 当前计划预算区间：范围完成 47%，accepted 有效工作量 110–175 AI 工作日，剩余 125–200，合计 235–375；日历预测分别采用有效并行度 1.0、1.6–2.0、2.2–2.8，保留 25–40、18–28、12–20 周的初始沟通区间。
- 任何投影不得输出 prompt、原始 rollout、命令、CWD、绝对路径、token 或 webhook。

## 文件责任映射

- `tools/session_coordinator/migrations.py`：新增 schema 36，建立 baseline、milestone ledger 和 forecast scenario 表，所有 outcome/cost_class 由 SQLite CHECK 约束为枚举。
- `tools/session_coordinator/ai_effort.py`：解析、写入、去重与聚合规则；只接受结构化值并计算 hours/day、quality cost、EV 区间和 calendar weeks。
- `tools/session_coordinator/server.py`：提供 `ai_effort.seed_baseline`、`ai_effort.record`、`ai_effort.report` 三个本地协调器命令；写入仍受协调器 mutation gate 约束，报告只读。
- `tools/session_coordinator/control_plane/snapshot.py`：把简洁预算投影加入实时 snapshot，供现有网页和本地状态页读取。
- `tools/session_coordinator/tests/test_ai_effort_api.py`：覆盖枚举拒绝、accepted-only EV、失败质量成本、superseded 排除、区间和 calendar forecast。
- `docs/tools/session_coordinator/ai-effort-ledger.md`：记录字段语义、隐私边界、录入示例和预测公式。

## M1：结构化账本与用户确认 baseline

**Goal:** 协调器能以枚举约束保存历史投入、当前计划预算和单条里程碑账本，不混淆历史成本与当前 EV。

**Dependencies:** 现有 `Database`、`migrations.py`、`CoordinatorError` 和本地 command dispatch。

**Implementation slices:**

- [ ] 新增 schema 36：`ai_effort_baselines` 保存命名 baseline、口径、AI 小时/工作日、区间和记录时间；`ai_effort_milestones` 保存 stable ledger id、`plan_id`、`active_ai_hours`、outcome、blocked_by、cost_class 和来源 session；对 outcome/cost_class、非负小时、非空 plan_id 建立 CHECK。
- [ ] 在 `ai_effort.py` 定义冻结 dataclass 与 `AiEffortService`，将 8 小时固定为一 AI 工作日，拒绝负数、未知 enum、空 `plan_id`、超长 `blocked_by` 和重复 stable id。
- [ ] 实现受控 baseline seed：写入用户提供的 6,005/5,371/634/911.7 历史数字以及 47%/110–175/125–200/235–375 当前计划预算，不把历史投入自动当成当前 EV。
- [ ] 实现 `ai_effort.record`：写入一个计划里程碑并保留 `blocked_by`；不写计划定义 Markdown，也不从活动会话内容猜测工期。
- [ ] 编写账本单测：非法 outcome/cost_class/小时值必须拒绝；seed 后的 raw historical 与 current-plan budget 独立可读；同一 stable id 重放不重复记账。

**Testing stage:**

- [ ] 运行 `python -m unittest -v tools.session_coordinator.tests.test_ai_effort_api`。
- [ ] 运行 `python -m py_compile tools/session_coordinator/ai_effort.py tools/session_coordinator/server.py tools/session_coordinator/migrations.py`。
- [ ] 运行 `git diff --check -- tools/session_coordinator/migrations.py tools/session_coordinator/ai_effort.py tools/session_coordinator/server.py tools/session_coordinator/tests/test_ai_effort_api.py`。
- [ ] 若服务重载窗口存在，执行一次受控 restart 后使用 `ai_effort.report` 读取 seed 结果；不得为该检查启动 Cargo。

**Exit evidence:** 枚举、seed 分离、重复防护与结构化记录的聚焦测试全绿；本地报告包含两组独立总计。

## M2：挣值、质量成本与日历预测投影

**Goal:** 单一报告可同时显示累计投入、accepted 有效工时、failed 质量成本、被阻塞范围及三档日历预测。

**Dependencies:** M1 ledger rows 和 baseline seed。

**Implementation slices:**

- [ ] 在 `AiEffortService.report()` 中按 outcome 聚合：accepted 进入有效完成；failed 按 cost_class 汇入质量成本；superseded 仅显示为历史排除项；输出每个 `blocked_by` 的当前小时和里程碑数。
- [ ] 实现 range-safe forecast：remaining AI days 除以有效并行度区间，再加可配置串行门禁天数；保持用户确认的配置标签和初始 25–40/18–28/12–20 周沟通区间，且明确预测是区间而非完成承诺。
- [ ] 将 sanitized report 加入 `control_plane/snapshot.py`，仅含 plan id、枚举、数字、日期和 session id；不得包含对话或命令内容。
- [ ] 编写对应测试：accepted + failed + superseded 混合样本只能把 accepted 计入 EV；blocked_by 聚合稳定；并行度越高日历区间不增加；零/负并行度拒绝。
- [ ] 更新 `docs/tools/session_coordinator/ai-effort-ledger.md`，给出三条可复制的本地命令示例和“历史投入不可外推”的醒目说明。

**Testing stage:**

- [ ] 批量运行 `python -m unittest -v tools.session_coordinator.tests.test_ai_effort_api tools.session_coordinator.tests.test_control_snapshot`。
- [ ] 运行 `python .codex/skills/zircon-project-skills/write-plan-output-records/scripts/audit_plan_output_records.py --repo-root E:\Git\ZirconEngine`，只处理本次造成的归属问题，不修改外部计划。
- [ ] 运行 `git diff --check -- tools/session_coordinator/ai_effort.py tools/session_coordinator/control_plane/snapshot.py tools/session_coordinator/server.py docs/tools/session_coordinator/ai-effort-ledger.md docs/plans/zircon_tooling/session_coordinator/01/2026-07-16-ai-effort-and-budget-ledger.md`。
- [ ] 在下一次无外部 Cargo 的受控 reload 后，调用 `ai_effort.report`，核对 750.6、671.3、79.3、114、47% 和三档日历区间均与本计划一致。

**Exit evidence:** 报告同时给出 raw historical、current-plan EV、质量成本、阻塞维度和 calendar forecast；网页 snapshot 可显示同一 sanitized 数字。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
