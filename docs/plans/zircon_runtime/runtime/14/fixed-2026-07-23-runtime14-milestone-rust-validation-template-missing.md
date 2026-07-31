---
handoff_kind: fixed
status: fixed
created_at: 2026-07-19
summary_slug: runtime14-milestone-rust-validation-template-missing
origin_plan: docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/14
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/control_plane/actions/models.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/tests/test_action_execution.py
  - tools/session_coordinator/tests/test_milestone_cli.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_action_execution
  - python -m unittest tools.session_coordinator.tests.test_milestone_cli
  - python -m unittest tools.session_coordinator.tests.test_validation_copies
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy
  - cargo +1.94.1 test -p zircon_runtime --lib runtime_14_module_family_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --nocapture --test-threads=1
resolved_at: 2026-07-23
---


# Coordinator01: Runtime milestones cannot bind a Rust validation template

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md`
- 来源执行切片：Runtime14 M4 navigation operation owner mirror sync exact3
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Runtime14 的 milestone commit 必须绑定 coordinator-owned immutable validation
  copy，但当前 public milestone API 不能选择任何 Rust/Cargo validation template。

## 失败现象与复现证据

Runtime14 exact3 的最终验收命令是：

```text
cargo +1.94.1 test -p zircon_runtime --lib runtime_14_module_family_mirror_docs_match_structure_audit_counts --locked --jobs 1 -- --nocapture --test-threads=1
```

`milestone validate` 当前只接受 `coordinator-actions` 与 `web-check`。前者运行五组
session-coordinator Python action tests，后者运行 Web npm check；两者都不编译或执行
Runtime14 Rust guard。直接 `cargo run-reserved` 或普通 `validation-copy run` 不会写入当前
workflow run/M4/input fingerprint 的 `workflow_validation_bindings`，因此即使 exit 0 也不能
满足 milestone validation gate。

Runtime14 已有 standalone guard 1/1 与 independent exact3 review C0/I0/M0，但这些仅是
方向性证据。此前两个未绑定 validation-copy Cargo 作业 exit 101 且 stdout/stderr 为空，也
不能作为目标 red 或 green。不得用 `coordinator-actions` 模板冒充 Runtime14 acceptance。

## 最低共享层根因

`ValidationTemplate` 是封闭枚举，CLI choices、action parameter validation、executor command 和
dependency roots 都只覆盖 coordinator/web 两类。Milestone service 只接受由受控 action 创建并
绑定到当前 topology/node/input fingerprint 的 terminal validation evidence；当前没有公开的
arbitrary command binding，也没有 repository Rust focused-test template。

## 架构修复验收

- 新增 coordinator-owned Rust focused-test validation template，命令必须由服务端固定，immutable
  输入必须由 server-owned Cargo metadata planner 收集完整 workspace/local-path closure；调用方不能
  注入任意 shell 或手写不完整 dependency roots。
- Runtime14 M4 模板必须在 immutable validation copy 中执行上述 exact command，并将 copy paths、
  source manifest hash、input fingerprint、terminal status/code 和 job/run identity 绑定到当前
  workflow run/M4 node。
- nonzero、missing target、source hash mismatch、dependency-root materialization failure 和 stale
  topology 必须 fail closed；不能产生 accepted binding。
- CLI、action catalog/parameter parser、executor command/dependency roots、milestone validation 和
  restart/terminal evidence tests 必须覆盖新模板。
- 修复提交后，Runtime14 必须更新 exact3 output record 到最终 `accepted` 字节，再 prepare 新 run、
  运行绑定模板、完成 fresh independent review，最后由 milestone commit 提交 exact3。

## 禁止临时方案

- 不得把 `coordinator-actions` 或 `web-check` 结果标记为 Runtime14 Rust validation。
- 不得开放任意 shell command 给 milestone validate，也不得让业务 Session 自报 source hash。
- 不得复用未绑定的 direct Cargo/validation-copy job，也不得在 validation 后修改 exact3 output
  record而不重跑绑定验证与复审。
- 不得中断或吸收 Coordinator01 当前 proof-bound bootstrap/admission 修复路径；本 failure 等待其
  现有租约自然提交或显式 handoff。

## 修复结果与回传

- 根因：The Runtime14 template initially treated a partial hand-written dependency-root tuple as a Cargo workspace closure, so its immutable copy omitted sibling workspace/local-path packages and could fail before reaching the focused test.
- 架构修复：The closed Runtime14 action now sends its exact server-owned command and exact overlay manifest through `materialize_cargo`; the planner decodes metadata as UTF-8, seeds every workspace member, recursively scans metadata and manifest-only local paths, ignores registry-managed packages, and automatically pins only sibling Git sources required by the closed template. Public caller-driven Cargo copies still require explicit external descriptors. Python and Web templates retain declared dependency materialization.
- 验证：Final dispatch/closure/materialize-and-run focused gates passed 6/6 in 60.654s; affected action, validation-copy, workspace-copy, milestone CLI and workflow modules passed 99/99 in 618.675s. A current-repository exact-command proof materialized 18,027 pinned paths, including root/runtime/interface/derive and both `zr_vm` binding manifests from `E:\Git\zr_vm@40f316fe78daed270d095c7b21152856ae51fed7`, with input manifest hash `89abedca857117c05ef65378b910f3844d0f1861f5d7cbd28f155543e5cd7434` and no Cargo launch. Earlier proof-bound 36/36, reservation/burst 51/51, affected coordinator 153/153 and failure-closeout 17/17 gates remain valid on their unchanged production boundaries.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
