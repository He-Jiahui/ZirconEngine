# PFO-4d1k Skinned Palette Arena Hard-Cut Plan

## 状态

- 日期：2026-08-27
- 范围：Runtime90 PFO-4d1k，GPU Scene skinning palette resource/upload/binding 架构
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 前置：PFO-4d1a-j 的 frame upload transaction 与两阶段 CPU state publication 已完成源码实施和静态检查；动态验收仍 pending。
- 证据边界：以下容量与对象数量由源码 ABI 推导，不是 RenderDoc、驱动内存或功耗实测。

## 结构性问题

当前每个 skinned stable instance 在 `GpuSceneSkinnedJointPaletteBuffers` 中持有两个独立 `wgpu::Buffer`。每个 buffer 使用固定 `SkinnedMeshJointPaletteStorage` ABI：256 个 `mat4x4<f32>` 加 16-byte params，共 16,400 bytes。每次 pose 更新对选中的 current buffer 执行矩阵前缀和 params 两次 `Queue::write_buffer`，mesh draw 随后创建一个绑定 current/previous palette 的专用 GPU Scene bind group。

对 1,000 个 64-joint instance，源码结构下界为：

- 2,000 个 palette buffer；
- 32,800,000 bytes（31.28 MiB）固定 GPU palette capacity；
- 每个更新帧最多 2,000 次 palette queue write；
- 每次 mesh draw build 最多 1,000 个 palette-specific bind group；
- active joint matrix 数据本身仅 8,192,000 bytes（7.81 MiB）双缓冲，固定 256-joint tail 造成约 4 倍容量放大。

这不是把两次 write 改成 batch 就能解决的问题。draw-local palette 还破坏了 GPU Scene 的 instance-index 模型：indirect compaction 使用 phase-global visible-remap bind group，而 palette 身份仍藏在 draw command 的 bind-group override 中，无法自然支持跨 instance 的紧凑 indirect submission。

## Unreal 对照

1. `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Skinning/SkinningSceneExtension.h` 使用 persistent growable scene buffers、span allocators 和 header/indirection 数据，不为每个 primitive 创建固定最大骨骼 buffer。
2. `SkinningSceneExtension.cpp::FinishSkinningBufferUpload` 通过 `ResizeAndUploadTo` 把 header、bone map、hierarchy、object-space 和 transform uploader 汇入 RDG，并用 blackboard 保证同一 builder 只发布一次。
3. `AnimRuntimeTransformProvider.cpp` 把 current/previous transform 写入 scene-global scatter uploader；shader通过 indirection/offset 访问，而不是每 draw 重新绑定 transform resource。
4. Zircon MVP 不复制 UE 的完整压缩 transform、defrag 和 provider 系统，但必须先采用同一所有权方向：scene-global growable storage + instance indirection + frame-owned upload。

## 目标架构

`GpuSceneSkinnedPaletteArena` 持有两个 grow-only storage buffer：

- staged slot：当前待提交 frame 的所有 active joint matrices，按 frame pending-draw 顺序紧凑打包；
- committed slot：上一个成功 scene frame 的 current palette，作为本帧 previous palette；
- 每个 slot 只在容量不足时按 power-of-two bytes 扩容；
- 两个场景 bind group 预先表达 `[slot0 current, slot1 previous]` 与相反方向，正常帧只选择方向，不按 draw 创建 bind group；buffer 替换或其它 GPU Scene resource 替换时才重建。

`GpuInstanceData` 新增一个 16-byte `skinning_palette_params`：

```text
[current_matrix_base, current_joint_count, previous_matrix_base, previous_joint_count]
```

WGSL 的 binding 3/4 改为 `array<mat4x4<f32>>`。所有 skinning helper 显式接收 `instance_index`，先读取 instance params，再以 `base + joint_index` 访问 arena。joint count 为零时保持现有未蒙皮 fallback。

## 帧状态机

```text
begin palette frame
  -> resolve previous history from last successful scene
  -> pack current active matrices and instance offsets
  -> grow staged buffer if needed / rebuild two scene bind groups
  -> append one contiguous palette upload to GpuScenePreparedUpload
  -> backend accepts one merged FrameBufferUpload ticket
  -> submit scene packet
  -> on scene success: staged slot/spans become committed previous
```

graph/admission/scene failure不滚动 committed slot 或 previous spans；下一帧覆盖同一 staged slot 并重新上传。该语义与现有 `roll_prev_skinned_palettes_after_success` 对齐，不把仅准备或仅受理的 pose 误当作可见上一帧。

## 算法与复杂度

