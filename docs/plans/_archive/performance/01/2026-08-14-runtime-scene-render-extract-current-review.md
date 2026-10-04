---
related_code:
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/level_system
  - zircon_runtime/src/scene/render_extract
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneVisibility.cpp
tests:
  - zircon_runtime/src/scene/tests/level_system_frame_state.rs
  - zircon_runtime/src/scene/tests/render_extract.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/render_extract.rs
  - zircon_runtime/src/scene/tests/render_extract
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_runtime/src/scene/tests/derived_state/runtime_freshness.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Runtime scene render-extract current-source结构审查（2026-08-14）

## 当前范围与证据身份

scene直接生产链12/12个Rust文件已按当前源码复读：3,652行、3,343个非空行、16条内联测试，fingerprint为`DB32126496F827CD1011EFC424162BCC5E3AB6874313371D5D2DF1C26BB9604F`。入口、缓存、编辑器与camera-loop 8个产品锚点共2,025行，fingerprint为`8F1323E67192F5427BDAB27735D0D94BFA046AC32E77F21E5BB5976CDACCC0D0`。13个直接测试文件共4,874行、86 tests，fingerprint为`064AA892812475738CD376D9AB28AC23157100D3DD0B920DE8A5353906D6F434`。

25个scene直接生产/测试文件`rustfmt --check --edition 2021`通过。扩展consumer锚点检查暴露两处其他会话已有格式漂移：`editor_state_render.rs`一处assert换行、`camera_loop.rs`及其test import排序；本会话没有覆盖其生产源码。当前直接源码多数为其他活动会话的modified/untracked内容，本轮只读，不把`.codex/outbox`候选当作产品实现。

## 已有进展，旧报告不可直接复用

- `LevelSystem`当前已把physics、animation、script拆成独立mutex；physics event和animation playback/pose使用`Arc` sealed snapshot，script key改为entity分区的borrowed lookup。2026-07-22报告中的“单WorldRuntimeState mutex、getter深clone三张map/event Vec、script临时String”已经过时。
- 粒子候选从全World entity降为dynamic-component owner，每owner只做一次外层lookup，emitters/bounds不再二次排序；多primitive transform revision也已移到entity级计算。
- camera loop以`Arc<RenderFrameExtract>`流式切换camera，并move VG/Hybrid-GI大sideband，避免为每camera完整clone；这些改进不能替代scene/per-view数据分层。

## P0：三套产品入口已经分叉

1. `LevelSystem: RenderExtractProducer`先取sealed animation handle，再在`with_world_mut`独占World锁中执行完整prepared extract，最后深clone被选中的pose。仓库产品caller未直接调用此trait入口，直接调用仅见测试。
2. runtime dynamic session在`RuntimeFrameExtractCache::current_extract`中先锁World取宽泛key；miss再次锁World并调用`World::to_render_frame_extract()`。该函数先clone整World，再在clone上运行RenderExtract stage。它绕过LevelSystem producer，因此既不消费level-owned animation pose，也不把animation generation写入cache key，world handle还使用默认`0`。
3. editor在shell lock内再取authoring World lock，调用legacy `build_viewport_render_packet`；该入口clone整World并重建projection。随后`RenderFrameExtract::from_snapshot`明确把animation poses、sprites、particles和advanced sideband置空。editor每帧提交因此同时承担clone成本与功能降级。

这不是可由局部循环优化解决的问题。PERF-MVP-620要求删除产品入口分叉：runtime、editor和插件/测试适配器都必须走一个generation-aware `LevelSystem`/render-world producer；legacy snapshot adapter只保留显式preview、roundtrip或synthetic用途。

## P0：World clone和稳定帧cache仍复制全量payload

`World::clone`不是便宜只读快照：它复制entities/kinds、dynamic JSON、component/type registries、schedule、resources、events/messages/observers、command/deferred state、NodeCache等，再用stable entity列表重建entity registry和component-storage projection。editor每次legacy packet都走此路径；runtime cache miss也在World锁内走同一路径。

cache hit同样不是零工作：`entry.extract.clone()`深clone完整`RenderFrameExtract`，现有diagnostic直接规定`extract.full_clones=1`及`full_clone_bytes=output_bytes`。miss又为cache entry深clone一次。key只含global change tick、lifecycle visibility revision、active camera和viewport size，任意无关World write都使全extract失效；level animation publication却不会使它失效。

PERF-MVP-349继续负责World clone=0；PERF-MVP-620负责cache返回generation handle/owned handoff、domain revision key和唯一产品producer。稳定generation的scene artifact build、World projection rebuild、full extract clone必须全为0。

## P0：scene-global与per-view工作没有分层

一次direct frame extract当前至少执行mesh列1次、sprite列1次、五类light各1次、为volumetric light再扫四类light、post-process volume列1次、camera descriptor列1次，另有dynamic-component owner扫描；camera fallback还可能再扫camera列。每个entry重复查询active、layer、world transform、mobility，mesh复制morph weights/layer set，五类light分别排序。

`build_visibility_input`在已经物化的mesh/sprite/particle上再构建renderables并排序，随后用三个`BTreeSet`分别生成all/static/dynamic entity Vec，属于额外`O(R log R)`比较、节点分配和layer clone。它只是visibility输入重包装，真正frustum/occlusion仍在graphics侧完成。

