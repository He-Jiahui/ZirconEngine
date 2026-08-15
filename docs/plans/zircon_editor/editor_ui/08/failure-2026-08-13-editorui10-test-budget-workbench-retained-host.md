---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-workbench-retained-host
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
related_code:
  - zircon_editor/src/tests/host/binding_dispatch/
  - zircon_editor/src/tests/host/pane_presentation/
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_toolbar_breakpoints/mod.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_state/
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents/
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover/
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/
  - zircon_editor/src/ui/host/play_pending_decision/tests/
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib structure_convention --locked
---

# EditorUI08: workbench and retained-host test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：这些测试覆盖 retained host、native workbench、shell、template runtime 与 workbench projection 的行为；
  不能由 EditorUI10 的通用 audit 或其他 UI component plan 吸收。

## 失败现象与复现证据

2026-08-13 的初始结构审计报告 21 个 EditorUI08 行为 owner 超过 800 行，0 个豁免：

- `tests/host/{binding_dispatch,pane_presentation}.rs`；`tests/host/manager/{bootstrap_and_startup,ui_asset_reference_and_promotion}.rs`。
- `tests/host/retained_callback_dispatch/template_bridge/{workbench_toolbar_breakpoints/mod,workbench_window_menus}.rs`；
  `tests/host/retained_menu_pointer/visual_screenshot/blend_space_workspace.rs`。
- `tests/host/retained_window/{native_host_contract,native_material_painter_mui_primitives,native_workbench_reference,shell_window}.rs`。
- `tests/host/template_runtime/{component_showcase_state,pane_body_documents}.rs`；
- `tests/ui/boundary/workbench_projection_cutover.rs`；
  `tests/workbench/layout/editor_layout_contracts.rs`。
- `ui/host/play_pending_decision/tests.rs`；
  `ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs`。
- `ui/retained_host/app/tests/drag_sources.rs`；
  `ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs`；
  `ui/retained_host/ui/tests/{component_showcase,host_scene_projection}.rs`。

其中 `native_host_contract.rs` 为 3123 行，是初始清单中最大的 test owner；它、
`shell_window.rs`、`native_workbench_reference.rs`、`native_material_painter_mui_primitives.rs`、
`blend_space_workspace.rs`、`workbench_window_menus.rs`、`drag_sources.rs`、`host_scene_projection.rs` 与
`ui_asset_reference_and_promotion.rs` 均已在本 failure 的前向修复切片中完成硬切。随后
`bootstrap_and_startup.rs`、`workbench_projection_cutover.rs` 与 `editor_layout_contracts.rs` 也已删除 flat owner；
当前 EditorUI08 剩余 4 项、全局剩余 31 项（2026-08-14 当前结构审计；仍以最新审计为准）。
不得用 blanket exemption 或提高预算隐藏其余债务。

## 最低共享层根因

workbench 和 retained-host 的跨行为回归持续追加到 flat owner 文件，未按 native host、shell/window、
menu/callback、template runtime、scene projection 和 pending decision 建立 folder-backed 子模块。通用 audit
此前也没有零容忍 test budget gate，因而没有阻断该扩张。

## 架构修复验收

- 每个列出的 owner 必须按行为拆为同名目录和薄 `mod.rs`，不超过 800 行；测试函数、fixture 与断言语义必须保留。
- 目录职责必须保持单一：native host/shell、menu/callback、template runtime、scene projection、pending decision
  不得重新聚合进新的综合文件。
- 不得留下旧 flat 文件、`#[path]` mount、compatibility re-export 或重复测试树。
- `audit_editor_structure.py --json` 必须不再报告上述路径；所有 48 项清零后，受管
  `cargo test -p zircon_editor --lib structure_convention --locked` 才能转 GREEN。

## 禁止临时方案

- 不得申请目录/glob/blanket test-budget exemption，不得提高 800 行限制或弱化 EditorUI10 Rust gate。
- 不得借拆分删除 native-host、workbench、pending decision 或 retained-host 的覆盖。
- 不得修改无关 EditorUI06/11 component 或 ZUI 测试以完成本交接。

## 修复结果与回传

