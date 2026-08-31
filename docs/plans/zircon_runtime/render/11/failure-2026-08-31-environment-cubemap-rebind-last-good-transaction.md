---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: environment-cubemap-rebind-last-good-transaction
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/11-environment-lighting.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_environment_capture.rs
tests:
  - managed Rust transaction tests for rebind commit, rollback, unchanged-size upload, and old-environment restoration
  - managed WGPU fault injection before resource-upload admission and graphics submission
  - current-source environment replacement PNG and RenderDoc replay under docs/tests/runtime/shader
---

# Render11: Environment cubemap rebind must preserve last-good

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：Shader06 environment/IBL publication and C3/C4 atomicity review
- 修复责任计划：`docs/plans/zircon_runtime/render/11-environment-lighting.md`
- 交接原因：最低共享根因位于 Render11 的普通 source-cubemap GPU resource/bind-group publication owner；相关文件是共享工作树中的外部在途修改，Shader06 不得拆分覆盖。

## 失败现象与复现证据

`SceneEnvironmentCubemap::ensure_uploaded` 已把 upload key 分为 `committed` 与 `pending`，但在 source/PMREM/IEM 尺寸或 mip 布局变化的 `requires_rebind` 分支中，它在 graphics submission 接受之前就立即替换全部 texture/view/size 字段。`write_scene_uniform` 随后立即用这些候选 view 替换 `scene_bind_group`。

`render_scene` 只在 `submit_graphics_command_buffers_with_frame_diagnostics_and_surface` 成功后调用 `commit_pending_upload`。在此之前的 resource-upload admission、submission transaction、writeback、surface blit 或 graphics submit 失败都会直接返回。下一次 `write_scene_uniform` 仅调用 `discard_pending_upload()`，而该函数当前只清除 pending key，不恢复旧 texture/view/size 或 scene bind group。因此控制面仍记录旧 generation，采样面却可以指向从未被接受 submission 初始化的新资源。

当前取证 hash：

- `environment_cubemap.rs`: `CE22A7DEA7D2A8007CA18226C1A93E3B039753F6FF38FC6846EC921A82ADD981`
- `write_scene_uniform.rs`: `2B5FE2C872FD12C0B51BA7D1E31C588D01FE1BAD97530DCB1D39375D101F425D`
- `render_scene.rs`: `FA7D2C301D39DCC8ADEAFFD827E108DD137294F14EDB3F30ACA9888129E15902`
- `scene_renderer_environment_capture.rs`: `13E3B18C316D2454993DF5ECA6D6EA031E9015D52DD66DE2D8D90023982F4341`

## 最低共享层根因

上传身份已是两阶段事务，但对应的 physical resources 和 bind group 仍是立即发布。这是同一 environment generation 被分成 key 与 GPU object 两个生命周期 owner，不是 PMREM 算法、WGSL 采样或 cache-key 错误。

同尺寸更新不需要新代资源：其 copy 仍位于当前 graphics encoder，未被接受的 submission 不会覆写已提交 texture。缺陷限定在 `requires_rebind` 的 texture/view/size/bind-group 替换路径。

## 架构修复验收

- 以一个 generation-qualified transaction 同时拥有 upload key、source/PMREM/IEM texture 与 view、布局尺寸、scene bind group 和 staging reservation。
- `requires_rebind` 时旧代继续作为 last-good；当前 encoder/draw 可以引用候选 bind group，但不得把候选写入持久 committed 字段。
- resource-upload admission 或 graphics submit 前任一失败都丢弃候选，旧 texture/view/bind group 仍可采样，committed key 不变。
- graphics submission 接受后原子提交候选 generation，然后才允许旧代进入现有 fence/retirement owner；不得以 GPU completion wait 代替 submission admission。
- 同尺寸更新保持现有原位 copy 路径，不因事务化每次重建 texture 或 bind group。
- 单元回归覆盖 rebind commit、所有提交前 rollback、失败后返回旧 environment、下一帧仍请求新 environment 时重试，以及同尺寸路径不新建资源。
- managed WGPU fault injection 验证失败帧之后仍采样 last-good；current-source PNG/RenderDoc 证明替换成功后 source/PMREM/SH9 来自同一 generation。

## 禁止临时方案

- 不得只回滚 pending key，或只恢复 texture 而留下指向候选 view 的 bind group。
- 不得在每个早退分支手工复制回滚代码；回滚必须由单一 transaction/guard owner 覆盖未来新失败路径。
- 不得通过 `device.poll`/wait、同步 readback、恢复 CPU texel upload 或禁用 environment replacement 规避失败。
- 不得将普通 environment 资源塞入 realtime IBL 双槽或 reflection-probe array；三者保持独立资源所有权。
- 不得为了补救 rollback 而让稳定同尺寸路径每帧创建 GPU object。

## 修复结果与回传

Open state: `待修复`; no runtime, image, timing, memory, or power pass is claimed.
