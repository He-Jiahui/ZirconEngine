---
related_code:
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/core/mod.rs
related_module_review:
  - docs/plans/performance/01/2026-08-15-editor-extension-contribution-overlay-current-architecture-review.md
base_report:
  - docs/plans/performance/01/2026-07-30-editor-core-root-contracts-current-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
tests:
  - tools.tests.test_plugin_extension_registry_finalize_coverage
doc_type: currentness-revalidation
status: static_current_revalidated_dynamic_pending_no_new_p0
---

# Editor core根合同currentness重验（2026-08-23）

## 当前冻结

| scope | Rust文件 | physical lines | bytes | tests | ordered path + NUL + raw bytes + NUL SHA256 |
|---|---:|---:|---:|---:|---|
| root DTO/descriptor/module mounts | 4/4 | 635 | 16,505 | 0 | `3d05a00c16f8c6cc659f14f48619868015572e262caf1be9beb6148481ba7f97` |

4/4文件已完整复读，并追踪operation、startup和plugin registration调用链。与其相邻的
`editor_extension.rs + editor_extension/**`也已按current fingerprint复核：5/5文件、1,540行、
50,847 bytes、5 tests、SHA
`3a23c87b5234807b1768eb04425b7f3138a30741e75b965f584ceb31a3e81777`，与2026-08-15逐文件报告
完全一致，其P0 registration/overlay结论无漂移。

## 逐文件结论

| file | current verdict |
|---|---|
| `editor_authoring_extension.rs` | 纯typed descriptor与move builder。`with_track_type`每次push后sort/dedup，多个capability builder也每次normalize；这是plugin/catalog构建期放大，归既有`PERF-MVP-538`的一次candidate/finalize，不是稳定帧热点。 |
| `editor_operation.rs` | operation path单遍校验且支持borrowed map lookup；`serde_json::Value`被move进DTO。深clone/retention发生在event与pending-edit owner，继续归`PERF-MVP-067/551`，不能用第二serialized DTO修补。 |
| `gui_startup_request.rs` | 一次性owned enum；project path/draft在startup match中移动。manifest/canonical/recent I/O归`PERF-MVP-075/100`，优化该小DTO不能改变F0。 |
| `mod.rs` | 仅模块挂载，无算法、I/O、锁、queue或callback。 |

`TimelineEditorDescriptor`逐项sort可以在10K构建规模放大，但当前产品/fixture只构造极少track types。
正确修复是Editor12的`DescriptorDraft -> validate/normalize once -> frozen contribution generation`，而不是
在每个descriptor发明临时cache或把排序移到稳定读取路径。本轮没有无依据修改production。

## Unreal源码依据

- `PluginManager.cpp:555-594`只在显式`RefreshPluginsList`边界重建discovery map/index；普通稳定运行不
  重复做structural normalize。
- `PluginManager.cpp:2884-2977`为enabled-plugin loading phase建立CPU trace/progress，并强制phase单调。
  Zircon应同样把descriptor construction、validation、factory prepare和一次publication分阶段测量。

只采用“显式结构阶段、一次publication、可测phase”原则；不复制Unreal全局mutable plugin manager。

## 依赖有序验收

1. Editor12在同一prepared candidate中批量收集track/capability/descriptor，normalize/finalize一次并发布
   一个immutable generation；失败发布0次。
2. Editor03继续在fanout/retention owner测JSON clone与retained bytes；root invocation只保留一个payload owner。
3. Editor16继续在project startup owner测canonical/manifest/inventory/recent I/O；root request保持一次move。
4. 构建矩阵为descriptors/track types/capabilities `0/1/100/10K`、duplicates `0/1/50%`：记录sort、
   comparisons、String moved/cloned bytes、candidate builds和publication；要求每generation normalize/publish<=1，
   stable generation work=0。

## 本轮静态门

- 4个root文件加5个editor-extension文件使用`skip_children`隔离后rustfmt 9/9通过。
- plugin finalize coverage初次3/4通过；失败是8月15日hard cut删除`scene_hooks`后测试仍硬编码21 family。
  当前20个typed fields与freeze/is_frozen集合完全相等，计数修正为20后4/4通过。
- 未运行Rust/Cargo或动态scale/F0/F4；没有current-source可执行文件。模块继续pending，不进入
  `review.md`，无里程碑commit或企微通知。
