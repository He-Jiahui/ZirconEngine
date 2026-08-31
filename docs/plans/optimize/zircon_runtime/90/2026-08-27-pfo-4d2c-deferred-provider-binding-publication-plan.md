# PFO-4d2c Deferred Provider Binding Publication Plan

## 状态

- 日期：2026-08-27
- 当前状态：`source_implemented_static_checks_passed_dynamic_validation_pending`
- 范围：reflection provider device-generation替换后的deferred binding lease发布
- 前置：PFO-4d2a/b已收敛system fallback与shadow slot workspace；动态WGPU验收仍pending。

## 结构重审与问题证据

`SceneReflectionProbeResources::prepare`允许environment-only placeholder在首次planar/cubemap provider出现时，通过`upgrade_environment_only_provider`整体替换probe/header buffers、cubemap-array view、planar texture/view及params buffer。该替换发生在`SceneRendererCore::write_scene_uniform`中。

forward receiver每次创建bind group时会重新读取`mesh_pipelines.reflection_probes.bindings()`，但`DeferredSceneResources`只在renderer构造时保存一次`ReflectionProbeGpuBindings`，帧准备阶段只重新发布了lightmap bindings。结果是provider升级后，deferred lighting仍可能持有旧placeholder handle；旧handle因`Arc` lease继续存活，不会触发WGPU lifetime错误，却会静默采样1x1/单slot占位资源。这是generation publication正确性问题，不是需要先凭profile决定的微优化。

## Unreal对齐与设计

Unreal在render-thread frame setup更新scene/view uniform parameter snapshot，pass只消费该帧已发布的资源引用。Zircon采用同一边界：provider owner先完成候选替换，再向所有跨模块consumer发布同一最终binding lease，然后才允许variant选择和graph录制。

1. `DeferredSceneResources`新增restricted reflection binding setter，与已有lightmap setter形成一致发布接口。
2. `write_scene_uniform`在`reflection_probes.prepare`返回之后立即抓取最终`bindings()`并发布给deferred owner。
3. provider profile downgrade、mesh command cache invalidation仍发生在发布之后；这样本帧deferred binding与随后选择的generic environment variant来自同一provider generation。
4. 不新增queue/device/poll/submit权限，不创建native WGPU资源，不建立第二套reflection owner。
5. 当前不实现forward receiver bind-group cache。该候选必须先取得真实native create成本、CPU p50/p95/p99与1/4/16/64 pass规模profile，再单独立项，避免用源码命中数冒充性能瓶颈。

## 验收门槛

- source顺序：`reflection_probes.prepare` < `set_reflection_probe_bindings` < `requires_generic_environment_pbr` < variant downgrade/cache clear。
- upgrade后deferred和forward consumer必须引用同一最终provider handle集合；source guard不替代真实WGPU validation。
- 局部`rustfmt --check`和scoped `git diff --check`通过。
- 动态验收：environment-only启动后注入planar/cubemap provider，真实deferred输出PNG非占位、WGPU validation为0、RenderDoc绑定资源身份一致。未取得这些证据前状态只标source implemented。

## 当前完成项与静态结果

- `DeferredSceneResources`新增restricted reflection binding setter；`write_scene_uniform`在provider `prepare`完成后立即发布最终`ReflectionProbeGpuBindings`，再观察provider upgrade并切换generic environment variant。
- 静态顺序为prepare → deferred lease publish → provider profile观察 → variant降级 → mesh command cache清理；reflection prepare与deferred publish均恰好一处。
- 两个触及Rust文件的局部`rustfmt --check`、scoped `git diff --check`通过。未运行Cargo、真实WGPU、PNG、RenderDoc、profile或功耗验收。
- forward receiver bind-group缓存候选保持未实施；在取得真实native create/CPU profile前不把源码调用次数当成性能瓶颈。
