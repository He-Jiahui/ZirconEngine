# Runtime213: Visibility / GPU Scene / Culling / Batching / Instancing / HZB / Virtual Geometry 当前工作树复核

- 复核日期：2026-09-01
- 复核 HEAD：`f31fd06f69fdaedb70a0a56fe6d0268de1af83a6`
- 复核类型：review-only；未修改 Rust、Cargo、ABI、tests、shader 或产品 UI，也未运行 Cargo、真实场景、GPU capture、soak 或 benchmark。
- 参考前账：[Runtime09B](09b-renderer-visibility-gpu-scene-review.md)、[Runtime94](94-runtime-visibility-review.md)、Runtime89/90/91/92/93、Plugins17。
- 当前约束：MVP 仍未通过；本报告只定义基础收口顺序，不授权 advanced renderer 或 Virtual Geometry 抢跑实施。
- Tooling：按用户要求排除；本轮也未查询、轮询、等待或实时跟踪协调器。

## 1. 冻结范围与证据

本轮对下列集合执行逐文件目录、符号、call-site、test-marker 与负消费者扫描，并对 authority、shader、提交链和产品 consumer 逐行复核：

- Runtime：`core/framework/render/scene_extract{.rs,/**}`、`mesh/bounds.rs`、`graphics/visibility/**`、`graphics/scene/{render_scene,gpu_scene}/**`；
- Renderer：HZB、mesh build/pass、history、compiled graph HZB binding/readback、shadow consumer；
- 产品入口：`graphics/runtime/render_framework/submit_frame_extract/**` 与 focused visibility tests；
- Virtual Geometry：`graphics/virtual_geometry_runtime_provider/**` 与 `zircon_plugins/virtual_geometry/runtime/src/**` 全树；
- 参考：Unreal GPUScene/InstanceCulling/SceneVisibility/Nanite，Unity Graphics GPUResidentDrawer/InstanceCuller，Bevy GPU preprocess/meshlet，Godot scene cull，Fyrox visibility。

| 范围 | files | lines | non-empty | bytes | tests | ignored | unsafe | HEAD/index/dirty | current-tree fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| Runtime core authority/consumer/shader | **332** | **66,883** | **61,344** | **2,444,083** | **703** | **39** | **1** | **230 / 330 / 260** | `84023cd283ae126f189dc2bc9224bab299a277324c09d5fe9c78d097035424d1` |
| Virtual Geometry provider/plugin | **272** | **45,377** | **42,262** | **1,666,156** | **301** | **33** | **0** | **233 / 272 / 100** | `29686dde80f55e82517f881d0b609ab7c2d36ea5c6ca19d1c21a806b7eaddb2f` |
| 五引擎参考切片 | **19** | **37,019** | **31,709** | **1,578,505** | **1** | **0** | **15 lexical** | n/a | `3af3e58471fcfca5331f841e25bde01fae180a2495e37f99233dedccc6688106` |

参考 revision：Bevy `fb89a864...`、Fyrox `8d815db3...`、Godot `8c7e6c58...`、Unity Graphics `a7e4c051...`；Unreal 镜像为 `Build.version` 6.0.0/UE5/changelist 0，以参考 aggregate fingerprint 冻结。

当前核心集合 260/332 dirty，且 VG 也有 100 个 dirty 文件。本文是共享 working tree 快照，不是 clean-HEAD 验收收据；实施前必须重算 fingerprint，并重验 bounds、journal consumer、mesh prepare 次序、HZB shader/graph binding 与 VG executor。

## 2. 总结论

Zircon 已经有真实底座：`FrameVisibility` 保留 main/custom/cascade/point-face/spot 的精确 view key；GPU Scene 有 dirty-range、staging、current/previous transform、共享双缓冲 skin palette arena；HZB 有真实 WGPU compute、indirect args/count/remap compaction、持久 workspace、bounded diagnostic readback；scene history 已按 domain 处理 camera cut、结构变化、分配变化和 feature disable。不能再沿用 Runtime94 的“HZB bounds 必然二次变换”“没有真实 GPU indirect”“history 只有白纹理兜底”等过时描述。

