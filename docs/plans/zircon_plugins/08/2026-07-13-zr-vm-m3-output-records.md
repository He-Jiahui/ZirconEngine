# ZrVM M3 GC 协约产出记录

> Owner：[`../08-zr-vm.md`](../08-zr-vm.md) · 日期：2026-07-13 · Session：`plugins-08-zrvm-m2-20260713`

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | M3-T1 · `HostHandle` 世代语义 | 完成 | 2026-07-13 | `HostHandle { index, generation }` 以无损 `u64` 表示；`HostRegistry` 采用可复用槽位、撤销时递增世代和 typed `HostRegistryError`。`dead_host_object_access_returns_error_not_ub`、stale/vacant/世代耗尽、poison 恢复与 panic 转换回归均通过。 |
| M3 | M3-T2 · `VmObjectRef` RAII root | 完成 | 2026-07-13 | `VmGcRootRegistry` 隔离后端根表，`VmObjectRef` 的 `Arc` lease 保证 clone 共享一个 root、最后一个引用 `Drop` 时只撤销一次；公开面只暴露 opaque object/token，不暴露 VM 裸指针。`dropped_ref_unregisters_gc_root` 及注册失败、后端生命周期、Send/Sync 契约均通过。 |
| M3 | M3-T3 · 增量 GC 预算与诊断 | 完成 | 2026-07-13 | `VmGcBudget`、`VmGcStepReport` 与有界 `VmGcDiagnostics` 已进入共享 script 层；`HotReloadCoordinator` 仅调度 Cooperative slot，以 FIFO pending backlog 保证公平，完整事务由 poison-recovering guard 串行化，panic/普通错误均恢复实例并重新入队。插件以包所有权 ID `zr_vm_language.script.gc_step` 在 Last 注册系统，排序在 `zr_vm_language.systems.last` 后，并登记两项资源。 |
| M3 | Review · 独立规范与质量复核 | 完成 | 2026-07-13 | 规范复核结论为符合 M3-T1/T2/T3；质量复核在修复 panic 丢失、pending 饥饿、label 转换、并发事务竞态与无效并发测试后，最终无 Critical/Important/Minor 发现。生产代码按 `gc_bridge/{host_handle,vm_object_ref,budget}.rs` 与 coordinator owner 拆分，没有在 `mod.rs` 堆叠行为；GC 测试再从 1013 行聚合文件抽到 `hot_reload_coordinator/tests/gc.rs`，主测试 owner 降至 685 行。 |
| M3 | Testing · Windows 分层验证 | 完成（真实后端除外） | 2026-07-13 | Windows toolchain `1.94.1-x86_64-pc-windows-msvc`、受管 pool `e6b9e81a…`、`--locked --offline --jobs 1`：`cargo test -p zircon_runtime --lib --no-default-features --features core-min,script,net-contracts script::vm -- --test-threads=1` 为 81/81；`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime -- --test-threads=1` 为 11/11 + doctest 0/0。初次最小配置缺 `net-contracts` 的 4 个 import 错误已作为验证配置诊断处理；插件首轮暴露无前缀 anchor，改为包所有权系统 ID 后 manifest/registration 双向校验通过。 |

## 未过度声明的验证边界

- `E:/Git/zr_vm/build` 当前不存在，因此 `backend-zr-vm`/真实 collector feature 没有在本切片重新执行；M4 仍负责真实 ZrVM root registry、collector step 和 feature 矩阵。
- 默认路径与共享 runtime-neutral GC 协约已由上述 81 + 11 项测试覆盖；本记录不把 M3 结果提升为 M4 或整个 Plugins 08 完成。
- 共享 Git index 仍有其他会话持有的 Shader 暂存项，本 Session 未 stage、commit 或 closeout，也未把外来警告计入本切片产出。

## 架构与参考实现对位

- Godot：对照 `dev/godot/core/object/object.cpp` 与 `core/extension/gdextension.cpp` 的对象 binding 生命周期；Zircon 使用世代句柄和结构化失效错误，不缓存宿主裸指针。
- Bevy：对照 `dev/bevy/crates/bevy_asset/src/handle.rs` 的共享 lease/最后一个强引用 Drop；Zircon 将同一原则用于 VM GC root 的 exact-once 注销。
- Fyrox：对照 `dev/Fyrox/fyrox-core/src/pool/{handle.rs,mod.rs}` 的世代槽位；Zircon 明确 generation exhaustion 时保持现有 live record，不制造可复用旧身份。

## 结构审查吸收

- 新生产 owner 均低于结构约定软预算；共享常量集中在 `gc_bridge/budget.rs`，没有裸 magic budget/history 数字散落。
- 跨模块失败保持 typed error；锁不跨后端回调，完整 GC transaction 由单独 guard 串行化，panic 经 `catch_unwind` 恢复状态后继续传播。
- 系统清单锚点必须既带 package 前缀又与实际注册 ID 对应，因此计划中的逻辑名 `script.gc_step` 落地为 `zr_vm_language.script.gc_step`；这不是兼容 shim，而是插件所有权硬约束。
- 模块说明见 [`../../../zircon_runtime/script/vm/gc_bridge.md`](../../../zircon_runtime/script/vm/gc_bridge.md) 与 [`../../../zircon_plugins/zr_vm_language/gc_bridge.md`](../../../zircon_plugins/zr_vm_language/gc_bridge.md)。