- frame packing：`O(total active joints)` 时间和一个可复用 `Vec<mat4>` scratch；不扫描历史 palette allocation。
- upload：非空 frame 一个连续 payload range，bytes = `64 * total active joints`；零 joints 不产生 upload。
- lookup：current span 在 frame-local `HashMap<stable_key, span>` 中均摊 `O(1)`，重复 stable key 复用 span；previous span 从 committed map 均摊 `O(1)`。
- resource count：palette GPU buffer 从 `2 * live skinned instances` 收敛为 2；direct scene palette bind group 固定为 2，visible-remap 每 active indirect phase 仍按其 buffer identity创建，但不再乘以 skinned draw count。
- memory：以 1,000 x 64 joints 为例，双缓冲 active payload 从 31.28 MiB 固定容量下界收敛到约 7.81 MiB，另有 power-of-two growth slack；真实 committed bytes 和 allocator峰值必须动态测量。
- CPU pose/history 的固定 256-joint `Copy` storage 本阶段暂不改为压缩 provider ABI，避免把 GPU resource hard cut 与 animation-data representation 混成一次不可验证迁移；它列为后续结构任务。

## 硬切步骤

1. 新增 arena/span/prepare/commit owner及纯算法测试。
2. 扩展 Rust/WGSL instance ABI和 offset tests，修改 skinning shader helper签名。
3. GPU Scene 初始化/重建维护两套 palette方向 bind group；visible-remap group消费当前方向。
4. sync阶段在 instance upload前完成紧凑 packing，把 arena upload附着到 `GpuScenePreparedUpload`。
5. scene成功时与 previous palette history一起提交 arena slot/spans；失败不滚动。
6. 删除 per-instance buffer map、slot helper、`write_active_prefix`、palette-specific bind-group创建和 MeshDraw buffer保活字段。
7. 保留 queue/prepared stats 的语义计数，但从“buffer存在”改为“current/previous palette span存在”。
8. 更新 PFO主计划、RDG transaction记录和 graphics模块文档，重新扫描 direct writes/create-buffer/bind-group热路径。

## 当前源码结果

1. `GpuSceneSkinnedPaletteArena` 已成为 palette GPU resource owner：固定两个 storage buffer，staged slot 只在容量不足时按 power-of-two 扩容，current/previous 方向由两套 scene bind group 表达。
2. frame sync 以 pending-draw 顺序执行 `O(total active joints)` 紧凑 packing；重复 stable key 复用 span。非空帧生成一个连续矩阵 payload并附着到 `GpuScenePreparedUpload`，零矩阵不产生空写。
3. `GpuInstanceData`/WGSL ABI 已从 176 bytes 扩展为 192 bytes，新增 current/previous base+count；binding 3/4 已改为全局 `array<mat4x4<f32>>`，CPU fallback 不再误发布 GPU-skinning motion 标志。
4. scene success 才提交 staged slot/span；graph、backend admission 或 scene failure 不改变 committed previous arena，下一帧覆盖同一 staged side并重试。
5. 旧 per-instance buffer map、slot helper、`write_active_prefix`、draw-owned palette buffer、palette-specific scene bind group 和 MeshDraw override 已硬删除。skinning 生产模块 `queue.write_buffer` 为 0，draw-owned palette buffer/bind-group override 为 0。
6. 初始/minimum palette binding 已从旧单实例 16,400-byte storage ABI收敛为一个 64-byte matrix slot。固定 256-joint CPU pose/history snapshot仍保留，未冒充已完成的 animation provider压缩。
7. 1,000 x 64-joint current+previous 的精确 active payload为 8,192,000 bytes（7.8125 MiB），对比旧固定 GPU capacity 32,800,000 bytes（31.2805 MiB）；该数字是源码布局推导，不是驱动 allocation实测。
8. 新鲜共享工作树扫描为 scene-renderer 12 次直接 buffer write/11 个非测试生产文件；其中包含本切片之外同时出现的 UI写入点。本切片可归因变化仅为 skinning生产模块 2->0，不能把共享工作树绝对数伪装成单调性能结果。
9. 精确 Rust 文件 rustfmt、scoped diff check、旧符号扫描、skinning direct-write扫描和 GPU Scene ABI offset源码检查通过。Cargo、真实 WGPU、PNG、RenderDoc、profile、memory和功耗验收均未运行/未声明通过。

## 动态验收（pending）

- Windows WGPU：first frame、stable pose、pose change、joint-count change、despawn/respawn、CPU-morph previous-source和scene failure retry。
- RenderDoc：binding 3/4 为两块global arena；instance params基址/数量正确；indirect remap路径使用同一方向；current upload先于全部mesh consumer。
- 规模：1K/10K skinned instances，分别记录 active joints、arena capacity/bytes、buffer creates、bind-group creates、upload ranges/bytes、CPU pack p50/p95/p99、GPU skinning耗时和功耗。
- 没有这些实测不得宣称 memory slack、driver overhead、功耗或最终压缩格式已经最优。
