---
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads
  - zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs
  - zircon_editor/src/core/extension/store/model/snapshot.rs
  - zircon_editor/src/core/editor_extension/view_descriptor.rs
  - zircon_editor/src/ui/retained_host/app/pane_payload_visibility.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
tests:
  - pane-payload current slice 3 of 3 Rust files reviewed, 332 lines, 2 tests
  - direct source-owner, visibility and projection chain 7 files rechecked, 1832 lines
  - scoped rustfmt 3 of 3 clean
  - tracked Rust source has no concrete EditorUiTemplatePaneDataSource implementation
  - current-source Cargo, plugin-scale callbacks, F4, WPR, Tracy and energy pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Editor pane-payload/plugin snapshot current-source审查（2026-08-15）

## 当前范围与正向边界

`host_lifecycle/pane_payloads.rs`和`pane_payloads/**`当前3/3个Rust文件为332行、315个非空行、2条测试，path+content fingerprint为`7D6466D2F0AAFBA7582770E07B11CF7A7BBD8345649A83044DC8FE9DCDA4E63C`。另复核source owner、capability snapshot、visibility gate与pane projection直接链7个文件、1,832行。3/3通过`rustfmt --check`；当前文件含其他会话未提交的targeted shell-content实现，本轮没有修改Rust。

当前源已有三项应保留的进展。UI Asset与Animation共享一次`current_view_instances()`，并只在对应kind可见时构建；Runtime Diagnostics、Module/Plugins和Build/Export均按当前model可见性裁剪；单个shell-content定向重算只查询target instance对应的template document source，不再调用全source入口。插件source handle在`EditorShell`短锁内取得，真正的foreign `source.snapshot()`在锁释放后调用，因此旧报告中的“callback持shell锁”不成立。

## P0结构问题：full recompute仍调用全部enabled插件source

完整重算仍无条件调用`ui_template_pane_data_snapshots()`。该入口在shell锁内把C个enabled capability重新收集为`CapabilitySet`，遍历S个registered source并按required capabilities筛选，把每个template-id String和source Arc复制到新`BTreeMap`；锁外再同步执行恰好S次foreign `snapshot()`并构造第二个result map。下游只按可见pane的`pane_template.body.document_id`查询结果，因此若本帧只有V个可见template document，`S-V`个callback、payload build和map entry全部无消费者。

源码规模为capability构造`O(C log C)`、source筛选约`O(sum(required capabilities * log C))`、source/result map物化`O(S log S)`，外加S个不受限制的foreign callback wall。`EditorUiTemplatePaneDataSource`合同只有`fn snapshot() -> value`，没有owner/data generation、NotModified、affinity、estimated bytes、deadline、cancel或last-good语义；payload内部又允许任意大小的values `BTreeMap`和component-patch `Vec`。因此隐藏或稳定source仍能在UI线程随每次Full重算重复分配，慢source可直接拉长F4帧。

这不是当前某个内建插件的实测热点：tracked Rust源码没有该trait的具体实现，当前源也没有可运行产品binary。它是外部/后续插件注册后立即可达的容量与调度缺陷。报告不填写虚构毫秒、功耗或收益；必须以0/1/100/1k source fixture和真实F4插件面板补动态证据。

## Unreal依据与目标结构

Unreal `TabManager.cpp:1711-1724`先查找并复用live tab；`1766-1823`只有tab不存在且实际invoke/restore时才进入spawn；`2630-2647`再检查spawner并只在真实spawn时执行`OnSpawnTab`。它不会因为某个插件注册了tab spawner，就在每次宿主更新时调用所有spawner。Slate `SlateInvalidationRoot.cpp:299-329,1281-1395`进一步以稳定widget proxy和unique dirty lists定向处理变化，只有明确slow path才整体清空。

Zircon应由EditorUI08从Workbench model/chrome generation发布唯一visible template document demand index，覆盖active document、visible tool-window active tab和每个floating/native active tab。Editor12/Plugins01在contribution snapshot中发布`{template id, owner generation, data generation, affinity, estimated bytes}` source handle，并提供batch resolve visible IDs的短锁API；stable generation直接返回NotModified。不能从consumer逐ID重复获取shell锁，也不能先调用全部source再过滤结果。

只有声明non-main affinity的pure source可作为Runtime11共享bounded single-flight ticket执行；main-affine source必须在UI deadline内定向调用。两者都要按source generation做stale completion丢弃、reload/unload取消、entry+bytes+age硬界和last-good保留，不建立EditorUI私有pool或第二份plugin registry。

## 验收

矩阵：sources 0/1/100/1k，visible V 0/1/16/S，dirty 0/1/100%，capabilities 0/16/1k，payload 0/64KiB/2MiB，callback 0/1/16ms/10s，full/targeted recompute 1/1k，main/floating/native panes 0/1/16/1k，reload/unload/stale completion，threads 1/16。记录source enumeration、ID/Arc clone、callbacks、payload bytes、NotModified、queue entries/bytes/age、shell lock wait/hold、callback-in-lock、UI p50/p95、RSS与energy。

硬门：hidden callback=0；stable visible callback/payload build=0；changed source每generation publish不超过1；callback-in-shell/registry-lock=0；full collector source work按V而不是S增长；同一payload artifact在main/native间共享；queue/last-good内存硬有界；reload/unload无stale apply。Cargo、fixture、F4 WPR/xperf/Tracy/energy完成前保留在`pending.md`，此CPU/plugin slice不单独使用RenderDoc；最终viewport像素回归随PERF-MVP-106/626的F4 render-owner gate执行。
