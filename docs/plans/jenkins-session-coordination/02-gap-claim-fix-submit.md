---
related_code:
  - tools/jenkins/gap/
  - tools/jenkins/gap/extractor.py
  - tools/jenkins/gap/state.py
  - tools/jenkins/gap/cli.py
  - .jenkins/pipeline/zircon-gap.groovy
  - .jenkins/state/coordination.sqlite3
implementation_files:
  - tools/jenkins/gap/__init__.py
  - tools/jenkins/gap/extractor.py
  - tools/jenkins/gap/state.py
  - tools/jenkins/gap/cli.py
  - .jenkins/pipeline/zircon-gap.groovy
plan_sources:
  - "user: 2026-10-04 规范化缺口文档，与 Jenkins 结合做认领-修复-提交流程"
  - docs/plans/jenkins-session-coordination/01-single-host-jenkins-coordination.md
  - docs/tooling/jenkins-session-coordination-architecture.md
  - docs/plans/jenkins-coordinator-pilot.md
tests:
  - tools/jenkins/tests/test_gap_extractor.py
  - tools/jenkins/tests/test_gap_state.py
  - tools/jenkins/tests/test_gap_cli.py
doc_type: milestone-detail
status: implementing
created: 2026-10-04
---

# 缺口文档规范化与 Jenkins 认领-修复-提交流程

本计划为 `docs/plans/optimize/` 系列缺口账本文档建立标准化工作单元，通过 Jenkins
协调器实现多 Session 不冲突的认领、修复、验证、提交闭环。

不扩展引擎功能，不改变 `docs/plans/optimize/` 文档的所有权，不绕开现有
`managed-cargo-check-v2` 验收边界。

## 架构概览

```
docs/plans/optimize/          ← 缺口账本（只读输入）
    zircon_runtime/NNN-*.md
    zircon_editor/NNN-*.md
    …

tools/jenkins/gap/            ← 新增支撑模块
    extractor.py              ← 解析缺口文档 → Issue 记录
    state.py                  ← SQLite CRUD（gap_issues / gap_claims）
    cli.py                    ← 命令行入口

.jenkins/state/coordination.sqlite3   ← 复用现有状态库，新增两张表
.jenkins/pipeline/zircon-gap.groovy   ← 新增 Pipeline，复用 waitControl 架构
```

## 数据模型

### gap_issues 表

```sql
CREATE TABLE gap_issues (
    issue_id    TEXT PRIMARY KEY,  -- {doc_slug}:{priority}:{seq}，如 runtime223:P0:001
    doc_path    TEXT NOT NULL,     -- 相对仓库根的文档路径
    priority    TEXT NOT NULL,     -- P0 / P1 / P2
    gate_id     TEXT,              -- 原文门控 ID，如 RT-VIS-P1-007
    summary     TEXT NOT NULL,     -- 从文档提取的缺口摘要（不超过 200 chars）
    status      TEXT NOT NULL DEFAULT 'open',
                                   -- open / claimed / fixing / submitted / accepted
    source_fingerprint TEXT,       -- 提取时文档的 SHA256
    extracted_at TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE gap_claims (
    claim_id    TEXT PRIMARY KEY,
    issue_id    TEXT NOT NULL REFERENCES gap_issues(issue_id),
    session_id  TEXT NOT NULL,
    claimed_at  TEXT NOT NULL,
    released_at TEXT,
    jenkins_request_id TEXT,
    jenkins_build_id   TEXT,
    receipt_ref TEXT,
    outcome     TEXT             -- null / passed / failed / abandoned
);
```

Issue ID 格式固定：`{doc_slug}:{priority}:{seq}`，其中 `doc_slug` 为文档路径的
last segment（去掉编号前缀和 `.md`），`seq` 为同文档同优先级的序号（三位零填充）。

## 缺口文档规范化

提取器运行后，在对应文档 frontmatter 写入 `gap_ids` 字段：

```yaml
gap_ids:
  - runtime223:P0:001
  - runtime223:P1:007
```

这是只读参考字段，认领状态以 SQLite 为权威，不在文档中写入 `claimed_by`。

## 命令行接口

```
python -m tools.jenkins.gap extract [--dir docs/plans/optimize] [--priority P0]
    扫描缺口文档，写入 gap_issues，更新文档 frontmatter 的 gap_ids。
    幂等：已有 issue_id 的条目跳过，fingerprint 变化的条目标记 needs_reextract。

python -m tools.jenkins.gap list [--priority P0] [--status open] [--limit 20]
    列出缺口条目。

python -m tools.jenkins.gap claim <issue_id> --session <session_id>
    认领一个 open 条目，写入 gap_claims，状态变为 claimed。
    若已被其他 session claimed 且未 released，拒绝并报告。

python -m tools.jenkins.gap submit <issue_id> --session <session_id> --patch-ref <ref>
    将已 claimed 条目提交到 Jenkins gap-fix job，状态变为 submitted。
    使用现有 submission.py 的 Jenkins HTTP API，复用 waitControl 轮询。

python -m tools.jenkins.gap accept <issue_id> --receipt-ref <ref>
    记录 Jenkins 验收回执，状态变为 accepted，同时更新缺口文档的门控状态。

python -m tools.jenkins.gap release <issue_id> --session <session_id>
    放弃认领，状态回到 open，claim 记录 outcome=abandoned。
```

## Jenkins Pipeline：zircon-gap.groovy

复用 `zircon-flow.groovy` 的 `invokeControl` / `waitControl` 架构，新增 `gap` domain：

```
stage('Claim validation')      invokeControl('gap', 'validate-claim', {...})
stage('Apply patch')           waitControl('flow', 'prepare-patch', {...})
stage('Cargo check')           waitControl('cargo', 'managed-check', {...})
stage('Accept gap issue')      invokeControl('gap', 'accept', {...})
```

job 参数：`ISSUE_ID`、`SESSION_ID`、`PATCH_REQUEST_REF`、`BUILD_ROOT`。

## 里程碑顺序

| 里程碑 | 交付内容 | 验收门槛 |
|---|---|---|
| M1 | `gap/state.py`：两张表的建表、CRUD、事务 | `test_gap_state.py` 全部通过 |
| M2 | `gap/extractor.py`：解析 P0/P1/P2 条目，写入 DB 和 frontmatter | `test_gap_extractor.py` 全部通过；扫描 `optimize/zircon_runtime/` 无报错 |
| M3 | `gap/cli.py`：`extract list claim release` 命令 | `test_gap_cli.py` 全部通过 |
| M4 | `zircon-gap.groovy` + `gap/cli.py submit accept` | 真实 Jenkins job 提交并完成一个 P1 缺口的完整闭环 |

M1-M3 不触碰 Jenkins 服务，可在服务离线期间完成。M4 依赖服务恢复上线。

## 约束

- 不恢复退役的旧协调器路径。
- `gap_issues` 只追加，不删除；状态回退路径为 `submitted → claimed`（Jenkins 失败时）。
- `extract` 不修改缺口文档的实质内容，只写 `gap_ids` frontmatter 字段。
- 每个 issue_id 在任何时刻最多有一个 `outcome=null` 的 claim（活跃认领唯一）。
- `accept` 必须绑定真实 Jenkins receipt_ref，不接受空回执。
