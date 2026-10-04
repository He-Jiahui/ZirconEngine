---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_runtime/text/09
reference_sources:
  - dev/slint/internal/core/textlayout/sharedparley.rs
  - dev/slint/internal/renderers/software/fonts/vectorfont.rs
  - dev/slint/internal/renderers/skia/font_cache.rs
  - dev/godot/servers/text_server.cpp
tests:
  - existing paint_text layout, placement, raster, clipping, blending and latest-crop tests
  - current-source Windows Cargo pending
  - 1/1k/10k glyph layout/cache/lock counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor paint geometry/text逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`paint_geometry.rs` + `paint_geometry/**`共 **4** 个Rust文件、**124** 行；`paint_text.rs` + `paint_text/**`、`paint_text_tests.rs` + `paint_text_tests/**`共 **30** 个Rust文件、**5,547** 行。两组已逐文件阅读 **4/4、30/30**。现有测试覆盖layout、placement、clip、blend、raster和latest-crop正确性，但当前源Cargo、规模计数、锁争用和产品trace未完成，因此两组仍留在`pending.md`。

## 已有正确边界

Paint geometry是小型纯函数边界，frame/rect/pixel坐标转换不持资源或全局状态，未发现需要独立整改的热点。Text路径已有glyph raster key、subpixel bin、font fallback、recording command与software pixel测试；clip外文本和空文本会提前退出。这些行为是后续收敛必须保持的正确性基线，不能用删除fallback、禁用平滑或跳过复杂文本来换性能。

## 热点与计划

- PERF-MVP-156：`draw_text_with_size_and_style_impl`在recording-only路径仍先生成完整layout/glyph，随后仅记录display text并丢弃glyph；runtime single-line继续叠加runtime layout、shape line与fontdue layout。`host_grapheme_advances`、`grapheme_positions`、`runtime_glyph_origin_x`和`shaped_grapheme_advances`又形成glyph×grapheme扫描，后者按glyph分配overlap Vec。目标是复用runtime text/09的ShapedRun/LayoutCache与单一resolved layout，以线性cluster merge同时生成advance/origin。
- PERF-MVP-157：`current_host_text_preferences`每次加`RwLock`并深clone三条family String；layout/style/request与`rasterize_cached_glyph`会在文本乃至glyph热路径重复读取。目标是带generation的immutable typography snapshot，每帧或command build只抓一次。
- PERF-MVP-158：每个新`HostTextFontRequest` cache miss都会`Database::new + load_system_fonts`，resolved font再`Box::leak`进无界全局map。目标是进程级FontDatabase、稳定face id、有界resolved-face cache与显式generation失效。
- PERF-MVP-159：glyph cache是无界全局Mutex，Swash ScaleContext由另一把全局Mutex串行，cache miss还能重复栅格同key。目标是按bitmap bytes加权的有界cache、分片或线程本地上下文、single-flight miss以及hit/miss/evict/wait/bytes指标。
- PERF-MVP-160：fontdue fallback保留8倍超采样bitmap，`draw_glyph_row`却在每次绘制为每个logical pixel重算浮点sample区间并循环降采样。目标是在raster miss时一次转为logical coverage，命中绘制只顺序读取alpha。

## 参考引擎约束

Slint的Parley路径把shaped paragraph放入`TextLayoutCache`，并随scale factor或component生命周期清理；software vector font以alpha-map字节数作为权重建立1 MiB thread-local CLRU，Skia font cache也用容量64的CLRU。Godot TextServer提供字体size cache与显式clear/remove生命周期。这些参考共同约束Zircon：缓存必须有owner、generation、容量和失效，不接受无界HashMap、`Box::leak`或consumer侧平行权威。

## 动态验收

对搜索框、树重命名、Console长行/CJK/emoji和1/1k/10k重复文本记录shape/layout/raster次数、glyph与grapheme访问数、cache hit/miss/evict/bytes、global-lock wait和主线程p50/p95。稳定generation同key每帧shape/layout≤1、系统字体扫描≤1/generation、第二次glyph draw无降采样工作；GPU recording、Softbuffer和截图像素、ellipsis、baseline、cluster命中与fallback必须等价。
