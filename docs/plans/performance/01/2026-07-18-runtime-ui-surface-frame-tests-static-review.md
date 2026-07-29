---
related_code:
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - 5 tracked Rust files and 9 test definitions statically reviewed
  - arranged/render/hit/focus/pointer authority parity covered
  - stable frame-copy and persistent-Taffy counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface frame authority tests逐文件性能静态审查（2026-07-18）

本批逐文件完整阅读`tests/surface_frame_authority.rs`与其目录5/5个tracked Rust文件、1,014行、9个测试。累计UI tracked source从525/783增至530/783。覆盖arranged/render/hit/focus/pointer统一authority、Taffy flex/wrap/grid、slot sizing和Zircon fallback。

测试中18次调用`surface_frame()`并逐层查询arranged/render/hit数据；当前API每次深clone frame payload，因此这些功能测试没有约束PERF-MVP-278的stable generation零复制。Taffy用例验证backend/fallback像素与route正确，但没有tree create、node insert/style/children set、compute或allocation计数，不能验收PERF-MVP-261 persistent Taffy tree。

EditorUI08需让frame发布generation-owned Arc artifact，1k stable accesses的tree/render/hit/String/ECS projection为零；EditorUI02需在100/1k/10k nodes稳定300帧记录Taffy create/insert/style/children/compute，stable create/insert=0。统一authority的frame/hit/pointer像素与route断言必须保持。current-source Cargo、规模counter、产品trace与像素完成前保持pending。
