---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: ui-text-distance-field-effects-type-resolution
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_runtime/text/05
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
tests:
  - cargo test -p zircon_runtime --test ui_component_slot_layout_contract --no-default-features --features ui --locked --jobs 1 --no-run
  - cargo test -p zircon_runtime --test ui_component_slot_layout_contract --no-default-features --features ui --locked --jobs 1 -- --nocapture --test-threads=1
resolved_at: 2026-07-13
---


# Text 05：UI text distance-field effects 类型解析失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行切片：S15.5 Property Editor Row 代码审查跟进 / Runtime Slot layout token 与参数解析验证
- 修复责任计划：`docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md`
- 交接原因：最低失败点位于 Text 05 明确拥有的 `scene_renderer/ui/render.rs` 和 text-effects DTO 投影，不属于 Editor Layout 或组件实例编译器。

## 失败现象与复现证据

Layout 15 新增独立集成测试，用正常 Runtime library 验证组件 Slot placeholder layout 在继承前解析 component token / component parameter，并让 caller mount 的独立叶子保持最高优先级。旧实现的测试二进制已真实 RED：width stretch 为 `Some("$value_slot_stretch")`，期望 `Some("Stretch")`。实例展开器完成解析修复后，E 盘受管紧凑通道执行：

```powershell
cargo test -p zircon_runtime --test ui_component_slot_layout_contract --no-default-features --features ui --locked --jobs 1 --no-run
```

在进入测试体前稳定失败：

```text
error[E0425]: cannot find type `UiTextDistanceFieldEffects` in this scope
  --> zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs:74:30
error: could not compile `zircon_runtime` (lib) due to 1 previous error
```

默认 feature 的 D 盘 retained pool 还受 `os error 112`（磁盘空间不足）阻断；E 盘最小 feature 通道空间充足并把源码失败唯一收敛到上述 E0425。

## 最低共享层根因

`ScreenSpaceUiTextBatch.text_effects` 使用 `UiTextDistanceFieldEffects`，但当前 `render.rs` 的 import/定义范围中不存在该类型。Text 05 的 2026-07-13 SM4-M1-S1 记录声明已新增 public text effects DTO 并投影到 `ScreenSpaceUiTextBatch`，当前源码的消费端与 DTO 导出/import 边界尚未闭合。

## 架构修复验收

- 在 Text 05 唯一 DTO owner/导出链上恢复 `UiTextDistanceFieldEffects` 的明确类型来源；不在 `render.rs` 复制结构或添加本地替身。
- 上述 `--no-run` 命令 exit 0，并执行 `ui_component_slot_layout_contract` 为 `1 passed / 0 failed`。
- 重新执行 Layout 15 的 Runtime legacy Slot、UI v2 surface frame、Editor Property Editor Row 严格几何和 Blend Space upward gate。

## 禁止临时方案

- 不添加 alias、兼容 shim、静默 fallback、重复 DTO、测试专用 bypass 或调用点特例。
- 不删除/`cfg` 掉 `text_effects` 字段来绕过编译。
- 不弱化 Slot token/参数解析、caller leaf precedence 或 Editor 几何验收。

## 修复结果与回传

- 根因：ScreenSpaceUiTextBatch consumed the existing Runtime Interface UiTextDistanceFieldEffects DTO without importing it into scene_renderer/ui/render.rs, so normal Runtime library compilation stopped before the Layout15 Slot contract could run.
- 架构修复：Import UiTextDistanceFieldEffects from zircon_runtime_interface::ui::surface in the existing canonical surface DTO import; no duplicate type, alias, shim, cfg bypass, or Editor-local text path was added.
- 验证：Windows E retained pool 的 `ui_component_slot_layout_contract` 为 `1 passed / 0 failed`。Frameworks05 收束 plugin bridge 后，当前源码 `zircon_editor --lib --no-run` exit 0；3153-test 新二进制的 PropertyRow Runtime Text projection `1/1`、三行严格几何 `1/1`、完整 Blend Space `13 passed / 0 failed / 1 ignored`、ZUI governance `74/74`、三档截图路由 `1/1`。
- 回传：Text05 类型解析故障已修复并返回 Layout15；PropertyEditorRow 审查集成与完整 Editor 上行布局门禁均取得当前源码 GREEN。三档图仍只写入 `docs/tests/editor`，仓库 target 与 D/E/F Cargo targets 的匹配 PNG 均为 0。
