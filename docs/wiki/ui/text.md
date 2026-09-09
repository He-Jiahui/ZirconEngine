---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/text/shaping/mod.rs
implementation_files:
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/ui/tests/text_layout/wrapping.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
doc_type: module-detail
---

# 文本系统

## Shaping 与字体

`text::shaping` 通过 Cosmic/字体集合执行脚本分析、双向 (bidi)、fallback span、emoji presentation、连字与 kerning。`TextShapeRunProvider` 以字体集合 snapshot 保证多行操作使用同一 revision；不支持的垂直模式返回 `UnsupportedWritingMode`。

## UI 文本布局

`layout_text` 将 shaped glyph run 排成段落和行框，支持水平/垂直方向、软换行、kinsoku、连字符、ellipsis、justify、tab、富文本 block/table 及 inline widget。`UiTextMeasureCache` 以文档 generation、样式和 viewport 为键缓存测量；`hit_test_text_layout` 与几何 API 提供字符位置、caret 和范围框。

```rust
use zircon_runtime::ui::surface::{layout_text, measure_text_size, text_caret_frame_for_layout};
use zircon_runtime_interface::ui::surface::{UiTextCaret, UiTextCaretAffinity};
let layout = layout_text(source, &style, text_frame, Some(clip_frame));
let caret = UiTextCaret { offset: byte_offset, affinity: UiTextCaretAffinity::Downstream };
let caret_frame = text_caret_frame_for_layout(&layout, &caret, source, &style);
let size = measure_text_size(source, &resolved_style);
```

## 编辑与安全文本

`UiTextEditState` 以 grapheme 边界维护光标/选择，支持撤销意图、词/行导航、剪贴板和 IME preedit。secure text presentation 在渲染前替换 glyph，但保留源范围与可访问语义；安全输入策略禁止将明文写入调试快照。

## 限制

范围 API 使用 UTF-8 字节区间，但所有编辑移动必须落在 grapheme 边界。字体集合发布会递增 revision，旧缓存自动失效；超预算 shaping 返回带 phase/dependency 的失败报告。
