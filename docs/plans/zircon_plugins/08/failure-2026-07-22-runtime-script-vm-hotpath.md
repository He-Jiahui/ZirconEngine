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
