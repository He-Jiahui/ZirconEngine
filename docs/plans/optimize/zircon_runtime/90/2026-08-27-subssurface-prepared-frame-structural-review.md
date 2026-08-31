# Subsurface Prepared Frame Structural Review

## 状态

- 日期：2026-08-27
- 当前状态：`review_complete_implementation_deferred_until_prepared_advanced_lighting_owner`
- 范围：`advanced_lighting/subsurface_pass` CPU/GPU数据准备与资源生命周期

## 结论

当前SSS主要问题不是单个`queue.write_buffer`。每个启用帧的setup/scatter会重复解析profile table，scatter clone profile rows，并创建setup params、scatter params、16-profile table三个uniform buffer；三个pass还各创建transient-view bind group。只把indirect args reset改成encoder clear会保留计划18已明确禁止的重复producer与每帧resource create，因此本轮不做这种局部修补。

## 参考与目标

- 计划18性能交接要求per-camera/scene/asset generation唯一`PreparedAdvancedLightingFrame`，其中包含SSS table；graph pass只消费prepared handles。
- Unreal `PostProcessSubsurface.cpp`与`SubsurfaceTiles.cpp`把tile buffers/indirect args注册在RDG中，并用graph compute/clear/build pass生成indirect work，不从feature直接写平台queue。
- 目标切片应一次解析profile table和camera params，复用pipeline-generation持久params/profile buffers或RDG parameter allocation，使用GPU ordered indirect reset/build，并将参数更新进入pass upload transaction。transient bind group缓存必须等PFO-4d2提供generation-qualified resource/view identity后实施。

## 实施前量化要求

- 记录SSS enabled帧的profile resolve次数、buffer/bind-group create数、upload bytes、active tile count、setup/scatter CPU与GPU p50/p95/p99。
- 目标静态上界：profile resolve每camera generation不超过1；stable pipeline/resource generation下params/profile buffer create为0；feature-off所有SSS prepare/create/upload为0。
- 未取得source-matched WGPU/RenderDoc/profile前，不宣称Burley算法、功耗或GPU耗时达到最优。