但产品仍不是 persistent/GPU-driven renderer。`VisibilityContext` 每帧重建多组 `Vec/BTreeMap/BTreeSet/HashMap`；新 `RenderScene`、component projector 与 `GpuSceneJournalConsumer` 没有产品 caller。CPU visibility 仍以 transform translation 加 scale-length 单位球作为 mesh bounds，而 GPU HZB 使用资源解析后的 local bounds，两套真值已经分叉。mesh build 在 visibility 过滤前完成完整 material/model/deformation/VG/morph/GPU Scene prepare；产品每个 pending draw 仍 `register(..., 1)`，instancing/upload plans 无消费者。HZB 只是对 CPU 已完整构造的若干 phase args 做 single previous-HZB compaction，结果不回写 `FrameVisibility`。

Virtual Geometry 的问题更明确：五个注册的 render-pass executor 只调用 `validate_context`；node/cluster cull、hardware raster、VisBuffer64 的现存“execute”主要在 CPU 构造记录和 buffer，production graph 没有 dispatch/draw/raster authority。`lib.rs` 仍声明固定 `[1,1,1]` workload，模块还明示 moved renderer snapshots “stay unwired”。这不能称为 Nanite 等级实现。

本轮不新增 canonical P0；Runtime09B 继续唯一拥有 7 项 P0。本轮登记 **60 项 P1：49 Open / 11 Partial / 0 Closed**，**12 项 P2：12 Open**，32 道资格门为 **25 Fail / 5 Partial / 2 Pass**。在固定画质、固定可见物、固定 LOD/shadow/material/resolution 的同机证据闭合前，不得宣称达到或优于 Unreal。

## 3. Runtime09B / Runtime94 纠偏

| 旧 P0 owner | 当前状态 | 当前工作树证据 | 仍需完成 |
|---|---|---|---|
| 09B-P0-1 无 persistent render scene | **Partial** | `render_scene`、change journal、component projector、journal cursor 与 GPU journal consumer 已存在 | 它们没有产品 caller；必须替换 per-frame extract 重建和旧 GPU Scene live-set sync |
| 09B-P0-2 spatial prefilter/多 view 全扫描 | **Open** | 单层 grid、overflow、COW map、10K 阈值仍在；每个 extra view 重跑完整 candidate Vec | hierarchical index、dirty refit、ViewFamily shared traversal 与 dense per-view mask |
| 09B-P0-3 HZB bounds 空间错误 | **Partial** | GPU 现保存 local center/radius，shader 只应用一次 world transform，旧“双变换”已修正 | CPU visibility 仍用伪 world sphere；缺同 primitive 的 extract/CPU/GPU/HZB parity 与真实非原点 GPU test |
| 09B-P0-4 instancing/GPU-driven plan 未消费 | **Partial** | 真实 GPU indirect compaction/count/replay 已存在 | instancing/upload plans 仍无 consumer；产品仍 one pending draw/one GPU instance，CPU 预建完整 args |
| 09B-P0-5 HZB truth publication 不完整 | **Partial** | domain history invalidation、fallback、GPU stats/readback 已加强 | 仍 single previous-HZB pass；occlusion 不发布到 FrameVisibility/streaming/picking/VG |
| 09B-P0-6 VG 名称高估 authority | **Open** | pass descriptor/executor/CPU records 很丰富 | executor 不编码 GPU 工作；hardware raster/VisBuffer 只造 buffer/records，需真实 cull/raster/depth/feedback |
| 09B-P0-7 visibility 前高成本 prepare | **Open** | `build_mesh_draws` 先 collect/select/possibly recollect/VG/morph/light/GPU Scene sync，后取 visibility state | visibility/residency demand 必须前移，昂贵 prepare 只服务最终需求 |

Runtime94 的 48 项 P1 没有丢失：R94-01..07 对应 RT213-001..007，08..19 对应 008..016，20..25 对应 017..022，26..34 对应 023..032，35..40 对应 033..040，41..48 对应 041..048；RT213-049..060 补上当前 VG 产品 authority 与 qualification 证据。

## 4. 当前产品链与断裂点

