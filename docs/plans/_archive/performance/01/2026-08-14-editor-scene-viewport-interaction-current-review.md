---
related_code:
  - zircon_editor/src/scene
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/scene/viewport/interaction_extract
  - zircon_editor/src/scene/viewport/pointer
  - zircon_editor/src/ui/host/editor_scene_mode_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/render_submission.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/graphics/visibility/static_index/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/visible_spatial_query.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorModeManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/DragTool_FrustumSelect.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
tests:
  - tools.tests.test_editor05_viewport_interaction_extract_contract
  - current-source Windows Cargo, F4 interaction, WPR, Tracy and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Editor scene/viewport interaction current-source结构审查（2026-08-14）

## 当前范围与证据身份

`zircon_editor/src/scene/**`当前物理清单139/139个Rust文件：9,079行、8,200个非空行、60条内联测试，fingerprint为`6F0E90A17CC432AA0D937F54F60143FB1760B84D37669E2FC7C7F3592A412304`。模块分布为modes 18、selection 6、viewport 114、root 1；本轮将旧128文件证据与当前文件清单逐项对账，并复读当前新增owner、25个modified文件及render/pointer/plugin直接调用链。10个runtime visible-query与retained-host锚点共1,871行、22 tests，fingerprint为`FA9E4B71AC70B4F31C9956CA5677CD053E4BF95E949491B712BD907EED295431`。

Editor05静态合同5/5通过。全scene `rustfmt --check`只暴露其他会话当前modified的`interaction_extract/cache.rs`与`interaction_extract/tests.rs` import排序漂移，其余current owners未报告格式差异；本轮没有覆盖或修改这25个产品文件。当前没有`zircon_app.exe`存在于workspace `target`或`E:\ZirconBuilds`，所以不能把源码复杂度推导冒充WPR、功耗或RenderDoc产品数据。

## P0：changed generation仍被物化为两套pointer表示

render路径先构造legacy packet，再把`packet.scene.meshes`完整`to_vec()`进`ViewportInteractionExtract`。pointer在render前首次到达时会独立调用同一`build_render_packet`，因此cache miss可触发World clone、完整render extract和第二份mesh DTO；新增的`pointer_fallback_packet_build`与`interaction_mesh_copy_payload_bytes`只让成本可观测，并没有删除成本。

每个changed generation在renderer snapshot发布前还至少执行：

1. `renderable_candidates`线性访问全部mesh并物化owner/position/radius Vec；其“相邻owner去重”只在同owner primitives连续时正确发挥压缩作用。
2. `rebuild_surface`把每个renderable、gizmo和handle投影为precision candidate、UiTree node、格式化node path及`BTreeMap` row，复杂度至少为`O(M + G)`访问和有序映射插入，而renderables并不是普通UI控件。
3. 成功提交renderer snapshot后，`RendererVisibleSpatialPickSource::new`又把全部renderable clone进owner `BTreeMap`，而该`O(M log U)`构造发生在`shared` mutex内；随后layout清空renderable nodes并第二次重建surface。

因此changed generation不是“一个interaction extract”：它先为pre-submit fallback建立per-renderable UI surface，提交后再建立renderer-query owner表并拆掉这些UI nodes。PERF-MVP-221保留稳定generation零重建的已有成果；PERF-MVP-222/620必须删除changed-generation双表示和legacy fallback，而不是继续优化这些临时Vec。

## P0：点选有空间索引，但事件仍在共享锁内分配和解析

renderer-visible snapshot的静态/动态index clone已是`Arc`共享，这是有效边界；point pick也只对ray返回owner做screen projection，不再逐event全扫全部mesh。但当前dispatcher持`SharedResolutionState` mutex跨越完整`query_ray`、owner lookup、candidate projection、hit Vec、排序/hover/report resolution和结果写回。`VisibleSpatialQuery::query_ray`每次事件还物化cell Vec、candidate-key `BTreeSet`、entity去重`BTreeSet`和结果Vec。

事件算法应改为：短锁取得immutable `Arc<PointerResolutionSnapshot>`和generation，锁外用caller/reused scratch执行cell traversal、epoch/bitset去重、exact hit与picking resolution，再以generation-check短锁发布route/debug。稳定pointer event不得建立树节点或有序集合；query work应接近访问cell与真实candidate数，而不是scene总量。该责任补强PERF-MVP-332，不能以当前“已有ray query”宣称拾取热路已验收。

## P0：框选仍是全场景CPU投影

`selectable_owners_in_rect`对全部`renderable_candidates`逐项做world-to-screen与circle/rect测试，再扫描全部scene gizmo shapes；它完全不使用renderer-visible spatial snapshot。拖框结束一次为`O(M + G)`，若未来用于hover feedback则会按move频率放大。

正确结构不是把2D矩形硬塞给world AABB：为perspective view构造selection convex volume，使用同一scene spatial index先返回K个候选，再做严格/非严格screen-space exact test；orthographic view可走box/volume fast path。visible-only与transparent selection必须是显式语义：前者消费rendered visible/hit-proxy等价集合，后者消费authoring scene spatial集合，但两者都不得默认回退全World扫描。

## P0：visible-query publication仍重复构建有序集合

每次dirty render成功后，`VisibleSpatialQuery::from_context`重新从`FrameVisibility`物化main-view visible-key `BTreeSet`，再全扫`bvh_instances`并建立visible-entry `BTreeMap`。static/dynamic index本身是cheap Arc clone，但visible set/map仍为`O(V log V + N log V)`；同一visibility构造链在visible batches和VG plan处已经分别请求main-view key set。

