---
title: Editor08 EventRuntimeHarness 配置环境恢复记录
category: zircon_editor
date: 2026-09-30
implementation_status: source_applied_and_attributed
validation_status: managed_editor_tests_not_run
performance_status: product_gates_open
---

# EventRuntimeHarness 配置环境恢复

公共测试支持层 `EventRuntimeHarness::with_enabled_subsystems` 在运行时初始化前设置进程级 `ZIRCON_CONFIG_PATH`，成功后无条件删除该变量。这会丢失调用前已有值；初始化或模块激活发生 panic 时，原删除语句不会执行，临时值会污染后续测试。

修复以局部 RAII guard 保存原始 `Option<OsString>`，在正常路径恢复原值，原来没有变量时才删除；异常展开也执行恢复。正常恢复仍位于 `configure_editor_test_runtime_build_set` 之后、manager 解析之前。Harness 不重复获取调用方持有的环境锁，真实 ProjectAuthority/ProjectManager 夹具与配置文件清理保持现有行为。

## 已应用与静态检查

源码路径：`zircon_editor/src/tests/editor_event/support.rs`。

| 证据 | SHA-256 |
|---|---|
| 应用前当前源码 | `24fb7736ceae18eafbb2481fa711552ffc8146c57bbe857fcc8ce1fe2214e796` |
| 应用后源码 | `5e733822a84959278e04fa3a4ac206dd2e901e1436234f8690ca3e39f71f5c1f` |
| 最终源码静态复核 | `0a141f1e36943af8f754a75a6cf71a1242ed7d92bdd4201db0fde31d9e289474` |
| 实际应用日志 | `d7810d555587a1f86a3f85112f0fa130dbb976e7222a1737168883aa0caf2a8d` |

应用日志位于 `.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor-event-harness-config-environment-restore-guarded-apply.json`。精确公共预览确认 archived 所有者可移交；重新核对原始字节、所有者、epoch 后转移至本会话，取得单路径 live lease，原子写入候选。rustfmt 检查和 scoped diff check 均为 0；`baseline.attribute` 确认登记应用后源码。已有外部修改从当前原始字节完整保留。

## 测试与剩余验收

新增三项回归源码：原值和原变量不存在时的恢复、guard 的 panic 展开恢复、真实 EventRuntimeHarness 正常构造后的配置恢复。guard 展开测试没有向 CoreRuntime 注入故障，不能代替完整 Editor 验证。这些 Rust 测试、Cargo/typecheck 和真实产品性能测试均未运行。

此修复恢复环境值，不提供互斥。keymap 和两个场景视口测试的裸 `env_lock()` 调用仍是独立待移交候选；完整环境隔离须在这些修复落地后通过分组 managed Editor 并发回归。单个命名测试、单线程成功或静态审查不能关闭这一门槛。

相关验证加入后续 Editor 批次，提交后继续独立修复，留存原始回执并在工作节点核对，不持续监控编译。产品延迟、分配和 RSS 门槛保持开放。

主计划：[Editor08](../08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md)。完成列表：`docs/plans/astra/features/editor/1069-editor08-event-runtime-harness-config-environment-restore-completion-list.md`。
