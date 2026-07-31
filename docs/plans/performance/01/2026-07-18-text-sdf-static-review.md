---
related_code:
  - zircon_runtime/src/text/sdf
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/17-ui-wgpu-surface-and-render-graph-integration.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
tests:
  - offline manifest and instance lookup source-level RED to GREEN guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows zircon_runtime SDF tests pending
  - stable-frame CPU/upload/RenderDoc counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime text SDF逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/text/sdf`当前源24/24个Rust文件已逐文件阅读，覆盖decode/params/mode、FDSM generation、offline artifact identity/codec/path、runtime font bake/measure/fallback/cache、offline source与全部测试。`font_bake.rs`属于同时进行的外部改动，本轮只读；局部修复限定在clean的`offline_source.rs`及其测试。

## PERF-MVP-249：SDF一帧多轮准备并重建整页

产品`ScreenSpaceUiTextSystem::prepare`先对atlas slots执行`generation_failures_for_plan`，再为fallback advances执行`prepare_sdf_runs_cpu`；renderer prepare随后对native decoration与SDF texts再次CPU prepare，最后`build_atlas_from_slots`。单次CPU prepare内部又为decoration、glyph metrics、resolved advance最多三次`render_scalars()->Vec<char>`，并逐glyph重建含font/family/language String的完整key。

glyph cache命中时`bake_glyph_cached`按值clone `RawBakedGlyph`，深复制bitmap；atlas build每帧按全部page分配/清零完整R8/RGBA pixels、线性find page、复制每slot bitmap，最后仅为report再扫描全部atlas统计nonzero。renderer随后按全plan上传并新建vertex buffer。稳定generation仍重复CPU DTO、bitmap bytes、page zero/copy/report scan/upload，和PERF-MVP-231/243/244形成同一产品链。

Text05需发布单一`CompiledSdfFrame`/generation artifact：failure、advance、decoration、slot bitmap与render plan共享一次prepare；glyph cache保存Arc/slot reference而非返回bitmap clone；atlas page长期驻留，只写dirty regions，report计数在生成/写入时增量维护。Render17消费persistent GPU page/buffer与dirty upload，Text09负责cache bytes/eviction与frame work budget。

## PERF-MVP-250：source identity与generation未批处理

动态glyph在`distance_field.rs`先parse face选择glyph，随后generator再次parse同一face。offline glyph原先每glyph通过`resolve_font_face`间接加载manifest后又显式加载一次，并两次resolve instance handle；本轮用源码RED→GREEN改为一次manifest注册和一次handle解析。仍未解决的是每glyph materialize standalone face、计算整份font source BLAKE3、规范化variation、parse face、同步读取/校验artifact，并从page逐行分配复制glyph pixels；build tool还单线程重复相同face/FDSM setup。

Text05需按`font asset/face/instance/bake params + font/asset generation`建立source context，缓存manifest、standalone bytes/source hash、parsed face/axes与artifact Arc；按unique face/instance批量glyph，runtime miss进入有界worker queue并允许取消，离线build复用同一batch generator。Text01提供face metadata owner，Text09/Runtime11提供memory、queue与worker预算。

## 现有回链与参考

unbounded `glyphs/measured_glyphs/face_resolutions/font_asset_faces/offline artifacts`回链PERF-MVP-228/Text09；per-glyph face/variation parse回链PERF-MVP-240/235；分离handle resolve回链PERF-MVP-246；fallback候选回链PERF-MVP-247。UE Slate把SDF atlas data同时缓存到shaped glyph和全局glyph key，并以request/respawn维护resident data；Fyrox长期保存page pixels/texture/modified flag，仅新glyph修改page。两者都不在稳定帧重建全部atlas。

## 责任计划与验收

Text05收到`failure-2026-07-18-sdf-frame-multipass-and-full-atlas-rebuild.md`与`failure-2026-07-18-sdf-source-identity-and-generation-not-batched.md`。1/100/10k glyph稳定300帧记录render-scalar Vec/key String、prepare passes、bitmap clone bytes、page alloc/zero/copy/nonzero scan、upload、vertex buffer create、Face/hash/manifest/I/O/worker queue与CPU p50/p95；稳定帧以上应为0或按changed glyph近O(delta)。需通过SDF/MSDF/MTSDF、offline/dynamic fallback、variation/TTC/CJK/emoji、generation reload、device loss、Softbuffer/WGPU/RenderDoc像素与current-source Cargo。完成前24/24仍留pending。
