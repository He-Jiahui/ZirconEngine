---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - project-library atlas resolver test
  - current-source Windows Cargo pending
  - multi-manifest lookup/hot-reload/cache-bound tests pending
  - 1/100/10000 atlas icon filesystem/decode/upload trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor sprite atlas逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`sprite_atlas.rs`、`sprite_atlas/**`与`sprite_atlas_tests/**`共 **9/9** 个Rust文件、**262** 行已逐文件阅读。覆盖source-key mapping、project-library discovery、manifest cache/validation、texture decode、UV投影与resolver fixture。当前源Cargo与动态trace未完成，因此仍留在`pending.md`。

## 热点与根因

PERF-MVP-180：`resolve_editor_sprite_atlas_image`位于template image paint路径。每次调用先由`atlas_manifest_candidates`从source ancestors找assets root，分配candidate Vec，`fs::read_dir`整个atlas cache目录、分配PathBuf并sort。每个candidate进入cache前仍执行path canonicalize与metadata modified/length；cache hit返回完整`SpriteAtlasAsset::clone`，包含entry Vec与字符串。Entry命中后`load_atlas_rgba`每次执行`image::open(...).into_rgba8()`解码整张atlas，再把owned RGBA放入draw payload。

Cache使用`BTreeMap<path+mtime+len, Option<manifest>>`且无容量/淘汰；manifest每次修改都会留下旧key。Resolver在第一个有效manifest中找不到entry时使用`?`直接返回None，不会继续后续candidate。上述同步文件系统、锁、manifest clone与图片解码全部发生在paint consumer，不应通过增加paint线程局部cache继续扩散owner。

## 计划与验收

Editor10的project-library/file watcher随asset generation一次构建immutable`source key -> atlas resource handle + UV` index，manifest/texture change精确发布新generation并淘汰旧记录。Render13 registry持有一份decoded CPU atlas和GPU texture；EditorUI08 compiled segment只消费handle/UV。Stable 1/100/10,000 icons的read_dir/canonicalize/metadata/read/parse/decode/manifest clone均为0；首次/changed generation每manifest parse≤1、每atlas decode/upload≤1，cache entries/bytes有上限。补multi-manifest later-candidate、mtime/length变化、删除/坏manifest、device loss与hot reload测试，保持resource key、UV、dimensions及GPU/Softbuffer pixels一致。