```text
RenderFrameExtract
  -> VisibilityContext 每帧全量重建
     -> CPU 伪 sphere bounds
     -> static/dynamic uniform grid + overflow
     -> main/custom/shadow 逐 view CPU frustum
     -> FrameVisibility + 多个无产品 consumer 的计划
  -> mesh build 遍历全部 phase meshes
     -> load/material/model/skin/morph/VG/pending draw
     -> 每 draw register(instance_count=1) + 全 live-set retain
     -> 此后才做 visibility/cache pruning
  -> CPU 建完整 command/indirect args/metadata
  -> previous HZB 单阶段 GPU compaction
  -> indirect replay；occlusion truth 不回写上游
```

目标链必须是：

```text
Scene/asset/transform mutation
  -> RenderSceneChangeJournal transaction
  -> persistent RenderSceneGeneration + canonical bounds generation
  -> dirty spatial refit + dirty GPU Scene scatter
  -> ViewFamily candidate / relevance / LOD / previous-HZB early cull
  -> visibility/residency-driven prepare + shared draw packets/instance spans
  -> depth/occluder update -> current-HZB late retest
  -> per-view final visible/indirect truth
  -> RenderGraph packet -> submission ticket
  -> completion-qualified history publish and retirement
```

## 5. P1：Scene、Visibility 与 Policy

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RT213-P1-001 | Partial | `RenderScene`/journal/projector 是完整度较高的新底座，但仅在自身模块和 tests 内消费 | 产品唯一 `RenderSceneService`；scene mutation 进入 journal，renderer 只读 sealed generation |
| RT213-P1-002 | Open | `VisibilityContext::from_extract...` 每帧重建 renderable/mobility/relevance/batch/bounds/history/index/plans | persistent primitive/view registry + add/remove/change transaction；steady frame 不重建 scene state |
| RT213-P1-003 | Open | `FrameVisibility.relevance_generation` 固定为 0，结果没有 scene/bounds/view generation | `VisibilityReceipt { scene,bounds,view_family,policy,history generation }`，跨代消费 typed reject |
| RT213-P1-004 | Open | `instance_upload_plan`、`particle_upload_plan`、`gpu_instancing_candidates`、context draw commands 主要只被 tests 读取 | 删除假合同或接入唯一 executor；capability 必须按产品 consumer 证明 |
| RT213-P1-005 | Open | `_hybrid_global_illumination` 参数被忽略，active/update/feedback/request 全部恒空 | HGI owner 发布 generation-qualified visibility bridge；未实现时 admission 明确 unavailable |
| RT213-P1-006 | Open | CPU FrameVisibility、GPU HZB compacted args、streaming/picking/shadow/VG 各持不同阶段真值 | per-view early/late/final visibility state machine 与唯一发布 receipt |
| RT213-P1-007 | Partial | RenderStats、HZB drop/readback、backend fallback 已有局部 telemetry | device/profile/view-family requested/effective policy、统一预算、降级原因和自动 regression gate |