Open state: `待修复`。`native_host_contract.rs` 已前向拆为 `native_host_contract/` 下的
`chrome_pointer`、`template_input`、`drag_projection`、`viewport_hierarchy_menu` 与唯一 `support`；
43 个测试函数保留，所有新文件不超过 800 行，结构审计已不再报告该 3123 行最大项。随后
`shell_window.rs`、`native_workbench_reference.rs`、`native_material_painter_mui_primitives.rs`、
`blend_space_workspace.rs`、`workbench_window_menus.rs`、`drag_sources.rs`、`host_scene_projection.rs` 与
`ui_asset_reference_and_promotion.rs` 也已硬切；当前其余 12 个 EditorUI08 owner
尚未处理，因此本 handoff 不能作为 EditorUI08 milestone 或 EditorUI10 根 failure 的 fixed evidence。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 test-budget forward handoff | `open` | 从准确的 48/0 结构审计中隔离 21 个 EditorUI08 workbench/retained-host 行为 owner；最大 `native_host_contract.rs` 为 3123 行。 | 获得业务源码 exact lease 后按行为 folder-backed 拆分，重审计并在全局计数归零后进入受管 structure gate。 |
| 2026-08-13 | native host contract folder-backed split | `resolving_failure` | 在 exact lease `c96286077c7b4281938c101f8c1624dc` 下删除 3123 行 flat owner，迁移为 4 个行为测试模块（43 个测试）与唯一 543 行 support；`rustfmt --edition 2024 --check --config skip_children=true` 通过，结构审计由 48/0 变为 47/0。 | 受管 Rust test；随后继续拆 EditorUI08 余下 20 项。 |
| 2026-08-13 | native host contract independent second review | `completed` | 独立复审 `C/I/M = 0/0/0`：删除前基线与迁移后均为 43 个 `#[test]`（13/8/10/12），无缺失、无新增、无重复；目录由 retained-window 根正确挂载，旧 flat 文件不存在，未发现 `#[path]` 或 `pub use` compatibility，六个新文件均不超过 800 行。 | 此局部 owner 已完成实现与二审；等待其受管 Rust gate。EditorUI08 handoff 仍为 `open`，剩余 20 项不得误标 fixed。 |
| 2026-08-13 | native host contract managed Rust gate admission | `pending_environment_repair` | 已以 7-path SHA-256 source manifest 申请受管 `cargo acquire test`，目标为 `E:/cargo-targets/zircon-editor/editorui10-native-host-contract-20260813`，过滤器为 `native_host_contract`；协调器在创建 job 前拒绝，原文为 `Coordinator-managed work is blocked until unregistered D/E/F artifacts are removed`。未生成 job、未启动 Cargo。 | Coordinator owner 必须先修复全局未登记 D/E/F artifact 准入；随后重新创建不可变 manifest 的受管测试。此环境失败不影响本地 43-test 映射、format 和二审证据。 |
| 2026-08-13 | shell-window folder-backed split | `resolving_failure` | 在 exact lease `b62974f18f03469a95e46c3dd14235cb` 下删除 1465 行 flat owner，迁移为窗口生命周期、场景快照、模板绘制、运行时渲染、指针回调 5 个行为模块及唯一 494 行 `support`；原始与迁移后均为 16 个 `#[test]`，定向 `rustfmt`/`diff --check` 通过，结构审计由 47/0 变为 46/0。 | 独立二审；随后 EditorUI08 仍剩 19 项，受管 Cargo 准入沿用 coordinator 环境修复后的新 manifest。 |
| 2026-08-13 | shell-window independent second review | `completed` | 独立二审 `C/I/M = 0/0/0`：16 个基线 `#[test]` 与五个行为模块完整一致（4/2/5/4/1），旧 flat 文件删除、retained-window 挂载正确、support import/helper 均为 `pub(super)`，无 `#[path]` 或 `pub use` compatibility，全部文件不超过 800 行。 | shell-window 局部实现与二审完成；EditorUI08 handoff 仍为 `open`，剩余 19 项与受管 Rust gate 不得跳过。 |
| 2026-08-13 | native-workbench-reference folder-backed split | `resolving_failure` | 在 exact lease `30403e400b434273995e831a108f50f7` 下删除 1204 行 flat owner，迁移为 reference surface、文本/模块输入、下拉指针、下拉键盘、菜单键盘 5 个模块及唯一 211 行 support；迁移前当前树已有一项 foreign `native_workbench_pointer_move_queries_the_hit_index_once`，故以 `HEAD` 24 项加该预存项为 25-test 基线。当前迁移后 25/25 映射一致，定向 `rustfmt`/`diff --check` 通过，结构审计由 46/0 变为 45/0。 | 独立二审；EditorUI08 剩余 18 项。foreign 命中索引测试仅被保留，不归入本 folder split 功能实现。 |
| 2026-08-13 | native-workbench-reference independent second review | `completed` | 独立二审 `C/I/M = 0/0/0`：HEAD 的 24 个和 current source 预存的第 25 个 `#[test]` 全部保留，五模块映射为 5/9/5/2/4；旧 flat 文件删除、正确挂载，support imports/constants/helpers 全为 `pub(super)`，无 compatibility，最大模块 323 行。 | native-workbench-reference 局部实现与二审完成；EditorUI08 handoff 仍为 `open`，剩余 18 项与受管 Rust gate 不得跳过。 |
| 2026-08-13 | native-material-painter folder-backed split and review | `completed` | 在 exact lease `1abccc89340347d0a283dbb193251548` 下删除 984 行 flat owner，迁移为 progress/overlay、field/icon、avatar/badge/chip、MUI X foundation、MUI X chart/chat 5 个模块和唯一 188 行 support。28 个 `#[test]` 映射完整一致（9/5/6/5/3）；定向 `rustfmt`/`diff --check` 通过，独立二审 `C/I/M=0/0/0`，support imports/constants/helpers 均为 `pub(super)`，无 compatibility。结构审计由 45/0 降为 44/0。 | 此局部 owner 已完成实现与二审；EditorUI08 handoff 保持 `open`，剩余 17 项与受管 Rust gate 不得跳过。 |
| 2026-08-13 | Blend Space workspace complete folder hard-cut | `resolving_failure` | 在 exact lease `08dc450f52944c7c862449d4bd931691` 下删除 807 行 flat root；目录完整挂载原有 6 个专项模块和本次 workspace/timeline/details/responsive/capture 5 个行为模块。旧根 6 个 `#[test]` 映射为 6/6，最大文件 679 行，定向 `rustfmt`/`diff --check` 通过、无 `#[path]`/`pub use`。截图调用方在 exact lease `98554d41205745638ab9fcb5af47605c` 下改为调用 folder 内 `support::blend_space_window` 和 `visual_capture::assert_*` 的真实实现；原有 foreign `composite_contracts.rs`/`toolbar_surface.rs` 只被保留挂载、未改逻辑。结构审计由 44/0 降为 43/0。 | 独立二审须确认完整 module tree、跨 sibling visibility 和截图调用；EditorUI08 handoff 仍为 `open`。 |
| 2026-08-13 | Blend Space workspace independent second review | `completed` | 独立二审 `C/I/M=0/0/0`：flat 基线 6 个 `#[test]` 完整迁移（2/1/1/1/1），六个既有 sibling 与五个新行为模块均只挂载一次；截图调用继续通过 `pub(in super::super)` 的真实实现进入，未发现 `#[path]`、`pub use` 或非限定公开边界。修复 support 的重复 sibling mount 后，定向 `rustfmt --edition 2024 --check` 与预算静态合同 5/5 均通过。 | Blend Space 局部实现与二审完成；结构审计保持 43/0，EditorUI08 handoff 仍为 `open`。 |
| 2026-08-13 | workbench-window-menus folder hard-cut and independent second review | `completed` | 在 exact lease `8a90e34f391843e2962ee89d6a89122f` 下删除 flat `workbench_window_menus.rs`，迁移为 main-menu bindings、asset creation、toolbar interactions、toolbar anchor 四个行为模块和唯一 support；HEAD 基线 14 个 `#[test]` 与迁移后精确一致（4/4/4/2）。独立二审 `C/I/M=0/0/0`，template bridge 正确挂载目录，support imports/helpers 均为 `pub(super)`，无 `#[path]`/`pub use`，最大模块 264 行。结构审计由 43/0 降为 42/0。 | 菜单 owner 局部实现与二审完成；EditorUI08 handoff 仍为 `open`，剩余 15 项和受管 Rust gate 不得跳过。 |
| 2026-08-13 | retained app drag-sources folder hard-cut and independent second review | `completed` | 在 exact lease `c2e487c3e4a84819ab0a90aabfa30d06` 下删除 895 行 flat `drag_sources.rs`，迁移为 scene/object、asset metadata/field、asset browser 三个行为模块及唯一 78 行 catalog fixture support。HEAD 基线与当前均为 16 个 `#[test]`（8/4/4）；独立二审 `C/I/M=0/0/0`，父 tests root 正确解析同名目录，无 `#[path]`/`pub use`/宽松 `pub`，最大模块 279 行。定向 `rustfmt`/`diff --check` 和预算静态合同 5/5 通过，结构审计由 42/0 降为 41/0。 | drag-sources 局部实现与二审完成；EditorUI08 handoff 仍为 `open`，剩余 14 项和受管 Rust gate 不得跳过。 |
| 2026-08-13 | retained UI host-scene-projection folder hard-cut and independent second review | `completed` | 在 exact lease `f85021f73acc4d2db0013b2e417b7978` 下删除 986 行 flat owner，迁移为 base scene、editor panes、workbench panes 与 assertions；唯一 HEAD 基线 `#[test]` 的断言与顺序完整保留。独立二审 `C/I/M=0/0/0`，同名目录由 UI tests root 正确挂载，无 `#[path]`/`pub use` compatibility，最大文件 429 行；定向 `rustfmt`/`diff --check` 通过，结构审计由 41/0 降为 40/0。 | host-scene-projection 局部实现与二审完成；EditorUI08 handoff 仍为 `open`，剩余 13 项和受管 Rust gate 不得跳过。 |
| 2026-08-13 | UI asset reference and promotion folder hard-cut and independent second review | `completed` | 在 exact lease `5e5a0d809a4e4b69a54a9f78cb5f3648` 下删除 801 行 flat owner，迁移为 reference navigation、tree/component transforms、component promotion 三个行为模块；HEAD 基线 8 个 `#[test]` 精确保留为 3/3/2。既有 `theme.rs` 的 4 项 foreign tests 仅作为同目录 sibling 保留、未吸收其改动。独立二审 `C/I/M=0/0/0`，目录 root 正确挂载四个实际 child，无 compatibility 或宽松公开边界，定向 `rustfmt`/`diff --check` 通过，结构审计由 40/0 降为 39/0。 | manager owner 局部实现与二审完成；EditorUI08 handoff 仍为 `open`，剩余 12 项和受管 Rust gate 不得跳过。 |
| 2026-08-13 | manager bootstrap and startup folder hard-cut | `completed` | 在 ownership transfer `545e2ca2edf38bc7d20d2d069ea111202707a95aab6dc1cad595bb07d96a7878` 与 exact leases `70929c66940d4571a69248cba2d7f965`/`a2699066f9c546d4b976c247eddd7231` 下删除 821 行 flat owner，迁移为 global layout、window topology、workspace restore、session startup 四个行为模块和唯一 26 行 module root。HEAD 基线 15 个 `#[test]` 与迁移后精确一致（2/4/2/7）；所有新文件不超过 293 行，定向 `rustfmt --edition 2024 --check`、scoped `git diff --check` 通过，旧 flat root 不存在且无 `#[path]`/`pub use` compatibility。独立二审 `C/I/M=0/0/0` 复核了完整映射、模块可见性和候选目录自包含性。 | 局部实现与二审完成；以不可变 validation copy 的受管 Cargo 精确运行验证。EditorUI08 handoff 保持 `open`，其余 11 项不得跳过。 |
| 2026-08-13 | workbench projection cutover folder hard-cut | `resolving_failure` | 在 ownership transfer `01557cf5ef447f25d062860530662461995b18f538a5ff8b83b82590af5d19da` 与 exact leases `fdb592a3dce24c8e8f2852a46ddcfdfc`/`4548513873a5470cac3252534a8d7b42` 下删除 904 行 flat owner，迁移为 template/reflection、ZUI prototype store、hit contract、layout frames、DTO boundary 五个行为模块和 103 行 module root。HEAD 基线 10 个 `#[test]` 与迁移后计数一致（1/1/1/6/1）；所有新文件不超过 287 行，定向 `rustfmt --edition 2024 --check`、scoped `git diff --check` 通过，旧 flat root 不存在且无 `#[path]`/`pub use` compatibility。结构审计将全局超限从 38 降至 37。 | 独立二审；随后以不可变 validation copy 的受管 Cargo 精确运行验证。EditorUI08 handoff 保持 `open`，其余 10 项不得跳过。 |
| 2026-08-13 | workbench projection cutover independent second review | `completed` | 独立只读复审 `C/I/M=0/0/0`：HEAD 10 项测试在新目录精确保留（template/reflection 1、prototype store 1、hit contract 1、layout frames 6、DTO boundary 1）；`boundary/mod.rs` 正确且仅一次挂载目录，旧 flat 文件不存在，无 `#[path]`、`pub use`、forwarding alias 或新增公开面。定向 `rustfmt`、`git diff --check` 与结构静态核验均通过。 | 局部实现与二审完成；仍需不可变 validation copy 的受管 Cargo。failure 总状态保持 `open`，不能用此记录替代余下 owner。 |
| 2026-08-14 | workbench layout contracts folder hard-cut | `resolving_failure` | 在 exact leases `f1de0c31d741465c81e317278b483de6`/`76e31400438b49cfbc13425d7bf2cca2` 下删除 949 行 flat owner，迁移为 region contracts、layout commands、breakpoints、geometry 四个行为模块及 30 行 module root。HEAD 与当前均为 23 项 `#[test]`，精确映射为 4/5/6/8；五个新文件最大 286 行，无旧 flat 文件、`#[path]`、`pub use` 或 migration compatibility 词命中，定向 `rustfmt --check` 与 `git diff --check` 通过。 | 独立二审；随后在不可变 validation copy 上运行受管 Cargo。当前 failure 仍为 `open`，余下 9 项不得跳过。 |
| 2026-08-14 | workbench layout contracts independent review correction | `completed` | 首轮独立只读审查为 `C/I/M=0/1/0`：active `related_code` 仍指向已删除 flat 文件；路径前向更新为活跃目录后复审为 `C/I/M=0/0/0`。HEAD/current 23 项测试逐体等价，目录挂载正确，三处移动后的 `include_str!` 解析到同一资源，root 仅保留 `pub(super)` 资产常量，无 `#[path]`、`pub use`、alias/shim/facade 或新增公开面。 | 局部实现与独立二审完成；仍需不可变 validation copy 的受管 Cargo。failure 总状态保持 `open`。 |
| 2026-08-14 | pane presentation folder hard-cut | `completed` | 在 ownership transfer `0dd0439f9c2a1e40529681593280ca0bd0c6c007df0d6b1b0d383edb4d43a6ab` 与 exact leases `9ba0267f63a941eab7291ac37bc5f390` 下删除 1025 行 flat owner，迁移为 first-wave payload、active debug snapshot、inspector customization、shell contract、document projection 五个行为模块与唯一 438 行 fixture support。HEAD/current 均为 5 个 `#[test]`，迁移后精确保留；所有新文件不超过 438 行，无旧 flat 文件、`#[path]`、`pub use`、alias/shim/facade 或 `include!`。定向 `rustfmt --check`、scoped `git diff --check` 与边界静态扫描通过；结构审计由全局 36 项降为 35 项。独立审查首轮为 `C/I/M=0/1/1`，前向修正记录基线和行尾空白后复审为 `C/I/M=0/0/0`。 | 局部实现与独立二审完成；仍需不可变 validation copy 的受管 Cargo。failure 总状态保持 `open`，余下 8 项不得跳过。 |
| 2026-08-14 | binding dispatch folder hard-cut | `completed` | 在 ownership transfer `82e05538576d62dcd47286239c4b7be4f1310d8edc61870ca78cd336316598c7` 与 exact leases `ab9bcd3c3bf445df863d3ca7d27ab2eb`/`cf33a62a289345e2ad7092c8e1acee74` 下删除 1258 行 flat owner，迁移为 inspector、docking、viewport、animation、selection、asset/welcome 六个行为模块与唯一 61 行 support。HEAD 的 21 项和当前工作树既有的 5 项测试均保留，迁移后共 26 个 `#[test]`，精确映射为 8/3/5/3/1/6；所有新文件不超过 577 行，无旧 flat 文件、`#[path]`、`pub use`、alias/shim/facade 或 `include!`。定向 `rustfmt --check`、scoped `git diff --check` 与边界扫描通过；完成时的结构审计由全局 35 项降为 34 项，随后其他 forward repair 已将当前全局值推进至 33。独立审查首轮为 `C/I/M=0/1/0`，前向修正 current-count 记录后复审为 `C/I/M=0/0/0`。 | 局部实现与独立二审完成；仍需不可变 validation copy 的受管 Cargo。failure 总状态保持 `open`，余下 6 项不得跳过。 |
| 2026-08-14 | pane body documents folder hard-cut | `completed` | 在 ownership transfer `7c5300b31e8ef91535d6e6e5ca720ce133fb69d1b7c87eb588a1f9f9ce25cc16` 与 exact leases `11da067e567d4ae88f4607a33605d343`/`b670ca2c950c4203b2f9725216d519a5` 下删除 884 行 flat owner，迁移为窗口文档、component showcase、runtime-v2 fixtures、host projection、pane registry 五个行为模块与唯一 336 行 support。HEAD/current 的 7 个 `#[test]` 精确保留为 2/2/1/1/1；既有 `asset_contracts.rs` sibling 保留为独立 4-test owner，根模块直接挂载它并删除旧 `#[path]` 属性。独立审查首轮 `C/I/M=0/2/1` 暴露 sibling 的旧 `use super::*` 和 support 缺失 registry import；已前向收敛为显式私有 imports 并补齐函数签名类型，记录行数同步为 HEAD 实际 884，复审 `C/I/M=0/0/0`。所有本次新文件不超过 336 行，无旧 flat 文件、`#[path]`、`pub use`、alias/shim/facade 或 `include!`。 | 局部实现与独立二审完成；随后通过不可变 validation copy 运行受管 Cargo。failure 总状态保持 `open`，余下 5 项不得跳过。 |
| 2026-08-14 | component showcase state folder hard-cut | `completed` | 在 ownership transfer `617069d7f58aeeec3f1da41fbbf32eb16d9e986708b1da965ba841a9eeb9b81d` 与 exact lease `e939610276ca44dfa8e3149ea7e4ebf4` 下删除 1078 行 flat owner，迁移为基础 bindings、完整 component actions、collection projection、context menu、复杂 runtime events 五个行为模块与唯一 13 行 binding helper support。HEAD/current 的 6 个 `#[test]` 精确保留为 1/1/1/2/1；所有新文件不超过 522 行，无旧 flat 文件、`#[path]`、`pub use`、alias/shim/facade 或 `include!`。独立审查首轮 `C/I/M=0/0/1` 指出 support 的实际行数为 13；已前向更正记录，复审 `C/I/M=0/0/0`。定向 `rustfmt --check`、scoped `git diff --check` 与边界扫描通过；结构审计由全局 33 项降为 32 项。 | 局部实现与独立二审完成；随后通过不可变 validation copy 运行受管 Cargo。failure 总状态保持 `open`，余下 4 项不得跳过。 |
| 2026-08-14 | play pending decision folder hard-cut | `completed` | 在 ownership transfer `03c24c3377c5d01b4e8966cd364ccf6bbcb479d34cfef6f03c981f138144f40b` 与 exact lease `8094d3dd45c843ba9d8fe5b18b2552c1` 下删除 830 行 flat owner，迁移为 publication、receipt consumption、expiry recovery、republish/contracts 四个行为模块及唯一 64 行 support。迁移前当前工作树为 16 个 `#[test]`（含相对 HEAD 的 10 个预存前向覆盖）；历史 `resolved_receipt_can_republish_a_pending_decision_after_execution_is_rejected` 已由当前源中更严格的 `consumed_receipt_can_republish_a_still_pending_decision` 行为取代，未回滚或重引入旧语义。目录迁移后 16 项当前测试完整保留为 5/3/4/4，最大文件 269 行；7 个 `include_str!` 均指向原有资源。独立审查首轮 `C/I/M=0/2/0` 指出未显式使用 Rust 2024 格式门禁且普通 diff 未覆盖 untracked modules；已前向收敛为 `rustfmt --edition 2024 --check`、逐文件 no-index 空白检查和目录边界检查均通过（仅 Git CRLF 规范化提示），复审 `C/I/M=0/0/0`。结构审计确认全局超限测试从 32 降为 31、migration debt 为 35。 | 局部实现与独立二审完成；随后通过不可变 validation copy 运行受管 Cargo。failure 总状态保持 `open`，余下 4 项不得跳过。 |
