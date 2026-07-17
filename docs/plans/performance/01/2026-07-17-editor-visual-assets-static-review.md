---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_images.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_images/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - runtime/editor-pages/MUI/template/SVG/tint visual asset tests
  - current-source Windows Cargo pending
  - cache hit/hot-reload/bounded-eviction tests pending
  - 1/100/10000 visual asset filesystem/copy/raster/upload trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor visual assets逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`visual_assets.rs`、`visual_assets/**`与`visual_assets_tests/**`共 **41/41** 个Rust文件、**1,720** 行已逐文件阅读；直接consumer `template_node_images.rs` + `template_node_images/**` **4/4** 文件、**257** 行也已完成。覆盖resource key、candidate/alias、PNG/SVG/MUI loading、tree/pixel cache、retained preview、tint/missing fallback、target sizing、node image geometry/identity/command及全部runtime/editor-pages/template tests。当前源Cargo与动态trace未完成，因此仍留在`pending.md`。

## 热点与根因

PERF-MVP-181：每次icon/image请求先分配cache key，再构造alias/source/MUI candidate PathBuf Vec并逐个`path.exists()`，之后才能形成包含display path/size/tint的pixel-cache key。Global `Mutex<BTreeMap<String, Option<HostPaintImagePixels>>>`没有容量或generation，hit通过`.cloned()`深copyRGBA、resource key和可选整atlas。SVG tree cache同样无界，且每个lookup先canonicalize/stat source；mtime变化留下旧key。Candidate/path work和文件系统因此在稳定cache hit也存在。

Retained preview每次`Image::to_rgba8`后复制bytes并对全图`DefaultHasher`；SVG intrinsic fallback会先渲染成Image、再复制到retained pixels；missing icon不进入pixel cache并重复O(width*height)栅格。MUI fallback在production resolver中从`dev/material-ui/.../*.js`同步read/parse/build SVG。Tint通过复制/逐pixel改RGB生成独立variant。以上工作都位于paint consumer，且cache不跟file watcher/theme/resource generation绑定。

Template-node consumer在image geometry不可见或clip miss时正确早退，但稳定可见node每次重新求tint、进入上述resolver并把owned image/atlas payload移入新command。该重复command/resource projection归PERF-MVP-178/181；consumer不应建立自己的pixel cache。

## 计划与验收

Editor10 registry随asset/file-watcher generation一次解析canonical resource与alias/MUI import；production不依赖`dev`源码runtime fallback。Render13按`(resource handle,generation,size,tint)`持有有界decoded/raster/texture variant，能在shader/command表达的tint不复制RGBA；SVG tree/missing icon在worker或import阶段生成。EditorUI08 compiled segment只clone轻量handle/UV/tint。

Stable 1/100/10,000 assets的candidate/path allocation、exists/canonicalize/stat、global cache lock、RGBA hash/copy、file read/parse/raster均为0；miss不阻塞paint；同variant decode/raster/upload≤1/generation，entries/bytes/eviction可观测且有硬上限。补hot reload、theme tint、negative-cache invalidation、MUI import、missing icon、retained preview与device loss测试，保持resource identity、target size、tint、UV及GPU/Softbuffer pixels一致。