## 6. P1：Bounds、Spatial Index 与 View Family

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RT213-P1-008 | Open | `RenderMeshSnapshot` 不携带 resource local bounds、bounds handle 或 generation | extract 传 `BoundsHandle + BoundsGeneration + kind`，与 mesh/deformation generation 对齐 |
| RT213-P1-009 | Open | CPU `mesh_bounds` 只取 translation 与 `scale.abs().length()*0.5` | 消费 canonical local AABB/sphere，经完整 affine 变换得到 world bounds |
| RT213-P1-010 | Partial | GPU 路径已保留 off-center local bounds并只变换一次 | 与 CPU/streaming/shadow 统一 owner；无效/动态 bounds 的 fail-open 也必须带 reason/generation |
| RT213-P1-011 | Open | skin/morph/cloth/VFX 没有统一 deformed/predicted bounds provider | bind/deformed/motion envelope 与 CPU/GPU reduction policy，按 LOD/quality 发布 generation |
| RT213-P1-012 | Open | static/dynamic 都是单层 uniform grid，不是 hierarchy；名称仍叫 BVH | large-world multi-level BVH/spatial hash，static build 与 dynamic refit 分离 |
| RT213-P1-013 | Open | `Arc<BTreeMap/BTreeSet>` 配合 `Arc::make_mut`，快照存活时可复制整图 | chunk/page COW 或 epoch snapshot，dirty 成本随 changed primitive 增长 |
| RT213-P1-014 | Open | update plan 构造 previous/current 全量 map，incremental path 仍扫描完整 instances | journal 直接携带 changed handle/bounds；禁止通过全表 diff 发现变化 |
| RT213-P1-015 | Open | oversized primitive 进全局 overflow；cell 过多时 fallback 为全部 entries | oversized hierarchy、bounded fallback、top offender/bytes/count 与 policy receipt |
| RT213-P1-016 | Open | query 生成 cell Vec、HashSet/BTreeSet 和 visible-entry HashMap；`visited_node_count` 实为 cell 数 | arena scratch、visited generation/bitset、dense spans；stats 使用真实算法术语 |
| RT213-P1-017 | Open | custom/cascade/point/spot 对同一 candidate Vec 重跑完整 sphere frustum | ViewFamily shared broadphase + SIMD/job/GPU view mask 与 compact per-view ranges |
| RT213-P1-018 | Open | `directional_shadow_ranges(None)` 仍 truncate 到 1，关闭阴影也创建 shadow visibility view | shadow disabled 生成 0 view；mixed-light 产品回归 |
| RT213-P1-019 | Partial | 精确 custom/cascade/point-face/spot key 已保留，旧“view identity 丢失”应关闭 | stable View/ViewFamily generation、parent/child、history identity、XR/cube/editor grouping |
| RT213-P1-020 | Open | `views.iter().find`、visible projection与 shadow union 会重复 Vec/BTreeSet 构造 | view handle 索引 + immutable dense ranges/bitsets；消费方不得临时重投影 |
| RT213-P1-021 | Open | mesh build 仍把所有 shadow view union 成单个 bool 参与共享决策 | per-view demand/mask 是 submission truth；union 只能是保守共享工作提示 |
| RT213-P1-022 | Open | 没有 light-specific layer/caster/receiver mask 与 ViewFamily contract | 同一 shadow descriptor 拥有 layer/channel/cascade/history/culling policy |

## 7. P1：GPU Scene、Memory 与 Instance Lifecycle

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RT213-P1-023 | Partial | change journal、component projector、generation 检查与 `GpuSceneJournalConsumer` 已存在 | 把该 consumer 接入产品 renderer，并删除旧 key-map 双 authority |
| RT213-P1-024 | Open | 产品 `GpuScene` 仍由 pending draws 逐帧注册/retain，不消费 RenderScene journal | `GpuSceneService` 只消费 sealed journal delta 与 generation-ready payload |
| RT213-P1-025 | Open | product entry/span 对外没有 generation-bearing primitive/instance handle | `PrimitiveHandle/InstanceHandle { index,generation }`，remove/reuse/stale GPU 引用可证明拒绝 |
| RT213-P1-026 | Open | 每个 pending draw 调用 `register(..., 1)`；instancing candidate 没接入 | compatibility key 编译 shared draw packet 与 dense instance span，常规 `instance_count > 1` |
| RT213-P1-027 | Open | `retain_registered_keys` 扫 entries，并对多组 skin/source/morph map 做全表 retain | journal removal queue + component arena tombstone/retirement，不以临时 frame live-set发现死亡 |
| RT213-P1-028 | Partial | per-instance palette buffer 已收敛为共享 current/previous 双 arena | 当前仍每帧清空、重排并上传全部 staged matrices；改为 stable span + dirty palette ranges + fence retirement |
| RT213-P1-029 | Open | morph/VG payload 用完整 Vec 相等比较发现变化 | producer version/dirty rows/ranges；GPU upload 只比较 generation/dirty metadata |
| RT213-P1-030 | Open | scene mega bind group 耦合 primitive/instance/light/skin/remap/morph/VG，任一增长可广泛重建 | 稳定 global tables、feature arenas、per-pass table 与最小 descriptor generation |
| RT213-P1-031 | Open | `material_payload_slot` 仍固定 `GPU_SCENE_INVALID_PAYLOAD_SLOT` | generation-qualified material table/parameter-block handle，供 GPU preprocess/binning 读取 |
| RT213-P1-032 | Open | grow-only buffer/span 缺总预算、fragmentation、relocation、shrink 与 completion-qualified reclaim | arena budget/high-water/fragmentation telemetry、relocation indirection与 submission fence retirement |

