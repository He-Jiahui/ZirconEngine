# PFO-4d1n Environment Cubemap Staging Transaction Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：Runtime90 PFO-4d1n，static environment cubemap immutable artifact、staging upload 与 scene submission transaction
- 前置：PFO-4d1a-m 已建立每帧唯一 `FrameBufferUpload`、graph/submit failure retry 与 success-only cubemap upload-key commit。
- 证据边界：本切片只修正算法和所有权。未取得真实 WGPU、RenderDoc、截图、profile 或功耗数据前，不声明 GPU 瓶颈消失或性能达到目标。

## 当前模块审计

1. `EnvironmentExtract::source_cubemap(...)` 已在 render submission 之前构建 mip-major、face-packed、256-byte row-aligned 的 immutable `SourceCubemapUploadArtifact`；正常产品路径不需要在 render thread 重做 RGBA16F 转码。
2. `CubemapUploadStagingArena::encode(...)` 已把所有变更 mip 合并进一个复用 staging buffer，并在 scene encoder 中记录 `copy_buffer_to_texture`，但仍直接调用一次 `queue.write_buffer`，绕过 frame upload admission、producer ledger 与字节计量。
3. `SceneEnvironmentCubemap::ensure_uploaded(...)` 在 prepared artifact 缺失、不完整或 staging 失败时，回退为 render-thread 浮点转码和逐 face/mip `queue.write_texture`。该支路既扩大最坏帧 CPU 工作，又在后续 graph/scene 可能失败前留下 GPU side effect。
4. source/specular artifact 是完整 mip 集；irradiance artifact 当前为 `Option`，无 IEM 时依赖 render-thread 生成 1x1 黑色 cubemap。要删除回退，artifact 必须覆盖这个合法默认资源。
5. outer `write_scene_uniform(...)` 已拥有 scene encoder 和本帧唯一 `WgpuBufferUploadBatch`，并且 cubemap upload key 只在 scene submission 成功后 commit；无需新增 uploader、ticket、submit 或 poll。

## Unreal / Lumen 对齐

- `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h:308` 明确把 upload 数据生命周期延长到 graph execution，并在 pass 执行前统一上传；feature 不直接拥有 platform queue。
- `dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/DynamicRHI.h:153` 和约 `550` 把 cube-face lock / texture update 放在 `FRHICommandList` owner 下；Zircon 对应为 immutable artifact、frame upload transaction 和 scene encoder copy。
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp:409` 的 reflection cubemap 由显式 render-thread resource owner 创建，并在 RDG/RHI command owner 内清理和写入。Zircon不复制其API，但保持 persistent texture owner、command ordering 与 success-only publication。
- `dev/LumenInUE5.5.4WithComputeShader` 的 D3D12 upload/copy 示例只用于核对 upload-resource 到 default-resource copy 的命令顺序；不能把 command list、queue 或 native resource权限散发给 feature。

## 实施方案

1. 把 `SourceCubemapUploadArtifact::irradiance_mip` 收敛为必有值；无 IEM 时预编码 row-aligned 1x1 black RGBA16F cubemap。`SkyboxSettings::source_cubemap(...)` 保证进入 extract/frame 前拥有 current artifact。
2. `ensure_uploaded(...)` 先验证当前 key 的完整 source/specular/irradiance artifact，再分配或替换 native texture；缺失/布局不匹配时 fail closed，不修改 active resource owner。
3. `CubemapUploadStagingArena` 用 checked size/offset 计算精确准备一个共享 immutable payload，确保 grow-only GPU staging capacity，向调用者的 `WgpuBufferUploadBatch` 追加一个 staging-buffer range，再在同一个 scene encoder中记录每 mip copy。
4. 删除 cubemap dynamic render-thread RGBA16F fallback encoders及其 `queue.write_texture/write_buffer`；`ensure_uploaded(...)` 不再接收 `Queue`。constructor 的cold 1x1 fallback texture初始化留给PFO-4d2 persistent resource owner，不在本切片伪装成dynamic migration。
5. pending upload key 继续在录制时 record，只在 scene submit成功后 commit；graph、batch admission或scene submit失败均保持 committed key，下一帧重建相同 upload transaction。

## 算法与性能门槛

- stable key：`O(1)` identity比较，0 payload、0 driver write、0 copy command。
- changed key：设总上传字节为 `B`、mip 数为 `M`，CPU pack `O(B + M)`、一个共享 payload、一个 staging buffer upload range、`M` 个 GPU copy；不按 face 产生 driver write。
- GPU staging capacity 只按 `next_power_of_two(max(B, 64 KiB))` 增长，不缩容；offset/总字节全部 checked，禁止溢出后静默走慢速路径。
- 删除旧回退最坏上界：render-thread 浮点转码 `O(B)` 与至多 `6 * M` 个 texture queue writes 降为0。
- 动态阶段必须分别记录 stable/changed frame 的 CPU prepare、upload bytes、native write/copy 数、GPU scene时间与300帧功耗；没有测量不引入 size/frequency 阈值。

## 源码阶段验收

- source cubemap artifact默认 IEM完整，constructor/extract均不把无artifact对象交给renderer。
- environment cubemap dynamic `ensure_uploaded/upload_batch` 的 `Queue` 参数、`queue.write_buffer`、`queue.write_texture`和render-thread texel encoder为0。
- staging batch append发生在 outer `FrameBufferUpload` admission之前，cubemap key commit发生在scene submission之后。
- 精确文件 rustfmt、source contract checks与scoped `git diff --check`通过。
- Cargo、真实WGPU、静态HDRI与no-IEM产品截图、RenderDoc、规模profile和功耗留在里程碑测试阶段。

## 当前源码结果

1. `SourceCubemapUploadArtifact` 现在始终包含 irradiance mip；无 IEM 时在 render submission 之前生成 row-aligned 1x1 black RGBA16F cube。`SkyboxSettings::source_cubemap(...)` 成为 current artifact 的构造边界，`EnvironmentExtract` 不再重复准备。
2. dynamic cubemap owner 会先验证 current artifact 及三组纹理布局；尺寸变化时以局部 texture/view/sampler 完成 staging/copy 录制，成功后才整体替换 owner 字段。artifact缺失、布局不匹配或 staging失败都会 fail closed，不会留下“尺寸已更新但bind group仍指向旧view”的半发布状态。
3. `CubemapUploadStagingArena` 保留 grow-only GPU buffer和host scratch，把所有变更mip打成一个共享immutable payload，向outer `WgpuBufferUploadBatch` 追加一个range，再在scene encoder内记录每mip copy。hot path raw `queue.write_buffer` 从1降为0。
4. source/specular/IEM 的 render-thread float encode 与逐face/mip `queue.write_texture` fallback已删除；稳定key仍在artifact读取和payload分配之前返回。pending upload key仍只在scene submission成功后commit。
5. 精确Rust文件通过`rustfmt --edition 2021 --config skip_children=true --check`；artifact完整性、dynamic queue权限、validate/rebind/stage/record顺序、scene-submit-before-commit与scoped `git diff --check`静态检查通过。
6. 新鲜scene-renderer直接buffer write扫描由9次/9文件降为8次/8文件；cold cubemap fallback的1个`queue.write_texture` call site归PFO-4d2 persistent initialization。Cargo、WGPU、HDRI/no-IEM PNG、RenderDoc、profile与功耗仍pending。