PERF-MVP-419/222应让frame visibility直接发布generation-owned dense visible range/bitset和owner-slot presentation table，render submission、VG、pointer query共享它；snapshot publication只组合Arc handle与identity，不第三次重建visible set/map。多primitive同entity继续以`stable_instance_key`保持renderer identity，authoring owner只在最终selection结果去重。

## P0：scene-mode/plugin回调位于editor主锁域

retained host每个tick调用`runtime.update_scene_modes()`；该入口取得editor shell mutex后同步调用base与全部overlay scene mode的`update`。plugin isolation只是panic/fault boundary，不是调度或时间预算。mode可每tick要求overlay invalidation，下一次render/pointer rebuild又在shell与authoring World锁域中同步调用scene-mode `build_overlay`和所有enabled viewport overlay provider；provider拿到完整`&Scene`且没有declared dependency、affinity、deadline、bytes或last-good generation合同。

这条链可把慢plugin callback、全scene扫描、interaction cache invalidation、legacy packet fallback和pointer surface双重构建串成一次UI停顿。PERF-MVP-621要求Editor05/Plugins01/Editor12共同拆锁：短锁捕获immutable input+mode/plugin generation，锁外执行callback并产出bounded effect/overlay artifact，短锁generation-check apply；只有显式non-main affinity的pure work进入Runtime11共享bounded scheduler，禁止plugin私建线程池。active mode仍可声明continuous tick，不能为追求指标删掉合法动画/工具行为；on-input/on-scene-change模式的稳定tick callback必须为0。

## Unreal Engine本地源码依据

- `EditorViewportClient.cpp:1668-1669`与`EditorModeManager.cpp:1499-1531`确认active editor modes可以按viewport tick更新；Zircon不能无依据删除continuous mode tick，但必须把continuous demand、耗时和invalidation显式化。
- `EditorViewportClient.cpp:6012-6025`只在`bShouldCheckHitProxy`成立时读取hover hit proxy，并在处理后清标志，支持以事件/失效驱动稳定拾取，而不是每tick重建候选。
- `DragTool_FrustumSelect.cpp:145-250`明确分离transparent selection的actor/frustum路径与opaque selection的viewport rectangle hit-proxy路径；Zircon应保留这两种语义，却不应把UE透明模式的全actor循环复制成唯一算法。
- `SceneVisibility.cpp:3612-3708`按primitive数与worker数计算frustum/occlusion/relevance task粒度，并在`3735-3738`建立依赖。并行应发生在persistent scene/spatial artifact之后，不能用worker掩盖World clone和重复DTO。

## 目标算法与实施顺序

1. 先完成PERF-MVP-620的唯一camera-neutral scene artifact；它同时发布render primitive identity、bounds、owner slot、visibility generation与共享mesh/pick presentation handles。
2. pointer surface只保留handles/gizmos/UI，renderables始终走scene spatial query；pre-first-render使用CPU scene artifact，不再建legacy packet或per-renderable UiTree nodes。
3. point query使用reused cell/hit scratch；rectangle query使用view convex volume broad phase，再只对K个candidate做screen exact test。visible-only/transparent输入集合分开验收。
4. visible snapshot直接借用frame visibility和owner-slot table；publication不得重建BTreeSet/BTreeMap。pointer事件锁外query，generation-check commit。
5. scene-mode/plugin改为immutable input -> bounded effect/overlay artifact；按demand/cadence调度，stable generation复用last-good Arc，只对实际output revision失效interaction/render。
6. 最后按eligible candidate、provider work与worker数选择串行/并行阈值；小scene和轻callback保留串行快路。

## 动态验收矩阵与跨计划交接

规模：meshes/owners/gizmos/modes/providers为0/1/1k/100k，pointer events 1/1k/10k，callbacks为0/1/16ms/10s，60/120/240Hz，stable/selection/camera/1% scene change/pre-submit/post-submit，opaque/transparent rectangle。记录World/full DTO clone bytes、interaction mesh copy、renderable/UI node/owner-map builds、surface rebuild、query cells/candidates/tree alloc、shell/World/shared lock wait+hold、callback wall/affinity/queue age/invalidation、CPU p50/p95/p99、RSS、CSwitch/ReadyThread、GPU timestamp与energy。

硬门：产品pointer fallback packet=0；changed generation renderable UiTree nodes=0；mesh/pick presentation authoritative owner=1；stable artifact/surface/provider callback=0；point query无per-event BTree allocation且mutex hold不含query/score/sort；rectangle visited接近spatial candidates；foreign callback wall不计入shell/World lock；continuous mode恰好按声明cadence运行，on-demand mode稳定tick=0；stale completion不apply。Editor05承接interaction/pointer与锁域，Plugins01承接callback/overlay ABI，Editor12承接lifecycle/cancel/diagnostic，Runtime11承接共享scheduler，Runtime07/Render04承接scene artifact与spatial selection。

当前Editor05静态合同5/5仅证明旧duplicate owner删除、kind-first gizmo过滤与single cache形状；它不能证明上述复杂度门。产品binary为0，current-source Cargo仍受共享编译错误阻塞，WPR/Tracy/energy/F4与RenderDoc均无可信样本，因此本记录不进入`review.md`。
