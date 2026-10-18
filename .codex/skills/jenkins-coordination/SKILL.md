---
name: jenkins-coordination
description: Run functional Jenkins coordinator tests or submit bounded commands through the independent Jenkins pilot and Windows tray.
---

# Jenkins coordination

Use the currently verified Jenkins management entry and runtime. The retired local coordinator, service, tray, scheduled tasks, leases, attribution APIs, and queue must remain untouched.

## Lifecycle ownership

Jenkins owns all version operations, branch/worktree lifecycle and multi-worktree
merges, source preparation, file-generation recipes, builds/tests, artifact reuse,
task cleanup and scheduled cleanup. Agents read, edit explicitly authorized
canonical files and hand off scoped inputs; they do not run substitute local
Git/generation/build/cleanup workflows without an explicit user exception.
Use the [management standard](../../../docs/tools/agent-artifact-management.md).

For the current formal flow, inspect the selected
`.jenkins/state/deployment/driver.json`, its immutable implementation and actual
runtime evidence. The existing `zircon-flow`, `zircon-execution` and
`zircon-maintenance` jobs own input/integration, execution/reuse and maintenance.
[The incremental entry](../../../tools/dev/jenkins_validate.py) supports scoped
handoff; verify its selected-driver capability before use. The pilot enablement
and bundle interface below apply to that pilot, not as a substitute for the
formal incremental route. A job/specification alone does not prove activation
of multi-worktree operations or scheduled cleanup.

## 核心项目与入口选择

复用下列核心项目，根据任务选择入口：

