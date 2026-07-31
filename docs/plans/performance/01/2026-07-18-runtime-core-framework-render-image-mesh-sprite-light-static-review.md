---
related_code:
  - zircon_runtime/src/core/framework/render/image
  - zircon_runtime/src/core/framework/render/mesh
  - zircon_runtime/src/core/framework/render/sprite
  - zircon_runtime/src/core/framework/render/light
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite
  - zircon_runtime/src/scene/world/render/lights.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_sprite_render/src/render/mod.rs
  - dev/bevy/crates/bevy_sprite_render/src/texture_slice/computed_slices.rs
tests:
  - render image eight of eight Rust files reviewed
  - render mesh five of five Rust files reviewed
  - render sprite eight of eight Rust files reviewed
  - render light five of five Rust files reviewed
  - sprite source-guard RED to GREEN for one matrix and no per-slice vertex Vec
  - rustfmt and scoped git diff check passed
  - current-source Cargo, counters, product traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render image/mesh/sprite/light逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`core/framework/render/image/**` 8/8、`mesh/**` 5/5、`sprite/**` 8/8、`light/**` 5/5，共26个Rust文件、818行；继续追到asset texture/mesh描述符、scene sprite/light extraction、phase queue、sprite vertex/batch/record与render stats生产调用链。该聚焦调用链不等于`zircon_runtime/src/graphics/**`逐文件验收。

image与mesh契约主要发生在asset/resource generation边界：image的format String和usage Vec值得在资源规模测试中量化，但当前生产读取以借用descriptor为主；mesh bounds为单遍、无临时分配，未发现独立每帧根因。GPU light 128-byte布局与shadow tier是Copy固定数据，问题集中在extract/diagnostics ownership。

## PERF-MVP-337：sprite重复CPU展开、排序与临时GPU对象

scene先按`(z_order, entity)`排序sprite Vec，`SpriteExtract`再分配phase input Vec并建立第二个排序phase queue。每render stage还把phase indices收集成临时Vec。九宫/平铺最多展开1,000 slices；原实现对每slice重算同一transform matrix并分配6-vertex Vec后拷入父Vec。本轮按TDD用每sprite一次matrix、精确reserve和固定数组append直接清除了这部分重复，行为/source guard、rustfmt与diff check通过。

剩余根因更大：`prepare_sprite_queue_stats`为统计完整调用`build_sprite_vertices + prepare_sprite_draw_batches`，真实`SpriteRenderer::record`随后再次完整执行；每个batch又创建一次GPU vertex buffer并开启独立render pass。稳定sprite仍每帧重算slices、CPU展开6 vertices/slice、上传并重建buffer。Bevy把tiled/nine-slice结果作为只在Sprite或Image变化时更新的`ComputedTextureSlices`，extract复用clear后的Vec/range，GPU侧复用instance/index buffers；该模式支持Render14/17硬切generation-owned slice artifact、single prepared batch authority与持久GPU instance buffer。

## PERF-MVP-338：light静态原因字符串与诊断重复遍历

每个可见rect light、每个camera extract都为固定英文degradation reason分配String；snapshot跨层clone时继续复制。render stats随后重新遍历ambient/rect slices只为统计ready/degraded，而packing/extract阶段已经读取同一标志。应以Copy `RenderLightDegradationReason`/static ID进入snapshot，只在UI/export边界格式化；ready/degraded counters由extract或light pack单遍累加并随prepared-light report传递，不能为诊断重扫/复制产品数据。该契约是跨模块公开面，本轮不做半套String→borrowed lifetime改写。

## 验收要求

sprite按1/100/10k sprites、slices 1/9/1k、phases 1/4、batches 1/100/10k、stable/1% changed记录sorts、phase/index/slice/vertex alloc、matrix builds、CPU vertex builds、upload bytes、buffer/pass/draw数和CPU/GPU p95：matrix≤1/sprite generation；stable computed-slice rebuild=0；每实际prepared batch只构建/上传一次且统计额外vertex build=0；GPU buffer creation稳定帧=0，pass按phase而非batch增长。light按1/1k/100k lights、cameras 1/8、stable/changed记录reason String alloc/clone bytes、readiness scans与pack CPU：frame hot path reason alloc=0，每light readiness visit≤1。Cargo、phase/sprite/light行为、F2产品trace与RenderDoc draw/pass/resource/upload证据全部通过前，四个目录留在`pending.md`。
