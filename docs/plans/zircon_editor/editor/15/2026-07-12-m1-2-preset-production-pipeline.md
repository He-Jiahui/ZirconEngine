# Editor 15 M1.2 preset 与生产导出流水线

## 范围

- `.zpreset` 严格版本壳、类型化 DTO、项目级存储与原子替换。
- retained export wizard 从独立 preset 名加载，解析 `profile_ref` 并校验 target mode 一致。
- 生产 CompileHost 硬切到 core `ZirconBuildStageExecutor`/`zircon_build.py`，PlatformBundle 接入
  core staged-layout validator；wizard 仅呈现 core plan 的拓扑顺序。
- resume 指纹绑定 expected output locator，并在跳过前复核源码、锁文件、构建脚本、工具链和
  staged artifact 内容摘要。
- Python `.zpreset` consumer 与 Rust DTO 对齐，拒绝未知字段、错误类型和旧式无版本壳 payload。

## 当前状态

M1.2 实现与复核整改已完成，并获得最终 `SPEC APPROVED` 与 `QUALITY APPROVED`。Navigation 05 的
`NavigationQueryFilter [f32; 64]` serde 阻断已由对应 owner 修复并通过 Editor lib check；记录已从
failure 迁移为 fixed。M1 统一测试仍需迁移旧 Cargo CompileHost report fixtures，因此尚未关闭。

跨模块修复记录：
`docs/plans/zircon_plugins/05/fixed-2026-07-12-navigation-query-filter-serde-array.md`。

Editor 15 测试硬切修复记录：
`docs/plans/zircon_editor/editor/15/fixed-2026-07-12-compile-host-report-test-hard-cutover.md`。

## 关键产出

- `zircon_runtime_interface/src/export/preset.rs`
- `zircon_editor/src/core/export/preset.rs`
- `zircon_editor/src/core/export/pipeline.rs`
- `zircon_editor/src/core/export/stages/`
- `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/`
- `zircon_editor/src/ui/retained_host/app/build_export_wizard_session/options.rs`
- `tools/zircon_export/preset_contract.py`
- `templates/projects/renderable-empty/export/desktop_windows.zpreset`
- `docs/zircon_editor/core/export/pipeline.md`

## 验证证据

| 验证 | 结果 |
| --- | --- |
| `cargo check -p zircon_runtime_interface --lib --offline` | 通过 |
| `cargo test -p zircon_runtime_interface --locked --offline` | 240/240 通过，doc-tests 通过 |
| `python -m unittest tools.zircon_export.tests.test_preset_contract tools.zircon_export.tests.test_pipeline_report_staged_compile_host` | 8/8 通过；含默认 Debug profile 被 release preset 覆盖的完整最终汇总回归 |
| `cargo check -p zircon_editor --lib --offline` | 通过；仅存在当前树既有 warning |
| `cargo test -p zircon_editor core::export::tests --lib --offline -- --nocapture` | 13/13 通过；Text 07 visibility failure 已迁移为 fixed |
| 受影响的 CompileHost/final-report/Validate/PlatformBundle Python test groups | 262/262 通过；旧 Cargo report fixtures 已硬切并迁移为 fixed |

## 产出记录与时间

| 时间 | 状态 | 产出 |
| --- | --- | --- |
| 2026-07-12 20:12 +08:00 | 实现完成，验收待上游 | 完成 strict `.zpreset`、原子替换、preset/profile 单一权威校验、production core executor 接线、输出身份与产物摘要恢复语义及 Rust/Python malformed contract tests；Editor focused test 的 Navigation 05 阻断已写入对应功能计划。 |
| 2026-07-12 20:45 +08:00 | 复核整改完成，Rust 复验中 | preset UI 改为 manifest profile 单一权威；ServerRuntime 指纹不再要求 Hub/Node；core 编排拆为独立模块；最终报告硬切 staged-build 参数并新增 release full aggregation test。Navigation fixed 已回填；旧 Cargo report tests 失败归档到 Editor 15 测试计划。 |
| 2026-07-12 21:05 +08:00 | M1.2 已完成 | 最终 `SPEC APPROVED`、`QUALITY APPROVED`；mode 权威收敛到 `ExportPreset.debug`，final aggregator 不再由旧 profile Cargo mode 否决 preset。M1 统一测试阶段保持独立未完成。 |
| 2026-07-12 21:18 +08:00 | 测试合同已修复 | 全部受影响 Python test consumers 完成 staged-build 硬切，四组 262/262 通过；对应 failure 迁移为 fixed。 |
| 2026-07-12 21:24 +08:00 | M1 Rust 测试被跨模块阻断 | focused test 编译返回 Text 07 rich-table provider visibility E0364/E0603；失败已写入对应功能计划，Editor 15 不添加绕过。 |
| 2026-07-12 21:42 +08:00 | 跨模块阻断已修复 | Text 07 owner 以受限可见性修复 E0364/E0603；runtime interface 全套 240/240、Editor core export 13/13 通过，对应 failure 迁移为 fixed。 |
| 2026-07-12 22:24 +08:00 | M1 统一测试仍未关闭 | Editor full lib 通过 Cargo 与 test binary 直接执行均在 904 秒无 summary，已写入本计划分区验证记录；Windows 平台策略又被 Render 18 SSS 测试 E0283 阻断，已写入 Render18 对应功能计划。 |
| 2026-07-12 22:44 +08:00 | Full gate 分片定位 | `core::` 分片 64/65；唯一失败为 Editor10 ProjectAuthority boundary guard 自扫描，exact panic 已追加到 Editor10 既有 failure。 |
| 2026-07-12 23:45 +08:00 | Full gate 继续分片并路由 | `scene::` 24/24；retained painter 28 项写入 Layout15；paint-text 2 项追加 UI03；component showcase 6 项追加 UI06；Welcome mount 1 项追加 UI08。Editor15 自有的 2 项 Build/Export pane fixture 已从旧 target 文本权威硬切为 typed `ExportWizardPanelViewModel`；共享编译拥塞导致新 binary 904s 未产出，保持待验证。 |
