---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: runtime-operation-ffi-sibling-visibility
origin_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_runtime/text/03
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/tests/operation.rs
tests:
  - cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --locked --jobs 1 -- export_runtime_multilingual_text_product_framebuffer_png --exact --ignored --nocapture --test-threads=1
resolved_at: 2026-07-15
---


# Editor03: runtime operation FFI sibling visibility blocks runtime integration builds

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 来源执行切片：LB-M5-T / VerticalRl rich-inline paragraph product framebuffer exporter
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：最低原因位于 Editor03 当前未提交的 runtime operation dynamic-API split；Text03 不拥有 operation FFI visibility 或 API-table wiring。

## 失败现象与复现证据

Windows coordinator-managed GPU job `zircon_runtime` exact ignored exporter 在测试体前编译失败，未生成 PNG。`zircon_runtime/src/dynamic_api/session/operation.rs` 把 `submit_operation`、`poll_operation`、`harvest_operation` 声明为 `pub(super)`；`session.rs` 随后尝试 `pub(super) use operation::{...}`，而兄弟模块 `dynamic_api::exports` 再导入这三个符号。Rust 报 E0364 三项与 E0603 三项，因为子模块的 `pub(super)` 只到父 `session`，不能被重新导出到父模块之外的兄弟 `exports`。

完整复现命令：

`cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --locked --jobs 1 -- export_runtime_multilingual_text_product_framebuffer_png --exact --ignored --nocapture --test-threads=1`

同一当前源在此之前已通过 Text03 focused filter：5 passed / 0 failed；因此这不是 rich-inline layout assertion 或 WGPU readback defect。

## 最低共享层根因

Runtime operation FFI bridge functions 的 module visibility 与它们的真实 consumer boundary 不一致。`dynamic_api::exports` 是 sibling consumer，所需最小内部边界是 `pub(crate)`（或等价的 crate-internal owner visibility）；当前 `pub(super)` 无法穿过 `session` parent re-export。

## 架构修复验收

- 三个 operation bridge functions 在不公开为外部 API 的前提下可被 `dynamic_api::exports` 使用；不要新增第二套 wrapper 或移动 FFI panic guard。
- `zircon_runtime/src/dynamic_api/tests/operation.rs` 的 submit → poll → harvest 真实闭环通过。
- Text03 exact ignored WGPU exporter 编译并实际执行；只有到达真实 framebuffer readback 后才允许产出 `docs/tests/runtime/text/runtime_text_vertical_rich_inline_paragraph_product_framebuffer_20260715.png`。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken the runtime API table, skip operation modules, or make the Text03 exporter compile a reduced feature graph.

## 修复结果与回传

- 根因：Runtime operation FFI bridge functions were visible only to the session parent, while dynamic_api::exports is a sibling consumer.
- 架构修复：Keep one operation bridge owner and expose submit/poll/harvest only crate-internally with pub(crate); retain the existing session re-export and FFI panic wrapper without aliases or compatibility shims.
- 验证：Current-source Windows managed exact ignored runtime text WGPU exporter compiled the full zircon_runtime graph and passed 1/1 after framebuffer readback; output PNG was generated under docs/tests/runtime/text.
- 回传：Editor03 operation sibling visibility no longer blocks Text03 product framebuffer acceptance.
