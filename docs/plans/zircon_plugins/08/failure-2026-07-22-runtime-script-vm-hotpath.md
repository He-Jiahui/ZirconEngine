---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: runtime-script-vm-hotpath
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/script/vm/runtime
  - zircon_runtime/src/script/vm/host_interface
  - zircon_runtime/src/script/vm/gc_bridge
  - zircon_runtime/src/script/vm/reflection
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
tests:
  - cargo test -p zircon_runtime --lib script --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --jobs 1 -- --nocapture --test-threads=1
---

# Plugins08：VM active tables、GC、reflection与discovery性能交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Runtime script 96/96逐Rust文件性能审查，PERF-MVP-444..447
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：Plugins08拥有VM lifecycle、四通道registration、GC、reflection schema和真实backend；M2/M3功能完成记录没有覆盖steady-state active artifact、host deadline、revision cache与bounded discovery。
- 生命周期键：`runtime-script-vm-hotpath`

## 失败现象与复现证据

stage system查询会深clone全部slot records，再构造active generation map、扫描/clone/sort registrations；package调用按name全slot收集排序。GC每frame全slot scan/sort，只信backend自报pause，memory soft/hard policy未执行。reflection prepare与commit重复build registry和验证全部World。package discovery在caller同步递归并读取所有候选完整bytecode，未设深度/文件/bytes/cancel预算。

本轮已让callback generation只读取Copy字段、registered systems消费owned Vec，并把GC pending FIFO从`VecDeque::contains`改为queue+HashSet membership；测试先RED后GREEN且rustfmt/diff-check通过。这些局部修复不构成active table、真实GC deadline、reflection transaction或I/O pipeline验收。

## 最低共享层根因

load/reload已有generation与transaction语义，但没有发布可供稳定帧直接消费的immutable artifacts；因此每次tick/callback/snapshot都从wide lifecycle records重建视图。GC与discovery也缺统一Runtime11 bounded work契约，使backend/I/O工作仍能占用owner thread无上限。

## 架构修复验收

- load/reload/unload原子发布active package index与stage/callback dense ranges；stable callback/tick不调用`list_slots()`、不clone manifest、不scan String package name、不sort全表。
- GC用host wall clock、检查granularity与可续cursor约束overrun；next-due结构不扫描非due slot，soft/hard memory policy有动作、RSS与诊断闭环；panic/error保持FIFO membership一致。
- prepared reflection generation携带一次验证的immutable registry artifact；commit只做provenance/revision检查和短publish，同revision snapshot复用Arc，World按changed type slots同步。
- discovery在bounded I/O worker执行并限制root/symlink/depth/file/manifest/bytecode bytes；第一阶段只读manifest，选中load再single-flight读取共享bytecode，watcher按path generation增量失效。
- 通过Runtime/Plugins08 current-source Cargo、1/100/10k slots/types/worlds/packages、GC真实duration/RSS、cold/warm I/O和F0/F4产品trace。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止让backend自报pause替代host deadline；禁止用更大的默认GC预算掩盖overrun。
- 禁止commit重复validate以换取“安全感”；prepared artifact必须由不可伪造token、revision和catalog provenance保证。
- 禁止简单把同步递归包进无界线程/队列；必须有容量、取消、路径与bytes预算。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

2026-08-11 active-interface snapshot 子切片已完成 current-source 实现：

