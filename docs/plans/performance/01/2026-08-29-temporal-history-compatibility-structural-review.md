---
related_code:
  - zircon_runtime/src/core/framework/render/camera/camera_snapshot.rs
  - zircon_runtime/src/graphics/runtime/history/validation_key.rs
  - zircon_runtime/src/graphics/runtime/history/is_compatible.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build/effective_view_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/LumenInUE5.5.4WithComputeShader/Res/Shader/ScreenProbeGather/TemporalReprojection.hlsl
status: source_implemented_dynamic_acceptance_pending
---

# Temporal history compatibility结构性复核（2026-08-29）

## 结论

当前首要瓶颈是正确性合同，不是局部hash或clone技巧。旧`FrameHistoryValidationKey`把完整camera、mesh、animation、lighting、post-process与particle内容当作全局兼容条件；正常运动会阻断reprojection。对旧实现做hash只会把错误的全帧失效做得更快，因此本轮先完成结构硬切，不声明性能优化里程碑。

Unreal `TemporalAA.cpp`以`!InputHistory.IsValid() || View.bCameraCut`选择camera-cut路径，并不要求scene inputs逐字相等。Lumen复刻的screen-probe temporal shader用velocity重投影，再按onscreen与history depth差生成局部visibility weights。两者共同约束Zircon：全局只管理资源/视图结构兼容，动态内容由重投影和局部拒绝管理。

## 源码切片

`FrameHistoryValidationKey`现在只含world raw identity、camera core/projection/custom-projection/HDR/MSAA合同和canonical effective feature集合。world generation、camera transform、geometry、animation、lighting、post-process与particle不再进入key。feature list在构造时排序去重，顺序或重复项不制造假失效。

既有camera velocity连续性算法上移到`ViewportCameraSnapshot::supports_temporal_reprojection_from`。submission在发布previous history前调用它；velocity params也调用同一函数。正常pan和小幅FOV变化保留history，大幅translation/rotation、projection kind/custom projection、clip或projection shape不连续返回`FrameInputsChanged`，但复用同一纹理allocation。

## 算法规模

旧key构造与比较规模随每camera可见mesh、animation pose、light、post-process volume与particle payload增长，并复制多组动态数据。新key不再扫描这些scene payload；剩余显式工作是effective feature排序去重，复杂度`O(F log F)`、存储`O(F)`，其中`F`是已编译feature数量而非scene primitive数量。该复杂度是源码推导，不是CPU采样数据。

后续性能优化前必须先用profile证明feature canonicalization或state-lock是热点；在没有数据前不引入hash缓存或generation token。若compiled feature producer能保证canonical order，可在测量后把排序收敛为`O(F)`验证/拷贝。

## 验证计划

动态正确性矩阵：300帧static、camera pan/orbit/continuous zoom、rigid/skinned/morph/particle/light/post变化、large cut、resize、projection/provider/feature切换；记录每域valid ratio、reset reason、reprojection coverage与rejection heatmap。

CPU矩阵：1/8/64 cameras乘0/1k/100k meshes，比较旧基线与当前key build/compare、submission state-lock、allocation bytes和p50/p95/p99。GPU矩阵：1080p/4K同场景TAA、GI、SSR、volumetric domain timings、history bandwidth与VRAM。功耗矩阵必须固定GPU、driver、backend、画质、warmup和帧率目标，再与Unreal同场景经验值比较。

## 当前证据与开放项

精确`rustfmt`、scoped `git diff --check`和`cargo metadata --locked --no-deps`通过。focused validator在外层等待244秒后无输出超时，没有返回request id，也没有Cargo/rustc输出；当前源码仍没有compile/test结果。没有新PNG、RDC、GPU timing或功耗数据。

P0-2仍开放：`previous_history_available`仍是全局bool，TAA/HZB/SSR/SSAO/GI/exposure/volumetric尚未拥有统一的per-domain generation、valid rect、reset reason与last-successful-frame表。完成该域表前不能把P0-1源码切片外推为temporal pipeline完成。
