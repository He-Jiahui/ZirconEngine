---
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/extract_output.rs
tests:
  - current frame-context builder slice 11 of 11 Rust files reviewed, 1711 lines
  - all 18 existing and added tests read; four source/ownership regressions added in this slice
  - descriptor, particle, provider, VG output and material-cache source gate changed from RED to GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 multi-camera trace and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics frame submission context静态审查（2026-07-18）

## 当前源覆盖

`build_frame_submission_context/**`当前11/11个Rust文件、1,720行已逐文件静态阅读，18条现有及新增测试均已读。另完整读取自动Virtual Geometry output owner、viewport history owner、camera loop source restore与material feature定义，确认clone与move边界；这些支撑读取不冒充其父目录已动态验收。

## 直接止损

camera history key原先即使已有selected descriptor也先深clone整份camera descriptor；现直接借用descriptor构key，只在旧fallback payload路径构造临时descriptor。

advanced material feature resolve原先对每个visible mesh调用`effective_material`，相同material id会重复load child/最多4级parent、创建visited set与lineage Vec、继承整份MaterialAsset。本轮增加frame-local `HashMap<ResourceId, Option<StandardPbrMaterialFeatures>>`，成功与失败都按唯一root缓存，使该阶段从meshes×lineage收敛到unique materials×lineage；最终generation resident artifact仍由Render08负责。

viewport snapshot已经owned的particle previous sprites此前又`to_vec`复制到extract；motion camera、advanced runtime plan、Solari report与capability summary也在context构造前二次clone。本轮由`ViewportRecordState`的单次take接口移动到extract/context。自动VG fallback原先clone provider registration String，并把extract、CPU reference、BVH visualization和resident page payload四组大对象全部clone/to_vec；现只clone provider `Arc`，`VirtualGeometryRuntimeExtractOutput::into_parts`把四组owner一次性move入context。五项源码门禁先得到RED，修改后全部GREEN；rustfmt和`git diff --check`通过。

## 剩余根因与参考

同一camera submission先调用`compile_submission_pipeline`取得runtime feature flags，构造AA/post/IBL options后又调用`compile_submission_pipeline_with_options`。稳定帧也有两次wide cache-key clone/hash与两次state锁；首次variant缺失时还可能在全局state锁内compile两套graph。`resolve_viewport_record_state`同样在state锁内深cloneprevious visibility/static index/particles、pipeline asset和capabilities，并调用Solari provider。该部分新增`PERF-MVP-414`，同时复用`PERF-MVP-365/411/413`，禁止另建不一致cache。

material/subsurface两阶段仍各自扫描meshes，frame-local material cache每camera重建；texture camera target每frame load asset，environment IBL compile option每camera调用runtime dispatch并可能检查cache文件；automatic VG也每camera load models/build extract。Render08/11与Runtime07须按asset/scene/environment/camera generations发布共享artifact，normal frame不允许文件I/O或parent/model重复load。

本地Bevy `crates/bevy_pbr/src/material.rs`把entity→material id放入带change tick的`RenderMaterialInstances`，material本体通过`PreparedMaterial`/`prepare_assets`按asset owner准备；本计划只采用“实例轻引用、唯一asset按变化准备”的边界，不复制其ECS系统或类型。UE RDG requested debug gate仍只用于PERF-MVP-413诊断门控，不作为context缓存实现模板。

## 验收状态

静态、源码RED→GREEN、rustfmt与diff门禁已完成。Windows validator仍在Cargo启动前`ConvertFrom-Json`失败，本轮未绕过协调器运行raw Cargo；18条测试没有current-source执行结果。`renderdoccmd.exe`仍不可用，也没有本切片capture；双compile/cache key、state-lock hold、material/IBL/model I/O和多camera规模数据均待验收，因此只更新`pending.md`，不进入`review.md`。