- `VmHostInterfaceRegistry` 现在以同一个 immutable `Arc` snapshot 保存 active package-name/generation/capability map 与按 stage 分组的 descriptor ranges；staged generation registration 不重建 current snapshot，manager load/reload/unload 返回边界统一发布一次。
- `VmPluginManager::run_registered_systems` 直接遍历 published system slice；`registered_systems`、`registered_behavior_nodes`、`registered_rpc_handlers` 与 `registered_editor_operations` 不再先调用 `list_slots()`，因此稳定查询不再 clone 完整 slot record/manifest 或重新排序全表。
- `slot_for_package_name` 从 published package index 直接取稳定 slot 列表；behavior bridge 从同一 snapshot 读取 active generation，不再每次回调扫描 String package name、排序候选并通过 `slot()` clone manifest。
- lifecycle 失败返回也刷新 active generation；reload rollback 成功继续发布旧 generation，rollback 失败或 unload 失败进入 `Failed` 时不会继续暴露旧 active descriptor。
- GC next-due 也已切为生命周期维护索引：`GcNextDueSchedule` 记录 slot/interval 的下一到期 frame，按 frame bucket + slot stable set 提取 due work；`gc_step` 不再遍历 `slots` 或对 due slot 全量排序，预算未消费工作仍由既有 pending FIFO/membership 跨帧保留。load/reload/unload 的成功、回滚和失败终态统一刷新调度，disabled/backend-managed/failed slot 不进入索引。
- `GcFrameDeadline` 现在从 host `Instant` 计算 frame deadline、每个 backend slice 的实际 wall time 与下一次真实 remaining budget；`VmGcStepReport` 同时保留 backend-reported pause/overrun 和 host elapsed/overrun，调度循环只使用后者。新增 slow-underreport fixture 让 backend sleep 5ms 但自报 0，在 1ms frame budget 下只进入第一个 slot，并产生 host overrun；同步 trait contract 要求 backend 保留可续 collector cursor 并按有界 work interval 检查 host budget。
- reflection prepare 现在生成并携带 `Arc<TypeRegistry>` 与完整 registrations artifact；generation commit 不再调用 `registry_for_state` 或 `validate_candidate` 重建，最终 World 同步仍执行一次最新 live-payload 校验。catalog 原子发布 candidate state、committed snapshot 与 epoch，重复 `current_snapshot` 和 commit 后 snapshot 复用同一 registry `Arc`；同代无变化 prepare/commit 不重建 registry 或扫描 World。changed-type-slot delta/CAS 仍由后续 Runtime13 边界处理。
- discovery 扫描阶段只读取 manifest，按 root/symlink/depth/entry/path/manifest count、单 manifest/总 manifest bytes 与 wall time 拒绝越界输入；manager 把扫描提交到所属 Core 的 Runtime11 `BoundedKeyedIoLane` I/O pool，公开 request 支持启动前取消、运行中协作取消、deadline 与 shutdown，兼容同步入口禁止从同一 I/O worker 自等待。选中 load/reload 才读取 bytecode，按 canonical package containment、单文件/缓存总 bytes 与 entry cap 接纳；相同 path+metadata fingerprint 通过 `OnceLock` 单飞，变更指纹替换旧代，瞬时失败不会永久缓存。
- TDD 结构门 `python -B -m unittest tools.tests.test_plugins08_vm_active_interface_snapshot -v` 为 8/8 GREEN；Rust behavior coverage 已加入 interface/package stable 128 次查询、128 项 staged registration 不重发、reload generation/unload removal、sparse interval/stable order/lifecycle reschedule、honest/underreport host deadline、reflection Arc identity、lazy materialization、discovery budgets 与 async request。固定 Rust 1.94.1 scoped rustfmt 与 `git diff --check` GREEN。
- 参考证据为 Bevy `dev/bevy/crates/bevy_ecs/src/schedule/schedule.rs` 的 changed graph -> rebuilt executable -> stable reuse，以及 Godot `dev/godot/core/object/class_db.cpp` 的 registration-time method map 与 read-time lookup。Zircon 的 intentional divergence 是保留 VM generation rollback，并让 failed lifecycle 返回也显式重发 active set。

本记录仍保持 `open`：当前子切片关闭 active package/interface/system projection 的 steady-state rebuild、GC next-due 全 slot scan/sort、host wall-clock deadline/underreport、prepared reflection artifact 重建，以及 caller-thread discovery/预读全部 bytecode；真实 ZrVM 检查粒度的产品测量、soft/hard memory policy、watcher path-generation 增量失效、changed-type World delta/CAS 和 1/100/10k 产品测量仍未完成。现有 ZrVM binding 没有 per-slot managed-memory bytes，不能用 root count 或全进程 RSS 冒充 memory policy 输入。全局 managed Cargo artifact gate 尚未解除，本轮 `cargo acquire` 客户端超时且查询确认没有生成本会话 job；新增 Rust behavior tests 没有执行结果，不能生成 fixed return。
