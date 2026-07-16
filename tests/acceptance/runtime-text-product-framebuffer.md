# Runtime Text Product Framebuffer

## Scope

- Shared system-font face byte materialization in `zircon_runtime/src/text/font/database.rs`.
- Direct system CJK face consumption by the production SDF atlas/render path.
- Runtime UI text layout, native/color/SDF batching, WGPU submission and framebuffer readback.
- CJK VerticalRl TTB/BTT shaping, vertical-origin placement, punctuation and real multi-column layout plus Latin, Arabic, Hebrew, Emoji, mixed BiDi and locale-sensitive Han rows.

## Baseline

- The existing Latin `ABCD` VerticalRl proof passed, but replacing it with `竖排布局` produced `node=110, changed=0`.
- The lowest shared defect was `FontDatabase::face_bytes(...)`: every `StoredFontSource::FontDb` face returned `FaceBytesUnavailable` even though the same face already had an authoritative backend ID.
- A diagnostic Swash coverage-to-SDF fallback was explored, then removed after the shared byte source was repaired; it is not part of the accepted architecture.

## Test Inventory

- `text_font_database_materializes_discovered_system_face_bytes` verifies Windows system face bytes and standalone SFNT parsing. The regression is present; its monolithic lib-test filter was not separately linked in this run.
- `sdf_font_bake_rasterizes_materialized_system_cjk_face` verifies direct `fontsdf` loading and nonzero CJK atlas pixels. The regression is present; its monolithic lib-test filter was not separately linked in this run.
- `export_runtime_multilingual_text_product_framebuffer_png` verifies ten text regions against an independently rendered background framebuffer; its VerticalRl command calls shared `layout_text`, requires exactly two right-to-left columns, then requires glyph deltas inside each column frame.
- Boundary coverage includes collection-face extraction, missing font bytes, empty/non-visible glyph rejection, locale-separated Han cache identity, complex Arabic marks, color RGBA glyphs and vertical height/width geometry.
- Failure behavior remains typed: unknown backend faces and unavailable bytes do not silently choose a second font database or renderer-local strategy.

## Tooling Evidence

- Build: `cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --no-default-features --features target-client --locked --jobs 1 --message-format short --color never --no-run`
- Build result: current-source target rebuilt in 13m24s; executable `runtime_text_multilingual_product_framebuffer-213fe7d8ff35ca45.exe`; 418 existing library warnings.
- Product run: exact ignored exporter, 1/1 passed in 99.85s (process wall time 103.2s).
- Visual inspection: original 1080×620 PNG inspected at full resolution; no tofu, overlap, blank row or policy text.
- Artifact scan: no same-name PNG under repository `target`, `E:\cargo-targets` or `F:\cargo-targets`.

## Results

- Artifact: `docs/tests/runtime/text/runtime_text_multilingual_product_framebuffer_20260710.png`
- Size: 100,139 bytes; 1,012 colors; SHA256 `AE10DC416FD87AC7382676796AF05FD3EE30C12B0904A482D225C3DB69F8D713`.
- Changed pixels by node: 3751, 3983, 1820, 2843, 3321, 3643, 4823, 1473, 1179 and 1789.
- CJK VerticalRl text: `竖排「标点」。第二列，验证。`; shared layout resolves two right-to-left columns.
- Two-column bounds: `(982,34)-(1049,273)`, 68×240, changed=4114. Right column changed=2548 with 30×240 extent; left column changed=1566 with 30×165 extent.
- zh-Hans/ja relative-region delta: 1613 pixels.
- Color Emoji face: `Segoe UI Emoji`, 9216 RGBA bytes.
- Arabic base plus fatha: two glyphs on one actual `Segoe UI` backend face.

## Acceptance Decision

Accepted for system-font materialization, direct CJK SDF bake, native `vmtx`, rustybuzz TTB/BTT, shaped glyph/face atlas identity, backend vertical-origin placement, CJK vertical punctuation, shared multi-column breaking, soft-wrap caret affinity and VerticalRl layout/hit-test/IME geometry. Horizontal cosmic per-run `locl`, variable-font axes, mixed-BiDi caret/range product corpus, platform candidate-window live QA, MSDF/MTSDF and full native/SDF paragraph parity remain open and are not implied complete by this acceptance.