## 8. P1：Preparation、Batching 与 Submission

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RT213-P1-033 | Open | material/model/load/deformation/VG/morph/light/GPU Scene sync 全在 visibility state 之前 | retained cheap update -> visibility/residency demand -> async prepare -> final packet |
| RT213-P1-034 | Open | material override 可能让 `collect_pending_draws` 对完整 mesh 集合执行第二次 | material generation admission 在 retained packet 上完成，禁止整帧重复展开 |
| RT213-P1-035 | Open | GPU Scene sync 遍历全部 pending draws，之后才做 cache/visibility pruning | sync 输入来自 persistent dirty journal与final residency，不来自完整待绘制 Vec |
| RT213-P1-036 | Partial | static command cache 能真实跳过/visibility-prune command rebuild | 它发生太晚，未避免前面的 load/material/deformation/GPU Scene CPU 成本 |
| RT213-P1-037 | Open | CPU 先造完整 command、indirect args、metadata、candidate，GPU 仅末端筛除 | GPU preprocess 从 persistent records 做 relevance/LOD/bin/count/args，CPU只提交 bounded descriptor |
| RT213-P1-038 | Open | indirect batcher 只合并相邻兼容 command | stable draw key + radix/GPU binning；透明顺序与不可重排约束显式 |
| RT213-P1-039 | Open | depth/shadow/base/alpha/PBR/transparent/velocity 重复 phase object/validation | shared draw packet + phase mask/permutation metadata |
| RT213-P1-040 | Open | skinned/VG/custom bind group/existing indirect 等分支退出通用 batcher | typed compatibility matrix、fallback reason 与统一 instance/indirect ABI |

## 9. P1：HZB、Occlusion 与 History

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RT213-P1-041 | Partial | history 已按 domain 处理 camera cut、结构/分配变化、feature disable，并在 graph binding 使用 availability | 补 view identity/occluder generation/warmup/resolution migration；failed submit 不发布 history |
| RT213-P1-042 | Partial | WGPU compute 会写 compacted args/count/remap/stats，真实进入 indirect replay | 仍不是完整 GPU preprocess，也不发布 final per-view visibility truth |
| RT213-P1-043 | Open | 一个 invocation 对一个 arg，再串行遍历该 arg 全部 instances | parallel instance cull + prefix/scan/compaction；大 instance span 不由单线程循环 |
| RT213-P1-044 | Open | 仅用 sphere、`world_radius/clip.w` 与中心 UV 单 texel | conservative screen rect、mip、多 sample/furthest rule、near-plane/behind-camera CPU oracle |
| RT213-P1-045 | Open | candidate 仅覆盖部分 opaque/alpha/advanced opaque/velocity phase | depth/shadow/opaque/alpha/velocity/transparent capability/policy matrix |
| RT213-P1-046 | Open | 产品只有 previous-HZB 单阶段；`TwoPhaseRetest` 没 executor | previous early -> occluder/depth -> current late -> final publish 状态机 |
| RT213-P1-047 | Open | builder 报每 pass 最多 4 mip，执行却每 mip copy params、建 bind group、开 compute pass | plan/execution 一致；persistent bindings/params 或真正 multi-mip dispatch |
| RT213-P1-048 | Partial | 有真实 offscreen WGPU cull test、bounded stats readback、drop accounting | GPU test 仍只用原点 local center/单位 transform；缺 off-center/nonuniform/negative/near/jitter/cut/resize/multiview |

## 10. P1：Virtual Geometry 产品 Authority

