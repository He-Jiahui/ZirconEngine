---
related_code:
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/id_allocator.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/morph.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/staged_upload.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/staging_ring.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/update_queue.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/upload.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/virtual_geometry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_plugins/04-animation.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
tests:
  - current GPUScene slice 18 of 18 Rust files reviewed, 4733 lines, 4261 nonblank lines, 44 inline tests
  - all 18 current Rust files pass rustfmt 1.8.0 check
  - upload-path threshold counterexample source gate passed
  - RenderDoc 1.44, WPR and xperf available; Tracy unavailable
  - current-source Windows Cargo, F2 counters, WPR, GPU timestamps and RenderDoc capture blocked
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics GPUScene current-source结构审查（2026-08-14）

## 当前范围与证据身份

`zircon_runtime/src/graphics/scene/gpu_scene/**`当前物理清单18/18个Rust文件：4,733行、4,261个非空行、44条内联测试，fingerprint为`93B49FDBD27262ACD386375F247A107EEA6D594183C5BFCDBA2068315E8329E8`。本轮逐文件复读旧15文件基线、新增bindless payload/staged upload/staging ring及直接consumer；18/18通过`rustfmt 1.8.0 --edition 2021 --check`。5个产品文件由其他会话修改，本轮不越权写产品代码。

必须保留的正向变化：primitive/instance/light在shadow相同时GPU写为0；dirty queue复用Vec并原地排序/合并；full upload直接discard dirty而不为废弃ranges排序；morph/VG使用grow-only capacity且稳定内容GPU写为0；morph/source history已改为`Arc`共享，不再深拷贝weight/mesh payload。新staging ring复用3个COPY_SRC buffer、CPU byte scratch和copy scratch，并把primitive/instance/light的大批次CPU数据合成一次`queue.write_buffer`。旧报告中的“morph weights深clone”和“large dirty只能逐range direct write”均已过时。

这些变化只减少最终GPU写和局部分配，没有改变GPUScene的场景更新owner。当前结构仍是render submission阶段从当前viewport的pending draws重建GPUScene输入；因此`uploaded_bytes=0`不能作为稳定帧算法已收敛的证明。

## P0：GPUScene仍由viewport draw重建驱动，而非scene delta驱动

`build_mesh_draws`每次先重新生成`pending_draws`，随后`sync_gpu_scene_pending_draws`为全部pending draw新建`live_keys: HashSet`和输出`entries: HashMap`。每项都执行stable-key register查询、skinning/morph history查询与stage、primitive/instance完整值构造和比较、transform revision写入，再插入输出map。lightmap存在时还从全部slots再建一张HashMap。

同步结束后，`retain_registered_keys`先扫描全部registered entries并分配stale-key Vec，再分别对current/previous palette、palette buffers、current/previous skinned source和current/previous morph weights共7张map执行retain。成功submit后又执行4类history roll：previous transform扫描全部entries和instance spans；palette/source/morph分别扫描removed、clear previous map并从current map完整extend。Arc只把payload深拷贝变成引用计数clone，稳定帧的全map访问、clear/insert和hash成本仍存在。

正确owner应由Runtime04/Render03发布scene generation唯一的`added/changed/removed` dense records。GPUScene保持persistent primitive/instance slot与generation，viewport/view只消费slot handle和visibility bitset，不再拥有注册/删除权。稳定scene generation的register、retain、history roll和scene-data比较访问必须为0；camera变化不得触发camera-neutral GPUScene数据重建。

## P0：skinned palette的数据模型按实例放大固定大对象

一个`SkinnedMeshJointPaletteStorage`固定内嵌256个4x4矩阵和16字节参数，即16,400字节。每个skinned stable key当前拥有两个完整GPU buffer，共32,800字节，即使实际只有1个joint；CPU侧current map、previous map和buffer committed storage在稳定提交后通常同时保留3份完整storage，约49,200字节/实例，变更提交前还会出现staged第4份。`stage_current_skinned_joint_palette`每个pending draw按值插入大state，buffer stage用完整storage相等比较判稳；真正变化时active prefix与params又分两次`queue.write_buffer`。这不是可由局部retain或Arc修补的布局问题。

Render03/Plugins04必须把palette变为`stable slot + revision + active joint range`，使用device级grow-only suballocated current/previous arenas或明确的frame epoch翻转；CPU history只保存handle/revision，dirty joint ranges才pack/upload。验收必须覆盖1/64/256 joints，而不是只用满256 ABI证明双buffer行为正确。

## P0：staging路径的256 KiB单变量阈值不代表真实提交成本

`flush_updates_with_staging`只以primitive+instance+light的merged byte总数选择路径。current stride下，一段1,490个连续instances为262,240字节，会因为超过256 KiB而从“一次direct queue write”变成“一次CPU shadow到staging scratch复制 + 一次staging queue write + 一次GPU copy”。反向例子是2,730个互不相邻的单primitive ranges，总计262,080字节，低于阈值而保留2,730次direct `queue.write_buffer`。源码门禁已复算这两个分支；当前两条staging测试只验证阈值边界和blob offset，未验证range-count成本模型。

此外staging只覆盖primitive/instance/light；morph、VG和palette仍各走独立direct uploader。staging路径虽把queue write降为1次，仍逐range在CPU scratch复制并编码同数量`copy_buffer_to_buffer`命令。Render01/03应按`bytes + merged range count + contiguous ratio + backend measured setup cost`选择direct/pre-sized staging/GPU scatter，且把morph/VG/palette纳入同一frame upload artifact。少量连续range优先direct；大量离散range预尺寸scatter；pack工作超过实测阈值才交给有界worker task，不能用固定字节常量掩盖全量scene访问。

