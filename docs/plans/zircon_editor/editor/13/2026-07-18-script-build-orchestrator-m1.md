---
plan: zircon-editor-13
milestone: M1.1
status: implemented-current-source-review-clean-cargo-blocked
session: editor13-script-build-orchestrator-m1-20260718
related_code:
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/core/script_build/mod.rs
  - zircon_editor/src/core/script_build/request.rs
  - zircon_editor/src/core/script_build/orchestrator.rs
  - zircon_editor/src/core/script_build/tests.rs
tests:
  - tools/tests/test_editor13_script_build_orchestrator_contract.py
  - zircon_editor/src/core/script_build/tests.rs
---

# Script Build Orchestrator M1.1

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-18 17:45 +08:00 | `implemented-validation-pending` | 新增唯一 folder-backed `core::script_build` owner。三触发源为 Watch/Command/Play；watch 使用 300 ms 滑动去抖、排序去重，超过 20 路径转为空 module list 的全量编译。每个 request 严格生成 `CompileModules -> ValidateLedger -> RefreshBindings` 三步，单 step in-flight；成功逐步推进，任一步失败原子清除剩余 step、request FIFO 与未成批 watch 路径。Play 只在第三步成功后返回 `resume_play`。错误 request id/no in-flight completion 均 typed fail 且不改变 active request。公开 snapshot 为后续状态栏/Play 投影提供只读状态。 | TDD RED：初始结构合同 4/4 失败；第一版实现后对照严格计划发现只实现 request FIFO，新增 step 合同后 3/4 RED；20 文件性能边界再新增 1/4 RED。最终 `python tools/tests/test_editor13_script_build_orchestrator_contract.py` 4/4，精确 `rustfmt --check` 通过。Rust 行为测试已覆盖去抖去重、显式 flush、20 路径边界、三步顺序、Play 等待/成功恢复、失败清队列、错误 id 不变性，但未运行 Cargo：Coordinator01 仍要求 immutable full compile-input snapshot，且当前共享依赖门禁未释放。本记录不声明编译/单测通过。M1.2 diagnostics、真实 VM、EditorJob/Play/bus/commandlet 接线仍开放。 |
| 2026-07-18 18:58 +08:00 | `implemented-validation-pending` | 异步 completion 硬切为返回原 `ScriptBuildStepDispatch`，同时校验 request id 与 step index；错误 request、旧 step 迟到、no in-flight 均 typed fail 且不改变 active request。删除只传 request id 的旧 completion 入口，不保留兼容重载。 | Async identity 静态合同新增的 2 项断言先 RED，实现后完整 Python 合同 5/5；精确 `rustfmt --check`、scoped diff check 与旧 API 扫描通过。Rust 新增 `stale_step_completion_is_rejected_after_next_step_dispatch`，但共享 Runtime12 Cargo 仍运行，故本行不声明 Rust test 已执行。 |
| 2026-07-22 | `implemented-validation-pending-performance-followup-open` | watch path 在第 21 条 unique path 时立即切 full-rebuild sentinel 并清空 `BTreeSet`；1 万 path 行为测试锁定 resident paths≤20、snapshot sentinel count=21。`last_outcome` 改 `Arc`，snapshot clone 不复制失败 String。 | 历史 standalone `rustc --test` 10/10，Editor13 Python 合同 5/5、源码守卫/rustfmt/diff 通过。持续 watch max latency 与 Command/Play generation single-flight/queue budget 仍 open，见 `failure-2026-07-22-script-build-debounce-admission-backpressure.md`；不宣称 current-source Cargo。 |
| 2026-07-22 | `implemented-current-source-review-clean-cargo-blocked` | current-source 初审 `Critical/Important/Minor=0/2/2`：`ScriptBuildStepDispatch` 已移除 `Clone`，`complete` 改为按值消费线性 ticket；Command/Play/due Watch 先 typed reserve request id，成功后才消费 watch batch，耗尽返回 `ScriptBuildEnqueueError` 且状态不变。新增 20→incremental/21→sentinel 双边界、Play flush 与 exhaustion 原子性行为合同。 | 静态合同先 RED 3/5，再恢复 5/5 GREEN；精确 Rust rustfmt 与 diff-check 通过；增量独立复审 `0/0/0`。持续 watch starvation/无界 Command/Play admission 仍由既有 open failure 跟踪；source-bound Cargo 被 Coordinator01 `validation-copy-external-sibling-path-dependency` 阻塞，failure return 与 managed commit pending，不提升 M1.1。 |