| ID | 状态 | 当前差异 | 必须重构为 |
|---|---|---|---|
| RT213-P1-049 | Partial | provider/residency/page request/feedback/CPU reference 数据结构与测试较丰富 | 保留为 oracle/streaming 基础，但所有输出必须绑定真实 GPU execution generation |
| RT213-P1-050 | Open | 五个 graph executor 只 `validate_context`，没有 encode/dispatch/draw/copy | executor 必须取得 typed resources、编码真实 GPU pass、返回 submission/readback receipt |
| RT213-P1-051 | Open | node-cluster cull descriptor 固定 workload `[1,1,1]` | dispatch 来自实际 frontier/queue counts 与 device limits，zero-work/overflow 有 typed policy |
| RT213-P1-052 | Open | node/cluster cull “execute”在 CPU 最多跑 8 waves、构造 BTreeSet/Vec 后创 buffer | GPU hierarchy queues、persistent work buffers、indirect dispatch、overflow/late pass |
| RT213-P1-053 | Open | hardware raster pass 只收集/pack records并创建 buffer，没有 render pass/draw | 真正 cluster raster、depth/coverage/write contract、hardware/software path parity |
| RT213-P1-054 | Open | VisBuffer64 pass 只打包 entries到 buffer，没有 VisBuffer attachment/raster | 64-bit visibility target、clear/raster/resolve/material decode 与 depth ownership |
| RT213-P1-055 | Open | `collect_virtual_geometry_indirect_stats` 调用 CPU named passes，但没有 production caller | 删除伪 render authority或接到唯一产品 graph；stats 必须来自实际执行 |
| RT213-P1-056 | Open | 模块明示 broad moved snapshots “stay unwired” | 完成 plugin-local neutral ABI hard cutover，删除旧 runtime-owner snapshots/双写 |
| RT213-P1-057 | Open | runtime prepare 只把已准备 page-request sideband建 buffer并返回默认 renderer outputs | GPU feedback/readback 由真实 pass 产生并经 ticket/completion进入 residency transaction |
| RT213-P1-058 | Open | automatic extract/visibility frontier仍逐帧处理全 mesh snapshot 与 CPU cluster list | persistent VG instance/cluster/page handles、dirty journal、GPU visible frontier |
| RT213-P1-059 | Open | 大量 allocation/performance test 被 `#[ignore]`，无产品 100K cluster/page churn基线 | 固定资产 hash、warmup/steady/soak、VRAM/RSS/upload/readback/queue overflow门 |
| RT213-P1-060 | Open | 没有与 Nanite 同画质、同几何误差、同 shadow/material/resolution 的公平比较 | visual parity 先行，再比较 CPU/GPU/frame/memory/streaming；结果绑定 BuildSet |

## 11. P2 长期能力

| ID | 状态 | 能力 | 前置条件 |
|---|---|---|---|
| RT213-P2-001 | Open | hardware occlusion query + HZB hybrid | canonical bounds、per-view history、async query lifecycle |
| RT213-P2-002 | Open | deterministic software occlusion/server visibility | fixed oracle、world bounds、bounded raster |
| RT213-P2-003 | Open | portal/room/sector/visibility cells | persistent scene与hierarchical spatial owner |
| RT213-P2-004 | Open | large-world multi-level spatial streaming | Runtime23/29、origin/partition generation |
| RT213-P2-005 | Open | GPU Scene online defrag/page relocation | generational handles、indirection、fence retirement |
| RT213-P2-006 | Open | stereo/multiview/foveated shared culling | ViewFamily、per-view history、view-mask indirect |
| RT213-P2-007 | Open | meshlet/cluster hierarchy与software raster | canonical mesh/VG artifact 与 M1-M7 |
| RT213-P2-008 | Open | GPU draw sorting/state binning/command compression | stable material/geometry tables、transparent constraints |
| RT213-P2-009 | Open | predictive occlusion/motion confidence/hysteresis | valid velocity/history generation与false-negative telemetry |
| RT213-P2-010 | Open | async-compute preprocess/HZB overlap | RenderGraph/RHI queue/barrier/timestamp evidence |
| RT213-P2-011 | Open | scene/view/HZB/indirect capture与offline replay | stable trace identity、artifact schema、budget/privacy |
| RT213-P2-012 | Open | adaptive CPU/GPU broadphase selection | 完整 telemetry、quality policy、可复现实验基线 |

## 12. 五引擎对照

| 参考 | 工程级 owner/产品链 | Zircon 当前差异 |
|---|---|---|
| Unreal | persistent GPUScene、dirty scatter、instance culling manager/load balancer、two-phase compaction；Nanite 真正 cull/raster/VisBuffer | Zircon journal未接产品，CPU预建全 args，VG executor无 GPU 工作 |
| Unity Graphics | GPUResidentDrawer/InstanceDataSystem 管 renderer add/update/delete、AABB/LOD/current-previous transform；occlusion second-pass retest | Zircon product handle无 generation，one-draw/one-instance，single HZB pass |
| Bevy | normal mesh GPU preprocess 有 early/late occlusion、persistent work buffers；meshlet用 BVH/cluster GPU queues | Zircon per-frame CPU context重建且无 late retest；VG CPU records未接 graph |
| Godot | Scenario 拥有 instance arrays、local/transformed/previous AABB、dirty list与 DynamicBVH | Zircon extract不传真实 bounds，spatial update靠全量 diff/uniform grid |
| Fyrox | 简单但正式的 world bounds、frustum/octree/query/occlusion cache与 fail-open | Zircon基础 query仍由 renderer临时集合拼装；Fyrox只作下限旁证，不作性能上限 |

