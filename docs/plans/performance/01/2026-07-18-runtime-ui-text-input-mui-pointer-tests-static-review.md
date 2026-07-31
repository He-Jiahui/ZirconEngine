---
related_code:
  - zircon_runtime/src/ui/tests/widget_text_input_mui.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
  - zircon_runtime/src/ui/surface/input/editable_text
  - zircon_runtime/src/ui/surface/input/text_state.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 22 MUI/pointer text-input tests reviewed
  - alias query/value selection drag context-menu and IME parity present
  - retained state still uses generic metadata attributes
  - long-text 120-Hz layout/binding/diagnostic/popup counters and current-source Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI text input MUI/pointer测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{widget_text_input_mui,widget_text_input_pointer}.rs`，共2/2个tracked Rust文件、1,376行、22个测试。范围覆盖MUI InputBase/TextField/Textarea/SearchField/Autocomplete aliases、keyboard/IME/autofocus，以及pointer caret/selection/word/line/drag/secondary context menu/capture/disabled语义。

## PERF-MVP-295：测试名的retained state仍是TOML metadata

MUI测试多处称“retained edit state”，但fixture与断言仍直接读写`UiTemplateNodeMetadata.attributes`中的value/query/caret/selection/composition。产品每event从这些通用TOML值重建owned editable state，再把多个字段逐项写回；component alias分类也重复发生。EditorUI03必须建立真正persistent typed state，EditorUI06只在binding/serde边界投影MUI aliases。

## Pointer selection热路径缺口

press/drag/double/triple click需要text-layout hit、word/line range、capture、binding report和完整drag diagnostics；测试文本不足十几个字符、drag只有一次move。连续100k selection drag没有layout lookup、full extract、word/line/grapheme visits、正文 clone、transactions或diagnostic/effect bytes计数。PERF-MVP-296要求node→text layout近O(1)借用，PERF-MVP-293/294要求默认热路径不构造完整诊断/多份payload。

## PERF-MVP-297：context popup生命周期

secondary release为每node生成context-menu id并写popup stack，测试只开1个popup且不覆盖owner detach/window close/重复右键/容量。text selection与popup owner lifecycle必须共用bounded input state。

## 验收要求

1/10k/100k chars、1/1k controls、连续100k caret/drag/right-click/IME事件记录metadata/layout/full-extract/word-line probes、text/effect/diagnostic clone bytes、transactions、popup entries与CPU p95。per-event transaction<=1、caret-only正文clone=0、layout lookup近O(1)、diagnostics off完整notes=0、owner close后popup=0。current-source Cargo与F4 Inspector/Console产品trace完成前，2/2留在`pending.md`。
