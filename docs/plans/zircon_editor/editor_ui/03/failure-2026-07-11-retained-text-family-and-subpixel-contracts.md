---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: retained-text-family-and-subpixel-contracts
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_text
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/tests.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/layout/rich.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/rich/bbcode_table.rs
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo test -p zircon_editor --lib --locked ui::retained_host::host_contract::paint_text -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked ui::retained_host::host_contract::paint_text::draw::layout -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked core::jobs::tests -- --test-threads=1
---

# Editor UI 03：Retained text family 与 subpixel 合同失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 完整门禁；后续由 Plan 14 M1/M2 门禁追加相同最低层的 rich-text/rich-link 编译证据
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 交接原因：失败集中在 retained text family/subpixel 行为与 Runtime rich-text/rich-link 模块导出边界，最低共享原因归 Editor UI 03 文本与字体栈，而非 Editor kernel/jobs。

## 失败现象与复现证据

Editor M1 当前源码 08:31 binary 的全量门禁中，`ui::retained_host::host_contract::paint_text` 仍有 5 项失败，覆盖 runtime positioned glyph subpixel phase、grapheme advances、family 选择和 underscore stroke contrast。独立 exact `retained_text_measure_selects_runtime_family_for_ui_and_code_faces` 为 0/1（30.38s）：实际 family `DengXian`，旧断言要求 `system-ui`。

该聚类归 Editor UI 03 的 retained-text 适配/显示合同，而非 Plan 01 内核。后续必须判定当前 Runtime Text Discover 后返回具体系统 face 是否为新合同，并修 family 语义或更新已退役抽象名断言；禁止硬编码平台字体、恢复旧 font fallback 或削弱 subpixel/contrast 产品断言。

Plan 14 M1/M2 的后续编译门禁又分别暴露 rich layout re-export 与 `ui/surface/input/rich_link.rs` 的 surface/private rich-text import 漂移；详细命令、lane 与错误码见下方产出记录。

## 最低共享层根因

当前最窄已证实边界是 UI 03 文本栈的唯一模块导出与 retained/runtime 消费合同尚未收束：一侧仍断言退役抽象 family 名，另一侧仍引用已移动或私有化的 rich layout/rich-text/surface 路径。需要由文本 owner 选择并公布唯一新路径，再同步所有消费者；不能通过上层兼容 re-export 维持旧模块树。

## 架构修复验收

- rich layout、rich text 与 rich link surface 类型从唯一公开 owner 导出，旧路径和兼容 re-export 为零。
- focused retained paint-text family/subpixel 测试通过，平台字体选择合同不靠硬编码系统 face。
- `cargo test -p zircon_editor --locked --no-run --message-format=short` 不再出现本记录的 E0432/E0603/E0282，随后向上重跑 Editor M1 与 Plan 14 门禁。

## 禁止临时方案