参考源码只能用于设计与验证路由，不能用于宣称性能。尤其 Bevy meshlet 也存在生命周期 TODO，Unity/Unreal 也有 CPU staging 与 fallback 成本；Zircon 必须用自身可复现 product receipt 证明取舍。

## 13. 目标 Owner 与重构顺序

| 目标 owner | 唯一职责 |
|---|---|
| `RenderSceneService` | primitive/instance registry、change transaction、sealed generation |
| `RenderBoundsService` | local/world/deformed/motion bounds 与 CPU/GPU ABI |
| `SpatialSceneIndex` | static build、dynamic refit、snapshot epoch、query/budget |
| `ViewFamilyService` | stable view/family identity、history、mask与shared traversal |
| `GpuSceneService` | generational handles、component arenas、dirty scatter、retirement |
| `VisibilityPipeline` | relevance/frustum/LOD/early-late occlusion/per-view receipt |
| `MeshDrawPacketCompiler` | generation-ready shared packet、instance span、phase mask |
| `GpuSubmissionPlanner` | GPU preprocess/bin/count/args/remap；不直接提交 queue |
| `VirtualGeometryRenderer` | GPU hierarchy cull、page feedback、raster、VisBuffer与resolve |
| `VisibilityHistoryService` | completion-qualified transform/view/HZB/LOD history publish |

依赖顺序：

1. **M0 Characterization**：冻结现有 CPU/GPU visible result、fallback matrix、bounds divergence与 product timings。
2. **M1 RenderScene hard cut**：将 journal/projector 接入产品，删除 per-frame registry authority。
3. **M2 Bounds hard cut**：资源 bounds 进入 extract、CPU spatial、GPU Scene、HZB、shadow、streaming 同一 ABI。
4. **M3 Spatial/ViewFamily**：hierarchical dirty refit、dense per-view result、零无效 shadow view。
5. **M4 GPU Scene lifecycle**：generational handles、typed arenas、dirty scatter、budget与 fence retirement。
6. **M5 Visibility-first packets**：true instance spans、shared phase packet，不可见对象不预付昂贵 prepare。
7. **M6 GPU preprocess/HZB**：previous early、current late、per-view final publish与 failure-safe history。
8. **M7 Virtual Geometry**：真实 GPU cull/raster/VisBuffer/feedback，删除 CPU 名义 executor 与双 authority。
9. **M8 Qualification**：100K、多 view/shadow、churn、OOM/device loss/soak/capture 与 Unreal 公平对照。

M1/M2 未闭合前禁止直接重写更复杂 culling shader；M1-M6 未闭合前禁止把 VG 的 CPU records 扩成更多假 pass。当前 MVP gate 未通过时只允许继续 review/characterization 与修复阻断基础合同。

## 14. 资格门

