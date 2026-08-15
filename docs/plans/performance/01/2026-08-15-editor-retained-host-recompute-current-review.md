---
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute
  - zircon_editor/src/ui/retained_host/app/invalidation
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/invalidation_bridge
  - zircon_editor/src/ui/retained_host/ui/scoped_presentation.rs
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
tests:
  - recompute current slice 13 of 13 Rust files reviewed, 1608 lines, 20 tests
  - direct invalidation, dirty bridge, presentation state and scoped patch chain 13 files rechecked, 1632 lines, 21 tests
  - render-only contract static RED then GREEN; changed files rustfmt and diff check clean
  - recompute rustfmt 12 of 13 clean; foreign-dirty presentation.rs import ordering remains
  - managed focused test stopped at coordinator session.register timeout, 0 tests executed
  - current-source F4, WPR, Tracy, energy and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Editor retained-host recompute current-source结构审查（2026-08-15）

## 当前范围

`host_lifecycle/recompute.rs`和`recompute/**`当前13/13个Rust文件为1,608行、1,524个非空行、20条测试，path+content fingerprint为`C10E36EB87B3AEC5AE35B4ECD4BA27562785A21D7F03B78B6A79D5DF4385301F`。另逐文件复核失效根、dirty bridge、host presentation state和scoped patch直接链13个文件、1,632行、21 tests；合计26个文件、3,240行、41 tests。当前文件含其他会话未提交改动，本轮只在取得协调器精确写锁后修改`mask/requirements.rs`和`root/tests.rs`中的render-only边界。

当前源已经出现需要保留的结构进展：失效事务按global/view/shell-content scope合并；Workbench projection、单组view presentation、单个shell-content和window metrics均有定向路径；shell-content与window metrics复用committed shell；正常viewport提交只在runtime diagnostics可见时刷新presentation；native store已有applied generation cursor。这些变化否定了2026-07-30报告中“所有成功submit无条件反向置presentation dirty”和“所有非paint原因都必然全重建”的旧结论。

## 已直接修复：render-only被错误升级为完整Host重建

修复前`HostInvalidationMask::requires_host_recompute()`把`RENDER`包含在内。`HostInvalidationRoot::invalidate_scoped`因此把纯render请求写入recompute transaction；`tick.rs:53-54`先调用`recompute_if_dirty()`、后调用`submit_render_frame_if_dirty()`，而decision没有render target，于是render-only稳定落入`Full`。一次事件静态触发1次shell snapshot、1次floating projection、全部pane payload、main presentation、全部native presenters、viewport/pointer surfaces和committed-state clone，然后才进行本来需要的1次viewport submit。hierarchy rename有3个直接render-only入口，`EditorEventEffect::RenderChanged`与asset refresh还会产生同类请求，因此此路径产品可达。

渲染需求具有独立所有权：dirty bridge会设置`render_dirty=true`，失效根单独累计`render_requests`；render submission即使没有pending `RENDER`也以`HostInvalidationMask::RENDER`作为原因兜底。修复据此只从`requires_host_recompute()`删除`requires_render()`，并把测试改为：render-only仍`requires_render`、仍计数，但不进入Host recompute transaction。静态合同先以“生产函数仍含`requires_render`”得到RED，再在一行实现后GREEN；两处变更通过`rustfmt --check`和scoped `git diff --check`。

量化边界是每个纯render失效从“1次完整Host重建+1次render submit”降为“0次Host重建+1次render submit”。这不是wall-time或功耗实测。托管focused test在Cargo前因协调器`session.register` post-response timeout退出，0 tests执行；当前源无可运行产品binary，因此WPR/Tracy/energy与RenderDoc均不能据此宣称改善已验收。PERF-MVP-626负责补齐动态闭环。

## 剩余P0：scoped view patch仍按view重复扫描并克隆浮窗

`presentation_only_view_ids()`先为事务分配`Vec<ViewInstanceId>`。`apply_scoped_ui_asset_presentation`随后逐view构造pane patch，主presentation探测固定dock和全部floating/native rows；floating patch即使目标不存在也clone每一行再重建ModelRc。之后`native_presenter_ids`再次扫描native rows，native store又遍历全部presenter，对命中的每个window重复同一presentation/floating扫描。测试明确锁定missing target的2行visited=2、cloned=2。

若同帧有V个view invalidation、main有F个floating row、W个native presenter且各自有Fw行，当前源码规模为`O(V * (F + W + sum(Fw)))` visits，并有同阶floating row clones；命中后的`Arc::make_mut`还可能复制整份host presentation，hit index不匹配时重建索引。这已经比full shell fallback好，但仍不是UE式indexed dirty-widget更新。

Unreal `SlateInvalidationRoot.cpp:299-329`把reason合并到稳定`FWidgetProxy`并以unique heap/list登记pre/prepass/post update；`356-423`只有root明确需要slow path时才清空cache和全树paint，否则走`PaintFastPath`；`1281-1395`按定向列表处理更新，直到确定slow path才整体reset。Zircon的下一步不应继续给presentation扫描加consumer cache，而应由EditorUI08/EditorUI01建立stable `{view instance -> main/native presenter slot, floating row, damage node}`索引和generation cursor，transaction直接携带dense slot；changed view只patch自己的immutable presentation segment，缺失/stale generation才显式降级full。

## 实施与验收

1. EditorUI08保持PERF-MVP-626的render/presentation/layout分域，给每个target记录counter；任何新增mask必须声明唯一consumer，render submission不得再次成为Host recompute原因。
2. EditorUI08/EditorUI01按PERF-MVP-106/113发布稳定view-to-presenter索引、immutable pane segment和per-window applied cursor；native presenter不得对每个view全表扫描，floating缺失探测不得clone行。
3. Editor05/Render17恢复current F4产品后运行render-only hierarchy rename、asset refresh和`RenderChanged` storm；WPR/xperf/Tracy测UI thread wall、alloc/clone、CSwitch/ReadyThread和energy，RenderDoc只验证submit次数、viewport pixels与present parity。

矩阵：views 1/16/1k，main floating rows 0/1/16/1k，native windows 0/1/16/1k，invalidations 1/1k/100k，render-only/presentation/layout/combined，diagnostics hidden/visible，stable/single change/stale target。硬门：纯render的Host recompute/full rebuild/pane payload/native apply/pointer sync均为0且render submit不丢；stable view的scan/clone/patch=0；changed view lookup近O(1)、每presenter每generation patch不超过1；missing/stale显式计数并有界fallback；Cargo、F4交互、WPR/Tracy/energy与RenderDoc parity全部完成前继续留在`pending.md`，不进入`review.md`。