更严重的是mesh LOD、particle透明排序、light/volume筛选都用首次selected camera的位置或layer构建；camera loop之后只替换`RenderViewExtract`的camera fields，并不重新生成LOD/particle order/scene payload。因此多camera共享的不是camera-neutral scene artifact，而是已经被首相机投影过的DTO，存在非首相机LOD与透明顺序错误风险。

结构目标分两层：

- scene generation发布camera-neutral packed artifact：primitive identity/transform/bounds/material/LOD ranges、typed particle state、light data、compiled volume data、camera descriptors和dirty revisions。artifact由mutation delta更新，不按camera复制。
- 每camera submission只生成view descriptor、layer/candidate bitset、LOD choice、透明order、volume influence和visibility结果；阈值以上才并行，输出dense bitset/ranges或compact indices，不重建scene-global DTO。

PERF-MVP-419承接visibility候选/bitset/并行；PERF-MVP-363/364承接volume唯一compiled artifact与per-camera resolution；PERF-MVP-465承接typed particle artifact及per-view order；PERF-MVP-620负责scene/view边界和统一入口。

## World锁和调度边界

正式LevelSystem extract在`with_world_mut`里运行RenderExtract内部系统、全部列扫描、排序、volume/light/visibility构建和skeleton过滤。runtime dynamic cache还把World clone与prepared extract包在`with_world`内；editor上层同时持shell lock。简单把这些循环投到worker会保留全量工作，并扩大锁/同步成本。

正确顺序是：mutation owner在明确stage结束时短锁publish scene generation与dirty delta；render线程短锁或原子取得immutable artifact handle后立即释放World/shell；per-view cull/sort/resolve在锁外按规模调度。只有证明eligible work足够大后，才按view或dense primitive range分块。

## Unreal Engine本地源码依据

- `ScenePrivate.h:1561-1596`把primitive、transform、proxy、bounds、flags和visibility id保存在`FScene`致密持久数组中，以packed index定位；它不为每个view clone gameplay World。
- `RendererScene.cpp:1570-1597`把transform变更入队；`1643-1677`可检测并跳过冗余transform update；`1681-1709`批量累计或提交render command。Zircon应复制“持久scene owner + changed primitive publication”原则，不复制UE对象模型。
- `GPUScene.cpp:1821-1842`只标记persistent primitive dirty并由GPUScene update消费，支持稳定generation零全量上传。
- `SceneVisibility.cpp:3612-3708`依据primitive数和worker数确定frustum/occlusion/relevance任务粒度；`3735-3738`显式建立frustum到relevance依赖。并行发生在持久scene数据之后，而不是用worker掩盖重复extract。

## 实施次序与行为门

1. RED行为门：runtime/editor产品入口不得调用`World::to_render_frame_extract`、`World::build_viewport_render_packet`或`RenderFrameExtract::from_snapshot`；runtime animation publication必须改变可见frame generation，非首camera必须得到自身LOD/particle order/volume result。
2. 统一producer：以LevelSystem/render-world generation为唯一入口，world handle使用真实generation；legacy adapter从产品调用图硬切。
3. 发布camera-neutral scene artifact：dirty primitive/light/particle/volume/camera ranges一次更新；stable generation复用Arc handle。
4. per-view projection：layer/candidate、LOD、透明order、volume、visibility由camera identity+scene generation构键；多camera共享scene bytes而不共享首相机结果。
5. cache与ownership：cache entry和graphics submission传递Arc/generation handle，unique时move；stable hit full DTO clone=0，diagnostics改为实际copy bytes而不是接受固定1次clone。
6. 最后才并行：对eligible dense ranges建立阈值、worker count和有界scratch；小scene保留串行快路。

## 动态验收矩阵

规模：entities/meshes/dynamic owners/lights/volumes为0/1/1k/100k，mesh primitives为1/8，cameras为1/8/64，stable/selection-only/camera-only/1% changed/animation-only。记录：World clone/projection rebuild、scene artifact build/dirty rows、component visits、JSON fields、sort/comparisons、BTree/hash nodes、alloc/clone bytes、World/shell wait+hold、main/worker CPU、queue age、p50/p95/p99、RSS、CSwitch/ReadyThread、GPU draw/dispatch/upload/timestamp与energy。

硬门：产品legacy入口调用=0；World full clone bytes=0；stable scene artifact rebuild=0；stable extract full clone=0；animation-only frame不陈旧；scene-global visits不乘camera；per-view work近eligible candidates；非首camera LOD/particle order/volume/visibility正确；disabled diagnostics overhead=0。UE耗时仅能在同机同规模、同可见primitive与同render feature矩阵下作经验比值，禁止填造绝对毫秒。

RenderDoc 1.44、WPR和xperf已确认可用，Tracy profiler未安装。`target`中没有`zircon_app.exe`且无产品进程；之前受管Windows build仍被其他会话的`zircon_runtime`编译错误阻塞。因此本轮不能生成可信WPR/energy/RenderDoc数据，不执行空capture，也不进入`review.md`。
