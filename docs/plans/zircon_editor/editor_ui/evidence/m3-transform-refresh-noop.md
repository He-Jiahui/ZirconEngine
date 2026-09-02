# M3 证据

- Gate: design-ready
- Owner session(s): current session
- Changed scope: Inspector Transform 连续编辑在规范化后的 axis value 未变化时直接返回 handled，避免重复 row string 组合、property mutation、dirty rebuild 和 projection refresh。
- Manifest: `docs/plans/zircon_editor/editor_ui/manifests/m3-transform-refresh-noop.yaml`
- Commands actually run: `git diff --check -- zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/transform_edit.rs`; `python -m unittest tools.tests.test_editor_zui_edit_control_index_contract tools.tests.test_editor_zui_product_interaction_event_contract tools.tests.test_editor_zui_inspector_layer_interaction_contract`
- Result summary: 16 tests passed；typed transform commit contract 保持不变；同值编辑不进入 retained surface refresh。
- Repaired failures: none
- Deferred external checks: Windows managed Rust tests、真实 pointer drag/keyboard repeat profile、full/patch GPU parity。
- Evidence links: `zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/transform_edit.rs`; `zircon_editor/src/ui/workbench/reference/template_surface.rs:211`。
- Unlocks: M4 Penpot parity 与产品壳验证。

## 性能边界

本切片位于 dirty rebuild 之前。旧路径即使输入值未改变，也会写 axis property、重新格式化 row value，并调用 `refresh_after_state_change`；新路径在 axis prefix 规范化后比较当前值，稳定输入成为 O(1) no-op。它不创建第二份 cache，也不改变 generation、layout 或 render owner。

这属于 Slate attribute minimal invalidation 在现有 retained bridge 上的直接应用。更大范围的 generation-owned render cache、SVG/text/GPU command cache 仍由其既有 Runtime/Editor owner 和并发切片负责，本记录不冒充其产品验收。
