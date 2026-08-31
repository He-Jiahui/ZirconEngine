---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: navigation-bake-selection-operation-arguments
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/src/operation_command/factory.rs
  - zircon_plugins/navigation/editor/src/bake_panel.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
tests:
  - powershell -NoProfile -Command "$zui = Get-Content 'zircon_plugins/navigation/editor/bake.zui' -Raw; if (($zui -match 'route = \"navigation.bake.surface\"') -and ($zui -match 'route = \"navigation.bake.clear_surface\"') -and ($zui -notmatch 'surface_entity')) { throw 'selected-surface routes have no surface_entity projection' }"
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild -VerboseOutput
---

# Plugins05：Navigation Bake 选择态与操作参数链路交接

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 待修复` | 2026-07-15 | Editor03 operation factory/runtime wiring 复核确认，Navigation Bake 的 `Bake Selected` 与 `Clear Selected` 按钮只提交 route，正常 retained-host 点击产生 `arguments = null`；两个 factory 均要求对象载荷中的 `surface_entity`，因此用户路径必然在命令创建阶段失败。缺口归 Plugins05 M6-T1：面板当前没有 surface 行数据、选择事件或“选择实体 -> typed operation arguments”投影，必须由 Navigation editor owner 完整实现。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M3.2 operation factory/runtime wiring 独立质量复核
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：Navigation editor 自有面板没有 surface 数据与选择投影，通用 Editor03 operation factory 无法构造领域实体参数。
- 生命周期键：`navigation-bake-selection-operation-arguments`

## 失败现象与复现证据

- `zircon_plugins/navigation/editor/bake.zui` 的两个 selected-surface 按钮只有 route，没有参数绑定。
- retained host 将空参数投影为 JSON `null`，非空参数投影为数组；它不会也不应猜测 Navigation 的当前 surface。
- Navigation factory 对 `navigation.bake.surface` 与 `navigation.bake.clear_surface` 都明确拒绝缺失 `surface_entity` 的请求。
- `NavigationBakeSurfaceList` 目前只有 `selected_index = 0` 静态属性，没有 surface 数据源、选择变更事件或稳定实体标识。

## 最低共享层根因

该失败不在 Editor03 通用 command factory：通用层已经忠实传递调用参数。最低缺失层是 Plugins05 Navigation editor 的 M6-T1 面板状态模型与绑定投影，尚未把运行时/场景中的 NavMeshSurface 列表、当前选择和按钮调用组合成 typed request。

## 架构修复验收

- Navigation editor 提供稳定的 surface 行模型，行必须携带实际 `surface_entity`，不得把显示索引当实体。
- 表格选择变更更新 Navigation 自有面板状态；无选择时禁用 selected-surface 命令或返回明确 UI 状态，不提交无效操作。
- `Bake Selected` 同时投影 `surface_entity` 与 `force_full_rebuild`；`Clear Selected` 投影 `surface_entity`，并以 factory 当前对象 schema 生成调用。
- retained-host 真实点击路径测试覆盖：选择 A 后 bake A、选择 B 后 clear B、无选择不提交、切换选择不复用旧实体。
- Plugins05 与 Editor03 的受管 Windows 包门通过后，按同一生命周期键回传 `fixed-*.md`。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止在通用 retained dispatcher 中按 route 名称特判 Navigation。
- 禁止默认为实体 `0`、首行索引或场景中任意 surface；禁止让 factory 在缺参数时静默改为 bake scene。
- 禁止只扩展数组解码而不实现真实选择态；按钮仍为空参数时该方案不构成修复。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.

## 2026-08-27 current-source validation continuation

The canonical static reproduction and the retained ZUI selection contract now pass:
both selected-surface routes project `surface_entity` from the table's stable
`selected_row_identity`, the bake route also projects `force_full_rebuild`, and both
commands remain disabled when no row identity is selected. `rustfmt --check` also
passes for `bake_panel.rs`, `bake_panel_retained.rs`, and the current shared
`operation_command.rs` blob.

Managed Windows job `0d26b703ac164fc082c9369ab38a7b6b` entered
`cargo test -p zircon_plugin_navigation_editor --locked` and was released normally
with wrapper exit `1` / Cargo exit `101`. Compilation stopped before the Navigation
editor tests at the foreign lower-layer error
`zircon_runtime_host/src/foreign_output/item_count.rs:80` (E0004): the current
`WorldQueryResult::TransformSnapshot` variant is not covered. No diagnostic names a
Navigation editor source. This is forward validation evidence only; the focused
Plugins05 test must execute successfully after the RuntimeHost owner repairs that
mixed blob, so this failure remains `open` and no fixed return is claimed.