| Gate | 状态 | 必须形成的产品证据 |
|---|---|---|
| VIS213-G01 | Fail | RenderScene journal 是产品 renderer 唯一 add/remove/change 输入 |
| VIS213-G02 | Fail | steady frame 无变化时不重建 primitive/visibility registry |
| VIS213-G03 | Fail | extract/CPU/GPU/HZB/shadow/streaming 共享 canonical bounds owner |
| VIS213-G04 | Partial | GPU local-bounds单变换已成立；仍需 CPU/GPU off-center/nonuniform parity |
| VIS213-G05 | Fail | skin/morph/cloth/VFX 有 generation-qualified conservative bounds |
| VIS213-G06 | Fail | spatial update/query CPU与alloc随dirty/candidate增长而非全scene |
| VIS213-G07 | Fail | multi-view共享 broadphase且保留dense per-view final result |
| VIS213-G08 | Fail | shadow disabled light创建0个shadow visibility view |
| VIS213-G09 | Pass | custom/cascade/point-face/spot key已保持精确 identity |
| VIS213-G10 | Fail | 同 mesh/material 64实例形成共享 packet 且 `instance_count > 1` |
| VIS213-G11 | Fail | GPU Scene product sync消费journal，不消费完整pending live set |
| VIS213-G12 | Partial | journal consumer有generation检查；产品旧entry仍无generational handle |
| VIS213-G13 | Partial | primitive dirty ranges存在；palette/morph/VG仍有全量重排/比较 |
| VIS213-G14 | Fail | resolved material payload进入GPU table并有generation |
| VIS213-G15 | Fail | visibility/residency demand先于material/deformation/draw packet昂贵prepare |
| VIS213-G16 | Fail | CPU只提交bounded preprocess descriptors，不预建全部GPU args |
| VIS213-G17 | Partial | domain camera-cut/allocation/feature invalidation已接；缺view/occluder generation |
| VIS213-G18 | Fail | previous early + depth/occluder + current late retest真实执行 |
| VIS213-G19 | Fail | conservative rect/mip/multi-sample/depth规则通过CPU oracle |
| VIS213-G20 | Fail | depth/shadow/opaque/alpha/velocity/transparent policy矩阵完整 |
| VIS213-G21 | Fail | HZB plan的reduce pass数与实际dispatch/bind-group生命周期一致 |
| VIS213-G22 | Fail | HZB final occlusion发布给picking/streaming/shadow/VG统一receipt |
| VIS213-G23 | Fail | 真实GPU test覆盖off-center、nonuniform/negative、near、jitter、cut、resize、多view |
| VIS213-G24 | Pass | HZB diagnostic readback queue有固定上限与drop accounting |
| VIS213-G25 | Fail | 每个VG graph executor编码真实GPU work并返回execution receipt |
| VIS213-G26 | Fail | VG hardware/software raster真正写depth/coverage/VisBuffer |
| VIS213-G27 | Fail | page feedback来自GPU pass并在submission completion后提交residency |
| VIS213-G28 | Fail | 100K instance/cluster固定scene基线记录CPU/GPU/alloc/upload/VRAM |
| VIS213-G29 | Fail | spawn/despawn/reload/camera churn/OOM/device loss/soak无泄漏或跨代读 |
| VIS213-G30 | Partial | HZB stats/capture入口存在；缺entity到ticket的cross-stage trace |
| VIS213-G31 | Fail | 同画质同可见物同硬件与Unreal比较并绑定BuildSet |
| VIS213-G32 | Fail | capability/descriptor/pass名称不能在executor无产品work时报告可用 |

## 15. 禁止的临时实现

1. 禁止因 HZB 双变换已修复就关闭 bounds P0；CPU 与 GPU 仍是不同 owner。
2. 禁止保留无 consumer 的 plan/DTO，再以类型存在声称功能完成。
3. 禁止把 multi-draw、GPU末端筛除或 `register(...,1)` 称为 true instancing/GPU-driven。
4. 禁止扩大 fixed grid/cell limit、增加更多 `Arc::make_mut` 或全表 diff 来伪装增量空间索引。
5. 禁止只优化 static command cache，而保留 visibility 前完整 material/deformation/GPU Scene prepare。
6. 禁止把 executor contract validation、CPU record buffer 或固定 `[1,1,1]` workload 称为 VG GPU pass。
7. 禁止在 failed/cancelled/device-lost frame推进 transform/HZB/LOD/VG history。
8. 禁止用 source-string test、被 ignore 的 microbenchmark 或单一 offscreen case替代产品qualification。
9. 禁止新增兼容 facade、双写或永久 fallback 保留旧 scene/bounds/GPU Scene authority；迁移必须 hard cutover。
10. 禁止以单场景 FPS、draw count、单 vendor 或不同画质宣称优于 Unreal。

## 16. 验证边界

本轮只做静态源码、shader、call-site、负消费者和参考对照审查。没有运行 Cargo、真实 GPU/RenderDoc/Nsight、Editor/App viewport、device loss/OOM、10K/100K scale、rapid churn、长时间 soak、visual golden 或跨引擎 benchmark。现有 WGPU tests、ignored performance tests 与 diagnostics 只作为源码底座证据，不是本轮动态验收，也不能把本报告标记为 implemented。
