---
related_code:
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/03-text-input-focus-and-ime.md
tests:
  - 5 tracked Rust files and 29 test definitions statically reviewed
  - 100 same-target mouse moves assert zero damage, component event, dirty, and rebuild
  - current-source Cargo and allocation/route counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI event routing tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/event_routing.rs`与`tests/event_routing/**` 5/5个tracked Rust文件、1,704行、29个测试。累计UI tracked source从488/783增至493/783。覆盖pointer press/capture/hover/scroll、component envelope、dispatch effect、keyboard/text/IME、host requests与invalid owner。

## 已有性能门禁

`repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface`在首次hover完成render-only rebuild后连续100次同目标move，逐次断言damage/component events为空、dirty flags为空且last rebuild report不变；press/focus/hover状态测试也断言dirty仅限render。这是高频输入正确的确定性下界，应保留并扩展到125/500/1,000 Hz产品trace。

根文件两个源码守卫覆盖PERF-MVP-283的unused hover clone与table scalar sort借用。其余用例仍默认物化route/result diagnostics、component envelope、applied/rejected effects与IME/string payload，并只核对结果数量/内容，不记录route/event clone bytes、effect owners、note String、binding probes或metadata mutation次数。

## 责任与验收

route diagnostics/effect owner/bounded input state继续回链PERF-MVP-293/294/297和EditorUI01；editable text/IME多property mutation与正文复制回链PERF-MVP-295和EditorUI03。1/16/64 depth、1/4/100 handlers与100k move/text/IME事件需记录route visits、clone/alloc bytes、diagnostic entries、effect owners、dirty nodes和rebuild；same-target move保持零damage/dirty/rebuild，release默认full diagnostics为零。current-source Cargo与F4产品trace完成前保持pending。