- 禁止恢复旧模块、增加 alias/compat re-export、硬编码平台字体或在 Editor jobs 调用点复制类型。
- 禁止删减 family/subpixel/contrast 产品断言，或用 test-only bypass 隐藏导出漂移。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor UI 03 / Editor M1 | Retained text family/subpixel upward regression | `未通过-5项待功能owner处理` | 2026-07-11 | 完整门禁 5 项 paint_text 失败；family exact 0/1，`left=Some("DengXian")`、`right=Some("system-ui")`。Runtime HUD glyph exact 已单独 1/1，因此本记录只接管 Editor retained-text family/phase 合同。 |
| Editor UI 03 / Editor M1 | 当前源码完整门禁复核 | `未通过-失败集合未变化` | 2026-07-11 | 08:31 当前源码 binary 完整执行 2930 项为 2763 passed / 133 failed / 34 ignored（2258.13s）；与 06:17 门禁逐项比较，133 个失败名 added=0、removed=0。本计划 5 项归属不变；同一 binary 的 family exact 0/1（30.38s），仍为 `DengXian` 对旧 `system-ui` 合同。 |
| Editor UI 03 / Plan 14 M1.1 | Runtime rich-text 导出编译阻塞 | `未通过-待功能owner收口` | 2026-07-11 | Plan 14 focused GREEN 第二次执行在编译 `zircon_runtime` 时被 UI 03 文本布局阻塞：`rich.rs` 无法从 `core::framework::render` 导入 `LaidOutLine/LaidOutText/LayoutItem`，且 `graphics::text::layout` 未导出 `measure_text_source_range_width_with_provider`（E0432 共 2 项）。日志：`E:/cargo-targets/zircon-editor-assets-content-scroll-hover-validator-0710/editor-jobs-m1-green-2.log`。该失败与 `core/jobs` 无关，禁止在 Plan 14 增加旧导出兼容层；UI 03 owner 应完成新模块路径的唯一导出/调用点收口后回传，再向上重跑 Plan 14。 |
| Editor UI 03 / Plan 14 M2 | Runtime rich-link surface/import 编译阻塞 | `未通过-待功能owner收口` | 2026-07-11 | 受协调 `cargo build -p zircon_editor --locked` 已通过，但随后 test target 编译在 `zircon_runtime/src/ui/surface/input/rich_link.rs` 报 E0432（`super::surface` 不存在）、E0603（`rich_text` 私有）与派生 E0282。短日志 `.codex/tmp/plan14-m2-editor-test-compile.stderr.log`，lane `E:/targets/zircon-engine/lanes/test-e1b65c1437b84754a6cf511a1b948fba`。最低共享原因仍归 UI 03 的 rich-text/rich-link 模块导出与 surface 边界；禁止在 Plan 14 恢复旧路径或添加兼容 re-export。 |
| Editor UI 03 / Editor03+08 M1 | 当前全量门文本与 glyph 回归复现 | `未通过-继续由功能owner处理` | 2026-07-12 | Windows 受管 job `520d85713df249afae31661a7697ad07` 已完成 Editor lib-test 编译并进入全量执行；`render_frame_submission_hud_text_renders_through_runtime_glyph_capture` 以及 retained `paint_text` family/phase/contrast 组再次失败。该轮全量共观察到 178 个失败名，随后 harness 资源停滞而人工终止；原始日志 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`。命令注册与事务内核相关测试已通过，本组仍只归 Text/Font owner，禁止在 Editor03/08 降低文本断言或恢复旧字体兼容标识。 |
| Editor UI 03 / Editor14 M2 | RichTable 唯一导出边界阻断线程合同复验 | `未通过-待功能owner收口` | 2026-07-12 | Editor14 定向命令 `cargo test -p zircon_editor --lib --locked --jobs 1 core::jobs::tests::thread_ownership_contract -- --test-threads=1 --nocapture` 未进入测试体：`zircon_runtime/src/ui/text/layout_engine/rich_table.rs` 与 `graphics/text/rich/bbcode_table.rs` 从 `core::framework::render` 导入不可访问的 `RichTable/RichTableCell/RichTableColumn`，产生 E0432 及 3 个派生 E0282，共 5 个编译错误。受管 job `3908ff20340a4e1f8e12e9a062ec6f59` 因登记 wrapper PID 先退出被 coordinator 标为 orphaned，但真实 Cargo 随后自然以编译失败结束；原始 stderr `D:/cargo-targets/editor14-thread-guard-20260712.err.log`。最低原因仍是 UI03 富文本表格类型唯一 owner/导出路径未收束，Editor14 不增加兼容 re-export。 |
| Editor UI 03 / Editor15 M1 | 当前 editor binary retained paint-text 精确分片 | `未通过-2项仍待功能owner处理` | 2026-07-12 | `paint_text::{font,raster,blend,sync}` 分片分别 13/13、18/18、4/4、1/1；`paint_text::tests` 15/17，失败精确收敛为 family `DengXian != system-ui` 与小字号 underscore 最大亮度 88 不满足 stroke contrast。此前 5 项集合已有 3 项在当前源码通过，但本文件保持 open，禁止 Editor15 修改字体合同。 |
| Editor UI 03 / Editor15 M1 | retained text draw/layout 精确分片 | `未通过-3项 subpixel/grapheme 合同` | 2026-07-13 | `paint_text::draw::layout` 25/28；失败为 same-phase compact label visible drift、cumulative subpixel phase drift、runtime grapheme advances 数量 `6 != 2`。与同一文件既有 family/underscore 2 项合计仍是 Text03 接管的 5 项；Editor15 不改字体 fallback、glyph positioning 或栅格对比度。 |
| Editor UI 03 / Editor07 failure closeout | `ShapedGlyph.font_instance_id` retained-host fixture hard-cut | `未通过-待文本功能 owner 处理` | 2026-07-14 | 受管 Windows current-source 命令 `cargo test -p zircon_editor --lib --locked tests::editor_event::runtime::when_evaluation::typed_document_focus_tracks_floating_activation_and_focused_close -- --exact --test-threads=1 --nocapture` 未进入 Editor07 测试体：`paint_text/draw/layout/tests.rs:741` 构造 `ShapedGlyph` 时缺少 Text02 已定稿的 `font_instance_id: Option<InstancedFaceId>`，产生 E0063。完整日志 `.codex/tmp/editor07-focused-document-current-exact-r2-20260714.log`。文本 owner 应按 host-font fixture 的真实实例语义显式投影（非变量实例通常为 `None`），不得给 `ShapedGlyph` 恢复默认兼容构造器或在 Editor07 绕过 lib-test 编译。 |
| Editor UI 03 / Layout15 upward gate | `ShapedGlyph.font_instance_id` host fixture consumption | `该编译子项已修复-整体 handoff 仍 open` | 2026-07-14 | 按既有验收结论为非变量 retained-host glyph 显式投影 `font_instance_id: None`，未增加默认构造器、alias 或 test bypass；`cargo fmt -p zircon_editor -- --check` 与 scoped diff check exit 0。受管 Windows job `c1abbe55243c4b1689e30bc79256c81a` 已越过原 E0063 并继续编译 Runtime，随后被活跃 Runtime Text owner 的 `ResolvedScreenSpaceUiTextBatches::{native_texts,sdf_texts}` 私有字段 E0616 阻断。该新边界不回滚本 fixture 硬切；本记录原有 family/subpixel 5 项仍未完成，因此 handoff 不返回、不改 fixed。 |

## 修复结果与回传

- 状态：`open / 待修复`；先跑本组，再向上重跑 Editor M1。