## P1：morph/VG稳定零上传仍付全payload比较，变化帧近三遍扫描

每次morph/VG upload先以slice equality扫描完整shadow和current。若内容变化但buffer不扩容，`write_changed_pod_buffer`再扫描current并为每个dirty run调用queue write，最后`clear + extend_from_slice`复制完整current；即变化数组近三遍CPU访问。stable只省后两遍和GPU写，仍是O(payload length)相等比较。大morph/VG payload也绕过新staging路径。

目标artifact应由asset/scene revision直接给出dirty ranges或immutable Arc generation；相同generation不比较payload，局部变更只访问dirty rows，缩短只更新active count而不重建capacity。只有producer无法提供delta时才允许hash/page comparison fallback，并必须受byte/time预算约束。

## P1：allocator与fallback mesh仍在错误边界做工作

ID allocator对sorted free spans做线性first-fit，精确命中执行Vec remove；pending frees每次非空都排序并与全部free spans完整归并。高churn/碎片场景成本分别为O(F)、O(P log P + F + P)，且高水位不会收缩。应在dense slot generation稳定后再选择ordered/size-class free index并记录fragmentation，不应先微调Vec。

fallback CPU-morphed skinned draw在`resolved_skinned_gpu_source_for_pending_draw`内直接`GpuMeshResource::from_asset`，即render submission期间可按draw重建GPU mesh；该问题继续由PERF-MVP-389/404的resource generation owner处理，GPUScene计划不复制第二套缓存。

## Unreal Engine本地源码依据

- `GPUScene.cpp:83-103`把全场每帧upload明确作为debug开关，并提供`r.GPUScene.ParallelUpdate=2048`的可配置工作量阈值；正常路径不是全场扫描后仅让最终write为0。
- `GPUScene.cpp:425-432,457-525`从dirty-filter view一次生成persistent primitive index对应headers，同时返回accumulators，使后续scatter uploader预尺寸而不再为计数二次遍历。
- `GPUScene.cpp:933-960`只有full-update条件才`IncludeEverything`并扫描全部instance ranges；steady path明确使用`GPUSceneDirty`和added filter。Zircon应先建立同等scene delta owner，再讨论多线程pack。
- `GPUScene.cpp:1240-1295`对0 upload立即返回，按primitive/instance/payload实际计数建立pre-sized scatter，并以工作量选择parallel；`1334-1350`才把primitive和instance pack交给render task parallel-for。依据是dirty规模，不是总scene规模。
- `GPUScene.cpp:1043-1072`primitive按POT、tiled instance按16 MiB chunk控制容量变化。Zircon的grow-only方向可以保留，但palette/morph/VG必须进入统一arena/容量策略，不能逐实例双固定buffer。

## 目标算法与实施顺序

1. Runtime04与Render03先定义camera-neutral scene generation、persistent dense slots及added/changed/removed command stream；GPUScene register/retain从viewport draw builder移出。
2. Render03/Plugins04把transform、skin source、morph weights和palette history改为slot revision/epoch handles；palette进入current/previous suballocated arena，按active joints和dirty ranges更新。
3. Render01/03发布唯一`CompiledGpuSceneUpload`，一次收集所有buffer目标、range counts和bytes；按实测cost model选择direct、pre-sized staging或GPU scatter，并把命令并入主graph/encoder。
4. dirty slot稳定后再替换first-fit allocator并量化fragmentation；morph/VG使用revision/page delta，禁止stable payload equality scan。
5. 最后才为超过实测阈值的header/pack工作启用有界worker tasks。禁止把全pending-draw/full-history遍历并行化后称为结构优化。

## 动态验收矩阵与阻塞

规模：instances 0/1/1k/100k/1M，visible 0/1/100%，changed 0/1/1/100%，cameras/viewports 1/2/8，free spans 0/1k/100k，churn 0/1/50%，morph weights 0/64/1k，VG pages 0/1k/1M，skinned instances 0/1/1k、joints 1/64/256，upload ranges 0/1/16/2,730/100k，bytes 0/4KiB/256KiB/16MiB。记录pending/live/history visits、hash probes、large-state copy bytes、allocator comparisons/sorts、dirty merge/pack jobs、queue writes、GPU copies/scatter dispatch、buffer objects/VRAM/fragmentation、CPU p50/p95/p99、CSwitch/ReadyThread、GPU timestamp和energy。

硬门：stable scene generation的GPUScene register/retain/history/payload compare visits=0；camera-only change的camera-neutral upload bytes/work=0；1% change访问近dirty slots；palette GPU buffer对象不随skinned instance×2增长，CPU不保留多份16,400-byte storage，upload近active dirty joints；单range大upload不多走无收益copy，数千小ranges不产生数千queue writes；morph/VG stable compare bytes=0；allocator allocation近O(log F)且commit无全free-list归并；pack worker数有界且主线程无全scene preparation。

当前18文件格式门禁和upload-threshold counterexample通过，但current-source动态验收不可成立：最近managed Windows `zircon_runtime` lib-test在843.4秒后因361个共享foreign编译错误结束，0 tests执行。RenderDoc 1.44、WPR和xperf可用，Tracy不可用；`target/profiling/zircon_editor.exe`为2026-08-10旧构建，早于2026-08-14 GPUScene源码，不能作为current-source capture。故F2 counters、WPR/energy、GPU timestamp和RenderDoc均保持pending，本记录不进入`review.md`。
