# PFO-4d2d Lightmap Atlas Generation Binding Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：lightmap atlas prepared revision、physical resource lease与deferred/forward binding发布
- 前置：PFO-4d2c已修复reflection provider替换后的deferred publication；动态WGPU验收仍pending。

## 结构重审与问题证据

`ResourceStreamer::ensure_texture_for_frame`会按资产revision更新`PreparedTexture { revision, resource }`，mip residency也允许同revision发布新的physical `GpuTextureResource`。但`SceneLightmapResources::prepare`只保存`atlas_asset: Option<ResourceId>`，并在读取streamer当前prepared row之前执行同ID早退；`ResourceStreamer::texture`又只返回`Arc<GpuTextureResource>`而丢弃prepared revision。

因此以下两类合法替换都可能被旧lightmap binding静默屏蔽：同一atlas资产热重载后的新revision，以及同revision residency变化后的新physical resource。旧`Arc`继续保活旧view/sampler，不会形成WGPU validation error，却会让forward/deferred消费过期lightmap。这是artifact/generation identity正确性缺口，不是以源码调用次数推断的性能优化。

## Unreal对齐与设计

Unreal的render resource publication以resource revision/generation与RHI reference共同确定当前scene binding；Zircon使用等价的prepared row快照，不允许consumer仅凭semantic asset id判断GPU lease仍current。

1. `ResourceStreamer`新增restricted accessor，一次读取同一`PreparedTexture` row并返回`(revision, Arc<GpuTextureResource>)`；missing row返回`None`，不借fallback掩盖未准备资产。
2. `SceneLightmapResources`同时保存atlas asset id与prepared revision。
3. stable判定必须同时满足asset id相同、revision相同、当前与候选`Arc::ptr_eq`；任一变化都重新发布atlas lease。
4. 无baked-lighting分支原子清除resource/id/revision。descriptor与resource id校验通过后才修改当前状态，错误保留上一合法lease。
5. 已有frame setup继续把最终lightmap bindings同时发布给deferred owner；forward consumer从同一`SceneLightmapResources`读取。
6. 不新增device/queue/poll/submit权限，不创建native资源，不改变texture residency owner。

## 验收门槛

- source门槛：ID-only early return为0；prepared revision accessor为1；stable判定包含revision与`Arc::ptr_eq`；clear分支清除三元状态。
- 局部`rustfmt --check`和scoped `git diff --check`通过。
- 动态验收：同ID revision更新与同revision physical replacement分别验证forward/deferred采样切换，WGPU validation为0，RenderDoc显示新view/sampler identity，PNG写入`docs/tests/runtime/render`。未取得这些证据前只标source implemented。

## 当前完成项与静态结果

- `ResourceStreamer::texture_with_revision`从同一prepared row返回revision与`Arc<GpuTextureResource>`，missing row返回`None`且不借fallback伪装ready。
- lightmap atlas stable判定现同时要求asset id、prepared revision与physical `Arc` identity相同；ID-only early return从1降为0。同ID新revision或同revision新physical lease都会重新发布。
- 无baked-lighting分支清除resource/id/revision三元状态；descriptor/id验证失败仍保留上一合法lease。
- 三个触及Rust文件的局部`rustfmt --check`、scoped `git diff --check`与精确source计数通过。未运行Cargo、真实WGPU、PNG、RenderDoc、profile或功耗验收。
