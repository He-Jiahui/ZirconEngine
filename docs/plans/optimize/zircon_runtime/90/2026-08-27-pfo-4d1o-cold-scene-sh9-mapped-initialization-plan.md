# PFO-4d1o Cold Scene SH9 Mapped Initialization Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：Runtime90 PFO-4d1o，scene environment SH9 cold initialization
- 证据边界：这是raw queue权限收敛，不是运行时热点优化；多renderer neutral resource重复创建仍归PFO-4d2a。

## 审计结论

1. `create_scene_bind_group_bundle(...)` 创建长期持有的SH9 uniform buffer后立即直接`queue.write_buffer`默认值；后续帧更新已经进入唯一`FrameBufferUpload`。
2. 该默认值在buffer构造时已知，适合与exposure history一致地使用`create_buffer_init` mapped initialization；无需ticket、submit、poll或feature queue。
3. 同一构造器还为每个renderer创建neutral cubemap、BRDF LUT、sampler和bind group。只迁移SH9不会解决PERF-MVP-351/390，因此本切片不改queue参数，也不声称第二renderer增量为0。
4. Unreal `FRenderResource::InitRHI`与system textures把固定初值绑定在resource initialization owner；Zircon映射为cold mapped creation，后续动态SH9仍由frame transaction更新。

## 实施与验收

1. 引入`wgpu::util::DeviceExt`，以`create_buffer_init`和`SceneEnvironmentSh9::default()` bytes创建SH9 buffer，同时保留`UNIFORM | COPY_DST` usage。
2. 删除构造器唯一`queue.write_buffer`，不更改fallback cubemap和BRDF LUT的cold texture初始化。
3. source guard锁定mapped initialization、动态`COPY_DST`能力和构造器raw buffer write=0。
4. 精确rustfmt/source/diff静态检查通过；Cargo和WGPU留到里程碑验证。

## PFO-4d2a 交接

- 在`WgpuRenderDevice` generation-local registry中建立共享neutral environment/system-texture owner，包含黑色cube、BRDF LUT、sampler及其typed views；初始化上传受device owner计量并能在device loss后重建。
- renderer只保留per-renderer scene uniform/SH9动态buffer与bind group，第二renderer neutral texture/sampler create/upload为0。
- 禁止用process-global `OnceLock<raw wgpu resources>`，因为它无法按device generation失效，也绕过budget、last-use和retirement。

## 当前源码结果

- scene SH9 buffer以default bytes mapped creation，仍保留`UNIFORM | COPY_DST`供后续frame transaction更新。
- 构造器raw buffer write从1降为0；新鲜scene-renderer直接buffer write扫描从8次/8文件降为7次/7文件。
- 精确rustfmt、mapped-init/source guard与scoped diff静态检查通过。Cargo、WGPU和PFO-4d2a多renderer资源计数仍pending。
