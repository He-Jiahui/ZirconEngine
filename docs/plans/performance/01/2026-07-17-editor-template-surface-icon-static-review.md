---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_asset_placeholder_visuals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_asset_placeholder_visuals/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_surface/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_color.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_color/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - asset placeholder/icon/pixel tests
  - icon glyph mapping/fallback tests
  - node surface/state-layer tests
  - style color projection tests
  - current-source Windows Cargo pending
  - stable-generation theme/icon/glyph/command counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template surface/icon逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_asset_placeholder_visuals*`、`template_icon_button_glyph_kind*`、`template_icon_button_glyphs.rs`、`template_node_surface*`与`template_style_color*`共 **12/12** 个Rust文件、**1,794** 行已逐文件阅读。覆盖asset thumbnail/preview、typed icon fallback、surface eligibility/state layer与typed role/color projection。当前源Cargo与产品规模trace未完成，因此仍留在`pending.md`。

## 已有正确边界

非asset visual、不可见尺寸、无preview与无icon时均可早退；真实preview替代semantic fallback icon而非叠加；node surface只生成有声明或交互状态的surface；asset name-area圆角补片与state layer有command/pixel测试。Geometry与role/color match为有界操作，没有paint内I/O、线程或队列。

## 热点与计划

每个asset thumbnail node都会读取metrics和palette，再按目标尺寸请求preview/icon pixels；稳定grid的theme lock、resource lookup/raster与owned image command由PERF-MVP-161、150和178共同收口。缺失icon的manual glyph fallback还执行`format!("{} {}", control_id, icon_name).to_ascii_lowercase()`，随后以约26种glyph的长`contains`链重分类。`SharedString=String`又使格式化输入与presentation clone成本归PERF-MVP-174。

Typed glyph identity应在template/presentation generation投影一次，compiled paint segment直接携带enum与resource handle。Stable generation不得重复string merge/lowercase、handler classification、theme获取、icon raster或surface command build；changed icon/theme/resource generation只失效对应segment。不得建立独立于Runtime09/EditorUI08 generation的glyph cache。

## 动态验收

在1/100/10,000 asset/icon nodes、resolved/missing/mixed icons、normal/focused/selected thumbnails上记录theme lock、resource cache hit/miss/raster/upload、formatted/lowercase bytes、glyph predicate probes、command build与CPU scope。Stable generation上述工作为0；单icon或theme变更只重建受影响segment；同resource generation raster/upload各至多一次。保持preview优先、semantic fallback、typed color、surface/state layer、clip/z/opacity与GPU/Softbuffer pixels一致。
