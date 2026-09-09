---
related_code:
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/theme/mod.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime_interface/src/ui/style.rs
implementation_files:
  - zircon_runtime/src/ui/style.rs
  - zircon_runtime/src/ui/theme/mod.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/style_mapping.rs
  - zircon_runtime/src/ui/tests/theme_registry.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime/runtime_pseudo_state.rs
doc_type: module-detail
---

# 样式与主题

## 级联模型

`StyleSheetScope` 持有 `UiV2ResolvedStyle`；`StyleField<P>` 可选择级联或节点 override。`resolve_property` 按 scope 从内到外查找，缺失时使用 `StyleProperty::default_value`。元素字段包括背景/前景/边框颜色、边宽、圆角、宽高和 opacity；opacity 最终限制在 0..1。

颜色支持 `UiStyleColor::Role`、`Rgba`、`inherit`、`transparent`、六/八位十六进制及 RGBA 数组。尺寸支持 auto、fill/full、数值和命名 style dimension。

## 按钮样式

`ButtonStyleFields` 解析 variant（text/contained/outlined）、color（primary、secondary、success、error 等）、size、icon placement、interaction state、loading 和 disabled，并合并元素字段。`resolve_button_style_from_values` 可直接从 TOML 值读取。

## 主题

`UiThemeRegistry` 保存活动 `UiThemeDocument` 及 fingerprint，解析 `$theme.palette.*`、`theme:` 和裸 role 名称。`apply_document` 返回 `UiThemeReloadOutcome`，调用方据 changed 决定重新解析静态样式。

```rust
let mut theme = UiThemeRegistry::default();
let outcome = theme.apply_document(new_theme);
let resolved = theme.resolve_role("$theme.palette.accent");
```

## 限制

未知 role 保留为 `Role`，不会静默变成黑色；运行时伪状态（hover、pressed、focused、disabled、loading）必须通过状态索引更新，否则样式缓存不会失效。