| 项目 | 用途 |
| --- | --- |
| [zircon-authorization](http://127.0.0.1:18080/job/zircon-authorization/) | 仍被管理工具及未完成请求调用的操作登记 |
| [zircon-execution](http://127.0.0.1:18080/job/zircon-execution/) | 共享构建与验证执行 |
| [zircon-flow](http://127.0.0.1:18080/job/zircon-flow/) | 任务流程编排 |
| [zircon-governance](http://127.0.0.1:18080/job/zircon-governance/) | 产物和存储治理 |
| [zircon-maintenance](http://127.0.0.1:18080/job/zircon-maintenance/) | 受管维护入口 |
| [zircon-management](http://127.0.0.1:18080/job/zircon-management/) | 公开管理入口；工作树、准备、验证等受管操作 |
| [zircon-merge-queue](http://127.0.0.1:18080/job/zircon-merge-queue/) | 唯一定时分支集成；验证实际候选后推进 main |
| [zircon-management-tests](http://127.0.0.1:18080/job/zircon-management-tests/) | 协调器回归验证；保留历史 #30 的 66 项成功证据 |
| [zircon-records](http://127.0.0.1:18080/job/zircon-records/) | 收件与请求登记 |
| [zircon-records-followup](http://127.0.0.1:18080/job/zircon-records-followup/) | 未完成请求的接续与回执核对 |
| [zircon-scheduled-cleanup](http://127.0.0.1:18080/job/zircon-scheduled-cleanup/) | 定时清理；保留现有 `H/30 * * * *` 触发器 |

源码、分支或工作树的准备与验证交给 `zircon-management`；组合任务由
`zircon-flow` 编排，构建和复用由 `zircon-execution` 承担。产物治理使用
`zircon-governance`，维护使用 `zircon-maintenance`；定时清理由
`zircon-scheduled-cleanup` 触发。

收件使用 `zircon-records`；已有请求的等待、失败或交付不明，使用
`zircon-records-followup` 对账并接续原请求。普通本地补丁和验证自动登记，
不要求用户先手工运行 `zircon-authorization`，参见下方 Local request intake。

协调器修复通过 `zircon-management-tests` 验证。历史 #30 仅证明对应版本的
66 项回归；每次任务仍核对当前版本、实际构建回执及原生终止证据，
协调器回归不能替代产品编译、定向测试或界面验收。

## Interpreter ownership

Reserve `.jenkins/runtime/python` for the deployed Jenkins runtime, its jobs and
supported clients with a managed lifecycle. Use general Codex Python for ad hoc
`-c` or stdin scripts, interactive patch workers, and unrelated maintenance helpers.

Use an available general Codex runtime Python for ordinary read-only inspection,
payload preparation and skill maintenance. Keep `-B` and the existing output
permissions. Explicitly authorized Jenkins bootstrap tests may use the dedicated
runtime; wait for their actual process and output termination before startup.

Current service ownership follows exact PID and birth identities, native and
lifecycle journals, and actual service-control entries. Deployment locks, Jenkins
Home ownership and endpoint exclusivity still apply; interpreter identity alone
does not establish service ownership. Normal request clients and ordinary helpers
may coexist with the service. Retain the raw inventory; client/helper classification
does not prove service death or issue native termination evidence.

Never terminate the Jenkins host, keeper, supervisor or JVMs to free the interpreter
for another helper. Use the public lifecycle entry bound to the currently selected
driver and runtime generation for lifecycle operations.

## Standing task delivery authority

The latest 2026-10-17 user instruction makes all ordinary Jenkins task validation
**optional and advisory**. Any session may commit its own scoped files immediately,
without a validation recipe, source-hash reconciliation, full compilation, formal
acceptance, historical capsule acceptance or a clean shared main. Native Git
preserves task scope, active locks, foreign changes and merge conflicts. Failed
checks remain failures and create original-owner repair handoffs; they do not
block commits or PR integration. The owner fixes and commits corrections without
another approval. Read the current delivery contract and executable policy;
older validation-before-commit or mandatory-green instructions below are historical.
Do not claim a stored inbox handoff has awakened or been read by the original chat.

The user's latest 2026-10-17 order is **check the task worktree's index.lock,
commit its exact files to its own branch immediately, then enqueue integration**.
Keep the shared `main` checkout clean; implementation belongs in a Jenkins-managed
task worktree. The latest 2026-10-17 user instruction further requires a real PR:
the unified scheduled flow validates the candidate containing current main, then
synchronizes local `main` only from the verified merged PR and identical tested
result tree. It must not publish a local candidate directly. Any session may submit; no additional per-chat grant,
commit/merge approval or whole-workspace acceptance is required. Follow the
[branch and scheduled integration contract](../../../docs/tools/jenkins-branch-integration.md)
and executable policy. Use `tools/dev/jenkins_branch_queue.py submit --input-file
<task-request.json>` with the original ready worktree/session and ownerThreadId.
Branch commit SUCCESS means integration pending, not validation or main success.

Resolve the real Git directory with `rev-parse --absolute-git-dir`; a linked
worktree's `.git` is a file. Check its index.lock before every mutation, and main's
lock again before publication. Preserve unknown locks until original ownership
and native outcome permit Jenkins recovery. Age/empty bytes/no current handle
alone do not grant release. Do not scan/commit foreign whole-tree dirt as scope.
The single `zircon-merge-queue` uses `H/15 * * * *` with concurrency disabled.
Validation failures retain branch commits and send exact original-owner inbox
handoffs; distinguish persisted handoff, Codex API acknowledgement and owner read.

Keep task ownership and appropriate merge validation. Scope and validation are not
additional human permission steps. Preserve other sessions' staged/working
changes; retain conflicts and unknown deliveries and resume their original
identity. Push, publication and external notification still require their own
authorization. Historical requests with `allowCommit: false` keep their original
input; start the authorized delivery from their verified result rather than
rewriting or replaying that request.

## Local request intake

Ordinary local patch and validation requests use automatic local request
registration. A missing session row is not a reason to ask the user for a grant,
run a manual authorization job, or stop another chat. The verified installed
management capsule records the local runtime owner, stable implementation and
validation context, and exact request inputs automatically. Session IDs identify
requests; they are not a human approval step.

Keep exact patch, dependency and source hashes, physical output checks, source
conflict checks and authoritative build receipts. Existing narrow worktree writer
contexts retain their original scope. Local task commit/main integration uses
the standing authority above; push and notification keep their separate rules.

Reconnect a known rejected, unsubmitted request using its recorded lineage and
the current runtime. Preserve its original rejection. An unknown delivery is
reconciled under its original identity without a duplicate submission. If the
installed receiver does not support automatic intake, report and repair that
capability within the current task instead of requesting per-session permission.

## Incremental validation

For engineering validation, use the user's [source ownership and incremental patch workflow](references/incremental-patch-validation.md): agents edit the authorized checkout and hand off scoped patches and hashes; the independent Jenkins coordinator prepares and reuses the source baseline, applies patches and runs the relevant build and tests. Agents must not create source snapshots, backups or source bundles. Keep required acceptance pending while continuing independent work and reconciling the original request.

A bundle-only pilot interface does not satisfy this handoff. Verify that the selected coordinator accepts incremental patches and prepares its own sources before submission. If the route is unavailable or unverified, report that condition and preserve the prepared patch; do not create a local snapshot or bundle to work around it.

## Enablement

Use `python -B -m tools.jenkins.jenkins_coordinator --repo-root <absolute-repo-root> status` to check `.codex/state/jenkins-coordinator/active.json` before activated pilot work. Require:

- `enabled: true` and `functionalTestsAllowed: true`;
- the configured repository and pilot root match the current physical identities;
- the current storage owner and keeper are live with exact PID creation times;
- immutable driver, source snapshot, and tray acceptance evidence match the active configuration.

If a check fails, perform read-only inspection and report the blocking evidence. Do not start, adopt, or repair an unknown process from a PID alone.

## Functional flow

For existing activated pilot requests, use the `tools.jenkins.jenkins_coordinator` entry and fixed templates only:

- `python-static` for Python checks;
- `managed-cargo-check-v2` for Cargo checks;
- `fault-probe` for controlled failure tests.

Bind each new request to a fresh `RequestIdentity`, coordinator-issued source identity, driver digest, and journal record. Preserve historical request IDs, attempts, receipts, artifacts, and failed results. If a request is pending or requires reconciliation, reconcile that exact request and attempt; never submit a fresh retry to hide an unknown outcome.

Run the configured, hash-verified `python.exe` with `-B` from the repository root, using `-m tools.jenkins.jenkins_coordinator --repo-root <absolute-repo-root>`. The pilot entry provides `status`, `submit --request-file <absolute-json> --bundle <absolute-zip>`, `reconcile --request-file <absolute-json>`, `cancel --request-file <absolute-json>`, and `verify --request-file <absolute-json>`. Use the same request file for the entire lifecycle. A bundle must already have been issued by the coordinator for that request; agents must not invoke `snapshot.capture` or `snapshot.materialize` directly. Functional tests needing captured source fixtures also require an implemented coordinator preparation path. Include new files only through an explicit allowlist. A submitted queue ID or terminal Jenkins result alone is not validation acceptance: require `verify` to accept exact local and archived bytes and native termination evidence.

When the configured service is stopped, use the tray or its `start` management entry within the task's authorization. It performs the existing keeper recovery protocol after confirmed native death. Never replace the configured root, manufacture an activation receipt, or automatically start a service merely to answer a read-only status request.

Keep all compiler products and caches under root-level `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`. Do not use repository-local targets, aliases, nested lookalike roots, or other drives. Do not print credentials, agent secrets, or raw Java command lines.

## Acceptance boundary

Coordinator operation enables functional tests and bounded command submission.
Whole-workspace, milestone, migration and production acceptance remain separate.
The user's standing task-delivery authority above covers local scoped commits
and main integration; it does not authorize pushes, publication or external
notifications. Preserve retired coordinator data and historical receipts.

## References

- [Activated pilot entry](../../../tools/jenkins/jenkins_coordinator.py)
- [Independent pilot](../../../tools/jenkins/pilot/)
- [Windows tray and recovery workflow](../../../docs/tooling/jenkins-tray.md)
- [Pilot operator contracts](../../../docs/tooling/jenkins-coordinator-pilot.md)
- [Acceptance plan](../../../docs/plans/jenkins-coordinator-pilot.md)
