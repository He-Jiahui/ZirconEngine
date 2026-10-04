---
title: Editor08 keymap 设置回归的环境锁修复记录
category: zircon_editor
date: 2026-09-30
implementation_status: source_applied_structurally_checked_attributed
validation_status: managed_editor_tests_not_run
performance_status: product_gates_open
---

# Editor08 keymap 设置回归的环境锁

真实回归 `host_and_manager_service_share_the_user_settings_keymap` 使用进程级 `SETTINGS_USER_ROOT_ENV`，并创建会修改 `ZIRCON_CONFIG_PATH` 的 EventRuntimeHarness。修复前的 `let _guard = env_lock();` 只持有静态锁引用，没有取得互斥 guard。源码本身以 `CR-EDITOR-SERVICES-0005` 标记了该问题。

已应用的修复先执行 `env_lock().lock().unwrap()`，再进入原有设置流程；仅删除已由候选修复的两行 BUG 注释。既有设置写入、环境恢复、host/manager keymap 一致性和修改传播断言全部保留，没有新增镜像测试。候选现已应用到共享源码并归属。

## 支持层与静态证据

`zircon_editor/src/tests/support.rs` 的 TestEnvironmentLock 在 `.lock()` 中恢复 poisoned mutex 并返回实际 guard。`.unwrap()` 沿用现有测试约定。guard 是测试中第一个绑定，逆序销毁使其在 Harness 的构造、设置操作和 Drop 期间保持持有；已检查的调用链没有重复获取该锁。

| 证据 | SHA-256 |
|---|---|
| 源码及冻结原始字节 | `504d478397009f094b60eed94835e09ed2249ab6fd3c5267c73197324f63b8de` |
| 修复候选 | `7cddcd1799a133f2c6f0b3fb58c951005fc5900f348e3c97343fecbe62f01d28` |
| 最小 patch | `8f4cb785a02aa992b9974a5473810678e3cd1434d15680be507becb19f0f3a16` |
| Root 独立静态复核 | `48d369a87cbb5885679ea53172fb290850274ad8ef5180f7e8c80ba20fdc49dd` |

候选包位于 `.codex/state/session-coordinator/async-validation-batches/offline-candidates/editor-keymap-test-environment-lock-v1`。独立复核还核对了锁支持层和 Harness 的当前字节。同步行为发生变化；测试断言和设置工作流没有变化。上述证据没有运行 Cargo/typecheck 或动态测试。

## 应用与验收状态

2026-09-30 的单路径 public 预览确认该源码仍由活动 M1 会话 `1d9a143c-c6a1-4da3-a702-eeeff3e26b8f` 持有，返回 `source_owner_executable`。预览日志 `2026-09-30-editor-keymap-test-env-lock-single-public-source-preview.json` 的 SHA 为 `410733de59b62e85432a0b88ecd0c9e2ecfbedf0839fba037732ce048e4fd1ea`。当时未转移所有权、claim lease 或改写共享源码。下述新准入和应用回执记录后续状态。

Editor04 的两个真实场景视口测试也只持有 `env_lock()` 引用，其独立修复候选同样等待所属会话移交。当前候选不能证明三个调用点已共同修复，也不能证明其他环境测试已经消除竞争。

下一批将现有真实回归纳入完整 Editor 分组 managed 测试，保留正常并发测试配置；只执行命名测试或以单线程成功不能验收跨测试环境隔离。Contracts 的独立批次不覆盖 Editor 测试。本修复是测试正确性修复，不提供产品延迟、分配或 RSS 达标证据。

主计划：[Editor08](../08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md)；关联 [Editor12 设置](../12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md)。完成列表位于 `docs/plans/astra/features/editor/1068-editor08-keymap-settings-test-environment-lock-completion-list.md`。

2026-10-01 UTC：原 M1 会话现为 `cancelled`。新的单路径公共预览允许接管；Root 以确认 fingerprint 完成转移，取得精确 live lease，复核原始 SHA 后应用了候选，格式和 scoped diff 检查通过并完成 baseline attribution。源码应用回执：
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor08-keymap-environment-lock-cancelled-owner-guarded-apply.json`
（SHA `7ae578171ada0a7771a8f696703582e7bc2afac632fed908212abe0236507c21`）。

本次还复核了已应用的 Harness 环境变量恢复守卫：构造与 Drop 路径不获取环境互斥锁。支持层复核与两个调用点的归属预览：
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor-keymap-and-scene-lock-exact2-public-preview-after-harness-and-palette-repairs.json`
（SHA `ff801ff22e31b32e097f0808191469cbddcee746b69b702efb4b71376f1c063e`）。
场景视口的两个 guard 修复仍由可执行所有者阻挡；本次只写入 keymap 源码。完整 Editor managed 并发回归及产品性能门槛保持开放。
