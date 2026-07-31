---
related_code:
  - zircon_runtime/src/graphics/visibility
tests:
  - current graphics visibility slice 61 of 61 Rust files reviewed, 4230 lines
  - all 25 tests read; seven behavior/source regressions added
  - per-view frustum precompute, shared extra-view candidates and four transient-ownership gates changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 multi-view/large-scene counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics visibility静态审查（2026-07-18）

## 当前源覆盖

`graphics/visibility/**`当前61/61个Rust文件、4,230行已逐文件静态阅读，25条测试已读：root 1/1、`declarations/**` 21/21、`culling/**` 7/7、`occlusion/**` 2/2、`planning/**` 17/17、`context/**` 10/10、`static_index/**` 1/1、`view_context/**` 2/2。覆盖main/custom/shadow view、serial/TaskPool frustum、HZB合同、batch/BVH/history/upload计划、静态空间索引、HybridGI/VG DTO与完整VG frontier/page/lineage计划。

## 直接止损

mesh frustum原对每个candidate重复构造view matrix、求half-FOV tangent与投影参数；custom target、directional cascade、point six-face及spot view又各自重建同一entities+bounds candidate Vec。本轮引入每view一个`BoundsVisibilityTest`，矩阵/投影参数只算一次；全部extra views共享一份candidate slice，无extra view时不分配，且移除custom camera额外clone。perspective/orthographic旧方程等价测试与串并行顺序测试保留语义。

batching原先先构造bounds candidate Vec，再转HashMap，主循环又查询；现唯一主循环直接计算一次bounds。显式visibility renderables原深clone整份含layer集合的Vec，但下游只消费entity+mobility；现改惰性tuple iterator，mesh fallback语义不变。BVH update plan直接借用history entries，不再为比较深复制全部key/bounds；draw instance Vec按可见batch总实体数预留。七条RED→GREEN行为/源码回归锁定这些数据流。

## 剩余根因

PERF-MVP-419负责多view culling：每个extra view仍对全部primitive做一次frustum结果Vec，且custom layer/shadow relevance过滤发生在frustum之后；`FrameVisibility::from_main_view`又建relevance BTreeMap并四遍投影SoA，visible set/query继续物化Vec/BTreeSet。1个main+4 cascade+6 point faces即可形成12×N CPU bounds测试与结果写入。Render04/05/09须发布prepared view descriptors/frustum planes，先过滤再测试，并按规模在并行CPU与GPU culling间切换。

PERF-MVP-420负责visibility scene/index owner：context仍每frame建mesh/phase maps、entries/batches/BVH/history多份key，`VisibilityContext: Clone`会深复制全部payload；最严重的是previous `VisibilityStaticIndex`每帧先深cloneentries/cells再所谓incremental update，static instances也完整clone，update还为全部instances建BTreeMap。`cells_for_bounds`按三维跨度无容量/最大cell预算，异常大bounds可放大CPU/RSS。Runtime07/Render04/06须让scene generation持久拥有immutable primitive SoA与mutable index/delta，history只持Arc handle/revision。

PERF-MVP-421负责生产VG plan：`cluster_visible`仍每cluster重算view matrix；每个visible cluster的ordinal/count会反复扫描、sort/dedup同entity cluster IDs，在visible DTO和draw segment两处重复。frontier每次split重排全frontier并clone/sort children；page request/hot/evictable存在Vec contains；requested page×candidate×ancestry与target×candidate×ancestry多重遍历，且每次ancestry walk新建BTreeSet。Render03/04与VG plugin须把entity range/ordinal/count/parent/depth/page dense index编译进asset generation，并让真实GPU/parallel cull产出唯一sealed plan/feedback artifact。

本地Bevy `bevy_camera/src/visibility/mod.rs`把Frustum作为view组件预先提供，先做继承可见性/layer/range过滤，再以parallel query cull并写view-owned visible queues。UE `NaniteCullRaster.cpp`把instance hierarchy、node/cluster cull、work args和visible clusters放入有容量上限的GPU work queues/compute阶段。采用“view数据预计算、先廉价过滤、持久索引、层级cull唯一artifact”的原则，不复制其ECS或RHI API。

## 验收状态

61/61静态阅读、七条RED→GREEN回归代码、rustfmt、source contract与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，25条测试没有current-source执行结果；RenderDoc CLI不可用且本切片无capture。primitives 0/1k/100k/1M、views 1/4/12/64、static/dynamic 0/50/100%、VG clusters/pages 0/1k/100k/1M、stable/1% change的matrix/tan、bounds visits、candidate/result bytes、maps/clones、index cells、frontier sorts、lineage visits、CPU p95/RSS、GPU dispatch/timestamp未完成，继续留在`pending.md`，不进入`review.md`。
