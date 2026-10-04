---
related_code:
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render_visibility.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/12-effects-particles.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/godot/scene/3d/gpu_particles_3d.cpp
tests:
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - current-source Windows zircon_runtime render-extract tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene world render projection逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/world/{render.rs,render/lights.rs,render_particles.rs,render_post_process.rs,render_visibility.rs}`当前源 **5/5** 个Rust文件、**1,990** 行、**8** 个就地test已逐文件阅读；范围包含viewport/frame extract、camera/mesh/sprite/light/Volume/particle投影、visibility input及inactive fallback。

## 已直接修复

`collect_render_particles`原先每camera/frame遍历全部`World.entities`，对每entity分别调用四次`dynamic_component`，即使场景没有对应payload也支付外层HashMap探测。现只收集并稳定排序`dynamic_components` owner ids，每owner做一次外层lookup，再在局部map查四个候选key；由于owner已按entity排序，删除emitters与bounds的两次重复sort。源码守卫先RED后GREEN，rustfmt/diff通过，归PERF-MVP-465。

## 稳定帧重复投影仍未解决

- 兼容`build_viewport_render_packet(&self)`仍clone完整World以运行RenderExtract internal systems，继续归PERF-MVP-349。
- 每个camera extract分别遍历mesh/sprite、五类light、volumetric light和Volume，多次读取active/layer/transform/mobility并构造/sort owned Vec；camera descriptors和layer unions也按request重建。应由Runtime07发布scene generation artifact，多camera只做view-specific filtering。
- Volume profile展开、同camera多个消费者重复resolve继续归PERF-MVP-363/364；visibility的scene SoA/candidate与多view全量工作继续归419。
- 粒子止损后仍每camera collect+sort dynamic-owner ids，并逐JSON field解析、创建sprite/bounds/gpu-frame Vec及透明depth sort。PERF-MVP-465最终由Runtime07/Render12把authoring JSON在import/component-change边界编译为typed emitter artifact，stable generation和多camera共享；GPU档以buffer/indirect handle为权威，不在scene extract回读/重建粒子正文。

`render_visibility.rs`还会先构造/sort`renderables`，再分别扫描生成renderable/static/dynamic三份entity Vec；本轮尝试申请该共享文件的写租约时发生精确冲突，按协作规则只读保留，交由PERF-MVP-419的single visibility artifact收敛，未越权覆盖。

## 参考引擎对照

Bevy `ExtractComponentPlugin`通过ECS typed query只访问匹配component（可加visible filter），并把app-world→render-world边界显式化；Zircon应采用typed/revision query和单一render artifact原则，而非每consumer扫描全World。Godot `GPUParticles3D`只在状态/visibility/processing边沿请求RenderingServer更新，inactive/one-shot结束后关闭internal process；Zircon粒子owner同样需要activity/revision gate，不应稳定帧反复解析JSON。

## 动态验收

1. current-source Cargo：camera override/fallback/order/stack、mesh LOD/primitives/material override、sprite/light/layer、Volume/fog、particle JSON/HUD/GPU frame、visibility output与inactive frame。
2. nodes/renderables/lights/volumes/dynamic owners/particles 1/1k/100k、cameras 1/8/64、stable/1% change记录World/entity/component visits、JSON fields、sorts、Vec/String clone bytes、extract builds与CPU p50/p95/p99。
3. GPU timestamp/RenderDoc：F2最小产品场景核draw/dispatch/copy/pass/resource；PERF-MVP-465要求当前world entity visits=0、outer dynamic lookup≤owners、emitters/bounds二次sort=0，最终stable JSON/extract/sort=0。

`renderdoccmd.exe --help`当前环境仍不可用，现有`.rdc`由其他会话持有；受管Cargo也被共享预约阻塞。动态证据未完成，因此本目录继续保留在`pending.md`，不进入`review.md`。
