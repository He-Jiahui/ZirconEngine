# Editor1068：Editor08 keymap 设置测试环境锁

状态：keymap 源码修复已应用、结构检查通过并归属；完整 Editor managed 并发回归和产品性能验收待完成。

优化记录：`docs/plans/optimize/zircon_editor/08/2026-09-30-keymap-settings-test-environment-lock.md`。

## 完成列表

- [x] 定位 `CR-EDITOR-SERVICES-0005`：keymap 设置回归仅取得 `env_lock()` 引用，随后修改进程全局环境。
- [x] 冻结原始字节和最小实际 guard 候选；独立复核支持层、poison 恢复约定及 Harness 销毁顺序。
- [x] 在原 M1 会话取消后完成公共接管，并取得当前归属、原始字节及精确 live lease 准入。
- [x] 应用并登记候选 SHA `7cddcd1799a133f2c6f0b3fb58c951005fc5900f348e3c97343fecbe62f01d28`；保留所有后续外部修改。
- [ ] 合并 Editor04 两个等待移交的实际 guard 修复，并通过完整 Editor 分组 managed 并发回归。
- [ ] 达到主计划规定的真实产品性能门槛；此测试修复没有提供延迟、分配或 RSS 达标证据。

源码路径：`zircon_editor/src/tests/editor_event/runtime/keymap_settings.rs`。冻结 preimage SHA `504d478397009f094b60eed94835e09ed2249ab6fd3c5267c73197324f63b8de`；独立复核 SHA `48d369a87cbb5885679ea53172fb290850274ad8ef5180f7e8c80ba20fdc49dd`。候选包位于 `.codex/state/session-coordinator/async-validation-batches/offline-candidates/editor-keymap-test-environment-lock-v1`。

早先的 public 单路径预览为活动 M1 所有者及 `source_owner_executable`；当时未应用。后续该会话已取消，新的公共准入允许并完成了本次写入。仍没有 Editor Cargo/typecheck、动态回归或产品性能通过结果。单个命名测试、单线程执行、静态审查或日志写入不满足完整环境隔离与产品验收。

2026-10-01 UTC：原 M1 会话现为 `cancelled`。新的单路径公共预览允许接管；Root 以确认 fingerprint 完成转移，取得精确 live lease，复核原始 SHA 后应用了候选，格式和 scoped diff 检查通过并完成 baseline attribution。源码应用回执：
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor08-keymap-environment-lock-cancelled-owner-guarded-apply.json`
（SHA `7ae578171ada0a7771a8f696703582e7bc2afac632fed908212abe0236507c21`）。

本次还复核了已应用的 Harness 环境变量恢复守卫：构造与 Drop 路径不获取环境互斥锁。支持层复核与两个调用点的归属预览：
`.codex/state/session-coordinator/async-validation-batches/2026-09-30-editor-keymap-and-scene-lock-exact2-public-preview-after-harness-and-palette-repairs.json`
（SHA `ff801ff22e31b32e097f0808191469cbddcee746b69b702efb4b71376f1c063e`）。
场景视口的两个 guard 修复仍由可执行所有者阻挡；本次只写入 keymap 源码。完整 Editor managed 并发回归及产品性能门槛保持开放。
