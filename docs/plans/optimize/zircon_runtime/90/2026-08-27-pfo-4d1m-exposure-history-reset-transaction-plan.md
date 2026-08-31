# PFO-4d1m Exposure History Reset Transaction Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：Runtime90 PFO-4d1m，per-camera exposure history invalidation 与 frame upload transaction
- 前置：history creation 已有 renderer submission ticket；PFO-4d1a-l 已建立每帧唯一 `FrameBufferUpload`。
- 证据边界：本切片修正重复初始化、native queue权限和失败重试，不把两个小 buffer range 的源码收敛解释为运行时瓶颈消失。

## 当前代码审计

1. `SceneFrameHistoryTextures` 拥有 read/write 两个 exposure storage buffer；创建时 `create_buffer_init(...)` 已把两者初始化为 `default_exposure_buffer_words()`。
2. `prepare_history_textures(...)` 对新建、resize重建和retained camera-cut都先判定 history unavailable，然后再次调用 `invalidate_exposure_history(&backend.queue)`。新建/重建因此在 mapped initialization 后重复写两次；retained reset则直接越过frame upload admission。
3. history selection发生在 outer frame owner，唯一 `frame_buffer_uploads` 在随后进入 `render_compiled_scene(...)` 后建立。为reset单独enqueue会制造第二个逻辑producer/ticket，不能接受。
4. compiled render已经持有 `Option<&mut SceneFrameHistoryTextures>`；可在 exposure resource确实live时准备reset ranges，在graph成功与backend/ledger受理后提交pending intent。graph或admission失败必须保留intent供下一帧重试。

## Unreal 对齐

- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessEyeAdaptation.cpp:1417` 创建multi-frame eye-adaptation buffers并用RHI lock/unlock写默认值；初始化属于buffer owner，不在后续每帧重复。
- 同文件约1182/1325只由view-state owner决定更新并在RDG中注册external buffer；交换由owner控制。Zircon保持双buffer与成功后的flip owner，不把reset写散发给feature queue。
- Zircon不复制UE的RHI lock API；mapped creation映射到WGPU cold initialization，retained reset映射到neutral frame upload transaction。

## 实施方案

1. 用 `exposure_reset_pending` 替代 `invalidate_exposure_history(&Queue)`；new/recreated buffers默认pending=false，因为mapped creation已经写入正确初值。
2. retained history unavailable且exposure开启时只请求reset；new/recreated history跳过重复请求。
3. `prepare_exposure_history_reset(...)` 在pending时构造一个共享immutable payload和两个目标range，追加到现有`frame_buffer_uploads`，不修改pending。
4. graph成功、batch受理且`FrameBufferUpload` ticket进入ledger后调用`commit_exposure_history_reset()`；更早失败保留pending。
5. graph执行通过短期`as_deref_mut()`重借history，不消费outer owner，以便受理后提交reset intent。

## 复杂度与验收

- retained invalidation固定两个range、一个小payload，`O(1)`；new/recreate重复range从2降为0。
- 不新增submit/poll/ticket；仍只有一个`FrameBufferUpload` producer。
- 源码阶段验证pending prepare/commit、new/recreate skip、顺序、raw queue=0、rustfmt/diff。
- 测试/里程碑阶段仍需Cargo、camera-cut/resize exposure product readback、真实WGPU screenshot、RenderDoc和300帧profile/power；缺少这些证据不声明视觉或性能验收。

## 当前源码结果

1. `SceneFrameHistoryTextures` 已用 `request -> prepare -> commit` pending intent 取代直接 `Queue::write_buffer`。prepare 复用一个 immutable payload，并向现有 frame batch 追加 read/write 两个 range；prepare 本身不清除 pending。
2. new/resize rebuild 继续由 `create_buffer_init` 写默认值，不再额外请求 reset，重复 exposure range 从 2 降为 0。retained camera-cut 的两个直接 queue write 降为 0。
3. compiled frame owner 在 exposure history 确实 live 时把 reset ranges 合入唯一 `FrameBufferUpload`；只有 backend 受理且 producer ticket 写入 ledger 后才 commit。graph 或 admission 失败会保留 pending，下一帧重试。
4. 精确 Rust 文件已通过 `rustfmt --edition 2021 --config skip_children=true --check`；源码顺序、共享 payload、recreate skip、history raw queue=0 与 scoped `git diff --check` 静态检查通过。
5. 新鲜 scene-renderer 非测试扫描剩余 9 次直接写、9 个文件；该数字是迁移范围证据，不是驱动调用或性能结果。Cargo、真实 WGPU、camera-cut/resize 产品 readback、PNG、RenderDoc、300 帧 profile 与功耗仍 pending。
