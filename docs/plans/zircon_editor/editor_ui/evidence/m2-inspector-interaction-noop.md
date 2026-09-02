# M2 证据

- Gate: design-ready
- Owner session(s): current session
- Changed scope: Inspector 搜索输入在 value 未变化时直接返回 handled，不再重复写 property、重建过滤投影或刷新 template surface。
- Manifest: `docs/plans/zircon_editor/editor_ui/manifests/m2-inspector-interaction-noop.yaml`
- Commands actually run: `git diff --check -- zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/inspector_filter.rs`; `python -m unittest tools.tests.test_editor_zui_inspector_filter_contract tools.tests.test_editor_zui_product_interaction_event_contract tools.tests.test_editor_zui_inspector_layer_interaction_contract`
- Result summary: 20 tests passed；Inspector 的 `Change` 与 `Submit` route 保持不变；同值输入进入 O(1) no-op，不产生重复 surface refresh。
- Repaired failures: none
- Deferred external checks: Windows managed Rust lib test、真实键盘输入和 IME 产品验证。
- Evidence links: `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/inspector_filter.rs`; `zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui`。
- Unlocks: M3 generation/cache/invalidation 性能切片。

## Slate/Penpot 对齐

- 对齐 Slate attribute minimal invalidation：只有属性值实际改变才触发后续 state/filter/surface 失效。
- 保留 Penpot 计划中的 transient edit / committed submit 区分；本切片不改变 `.zui` 的 `Change`/`Submit` route，也不引入直接 World mutation。
- disabled/read-only 与 selection source 仍由既有 typed state 决定。
