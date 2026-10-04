---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: navigation-bake-selection-operation-arguments
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_plugins/05
plan_link_mode: child_record_only
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

## 2026-09-11 lifecycle scope correction

The original handoff omitted the required `plan_link_mode: child_record_only`
frontmatter field. The coordinator therefore imported an empty `related_code`
projection even though the handoff listed the four affected source paths. This
metadata-only correction restores the canonical child-record lifecycle scope and
does not change production source, acceptance criteria, or the historical
validation result. The source repair and static contract evidence remain
historical; managed Windows Cargo validation is still blocked by the foreign
RuntimeHost `WorldQueryResult::TransformSnapshot` exhaustiveness error above.

## 2026-09-11 local validation continuation

The exact selected-surface static reproduction completed with
`navigation selected-surface route projection: PASS` (`surface_entity` and
`force_full_rebuild` projections present). The repository
`validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild
-VerboseOutput` then produced no additional output for a bounded multi-minute
window and was interrupted with wrapper exit code 1. This is recorded as a
local timeout/interruption only, not as managed validation evidence; the open
failure and its required Windows Cargo gate remain unchanged.

## 2026-09-25 successor current-source reconciliation

- Coordinator successor `failure-roll-01a084c8-plugins05-navigation-bake-selection-r3` replaced the stale Plugins05 operation-status primary only after its zero-lease lifecycle was archived. A malformed intermediate registration was cancelled before any lease or source mutation; no unrelated operation-status receipt was absorbed.
- Current-source probe passed `EDITOR05_NAV_BAKE_CURRENT_SOURCE_PASS=6/6`: both selected-surface routes carry `surface_entity` from `selected_row_identity`, the bake route carries `force_full_rebuild`, the panel owns stable row/entity selection and disables selected actions without a row, the operation factory rejects missing `surface_entity`, the retained regression tests cover A/B/no-selection/refresh behavior, and the generic retained dispatcher has no Navigation route special case. The route-only reproduction passed `NAVIGATION_SELECTED_SURFACE_ROUTE_PROJECTION_PASS=6/6`.
- Existing repository contract `python -X utf8 -m unittest tools.tests.test_navigation_editor_contract -v` passed `Ran 1 test ... OK`. This is static/Python evidence only; no Cargo result is inferred. `rustfmt --edition 2021 --check` over the claimed Rust files exited 0. Scoped `git diff --check` exited 0 with only normal LF-to-CRLF warnings.
- Current seven-path manifest SHA-256 values are: `docs/plans/zircon_plugins/05-navigation.md=53ec3dc3b482e074216148376fd528a41be1194110a6efd27775a90f12247abf`; `zircon_plugins/navigation/editor/src/operation_command/factory.rs=c1bdd2277f028c68cec3874245852ab97000d6b46f2b7f9514647d99c804f796`; `zircon_plugins/navigation/editor/src/bake_panel.rs=247b3de5ea37140e4762d6ee68f73c5e35f8dcc73c20261bf75476775ab084c0`; `zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs=4a011f05fa865ada7b8da518b3c749233abe6335e237d72307520a46219e8f66`; and `zircon_plugins/navigation/editor/src/tests/bake_panel_retained.rs=ddb879d93e22b9cd4bcbff57a037eaef6018614be858b2d0add3aec428bab0ce`. The claimed `bake.zui` is a pre-existing dirty owner overlay (`cde2954aa86316f64b3bea0166ac0b8e96d249ef284b012d522f9a2fa2bbab3c`); `zircon_plugins/navigation/editor/src/tests/operation_command.rs` is a separate stale-owner dirty path and is intentionally not absorbed by this lifecycle. This reconciliation made no source edits.
- Dynamic acceptance remains pending: the exact selected-surface route test, Navigation editor package gate, and upward Editor03/Plugins05 gates must execute under Windows managed Cargo with `--locked` and coordinator-assigned target. The previous package job stopped in foreign RuntimeHost `WorldQueryResult::TransformSnapshot` exhaustiveness before Navigation tests; external dirty `E:/Git/zr_vm` remains a blocker. Independent C/I/M review, canonical `fixed-*` return, and closeout remain open.

## 2026-09-25 static validation ticket receipt

- Coordinator accepted static source-contract ticket `4451db8601b54fe9a9ad57729f048537` (request `6f6fedf18aa34f7bb7786274311affe1`) against the seven-path manifest sealed by snapshot `3862`. The inline command emits `EDITOR05_NAV_BAKE_CURRENT_SOURCE_PASS=6/6` and `NAVIGATION_SELECTED_SURFACE_ROUTE_PROJECTION_PASS=6/6`; `coverage.fullCoverage=false`, `staticParseOnly=true`, and Cargo/Rust are disabled.
- Admission returned `status=queued` with `executionKind=pending`; no Navigation Cargo package, exact filter, or test body has run. Dependency admission reports the open Editor05 overlay wiring and downstream Plugins05/Editor gates. This receipt is queue evidence only and does not satisfy dynamic acceptance.

## 2026-09-25 independent static review

- Reviewer `review-editor03-gizmo-private` re-read authoritative snapshot `3862`, the seven-path manifest and the static ticket receipt. It confirmed both 6/6 markers, the Navigation editor contract test (`1/1`), rustfmt exit 0, scoped diff-check exit 0, the dirty `bake.zui` provenance, and the intentional exclusion of the stale operation-status test owner.
- Independent review result: `Critical=0`, `Important=0`, `Moderate=0`. This is static handoff approval only; exact Windows managed Cargo, Navigation package/upward gates, external dependency cleanup, canonical `fixed-*` return, and closeout remain pending.
