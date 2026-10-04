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
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/placement.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw/layout/tests.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/text/model/mod.rs
  - zircon_runtime/src/text/layout/rich.rs
  - zircon_runtime/src/text/layout/rich/tests.rs
  - zircon_runtime/src/text/layout/rich/materialize.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/rich/bbcode_table.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_layout.rs
  - zircon_runtime/src/ui/text/layout_engine/measurement.rs
  - zircon_runtime/src/ui/text/rich_text.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 ui::retained_host::host_contract::paint_text_tests::retained_text_measure_selects_runtime_family_for_ui_and_code_faces -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 ui::retained_host::host_contract::paint_text_tests::retained_text_preserves_small_underscore_stroke_contrast -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 ui::retained_host::host_contract::paint_text::font::tests::runtime_font_request_generation_includes_face_role_and_runtime_generation -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 ui::retained_host::host_contract::paint_text::draw::layout::tests::layout_uses_runtime_artifact_faces_for_basic_text -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 text::layout::rich::tests::text_rich_line_materialization_borrows_source_and_uses_a_run_cursor -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 text::model::shaped_run::tests::shaped_lines_borrow_absolute_ranges_from_one_shared_source -- --exact --test-threads=1
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 -- --test-threads=1
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

### 2026-09-19 current-source static contract receipt

Coordinator-managed static ticket `757c977d485643aa87c92d0a07727e57` passed
with job/run `624b23a3893341c38d07c9351c27bcb2` /
`757c977d485643aa87c92d0a07727e57` (exit 0,
`EDITORUI03_RETAINED_TEXT_FAMILY_SUBPIXEL_SOURCE_CONTRACT_PASS`,
`CHECKED_PATHS=5`). The sealed source-manifest hash is
`137f00cd5d3234cc9b23de2dfabba21bed5cc89143aba0201e5f58856ccfe780` and
covers this failure record plus the retained paint-text family tests, runtime
font request owner, subpixel placement owner, and runtime-artifact layout tests.
The checker confirms preference-derived family/weight selection without the
retired `system-ui` assertion, runtime family and grapheme/contrast regressions,
and grayscale-versus-subpixel origin handling. This is static/parse-only
evidence; focused managed Editor tests, platform-font discovery, upward Editor
M1/14/15 gates, independent C/I/M review, canonical failure return, and
coordinator closeout remain pending. The failure stays `open`.

### 2026-09-21 independent source review (review-editorui03-retained-family-r1)

- The four attributed source owners were re-read against sealed manifest
  `137f00cd5d3234cc9b23de2dfabba21bed5cc89143aba0201e5f58856ccfe780` and matched:
  `paint_text_tests.rs` `e7e2c770f0e342529d7b702c0666287c3669d130a07af4dba3e210ca14ad8be7`,
  `paint_text/font.rs` `1d4e45a257d5d52f30e19b151ebd0511297f3827611f424a3f62b0aa2580b99f`,
  `paint_text/draw/placement.rs` `e00b6db199b5e098f4403b49315d1d686dd072c820edbcd864b87b818e603c12`, and
  `paint_text/draw/layout/tests.rs` `435dd60fa7bb11f63e3dc9f3e092553d37009f40b0994aabc9247ae181b1eb7c`.
  The failure document itself carries the expected receipt drift and was not treated as a
  source-owner mismatch.
- `font.rs` resolves UI, strong-UI, and mono faces from the current host preferences, carries
  the corresponding weights into runtime styles, and hashes both the resolved runtime-font
  generation and the complete request for cache identity. No retired `system-ui` alias,
  platform path, or hard-coded face is reintroduced. The focused tests cover family separation,
  preference mapping, style-face selection, and stable/distinct request generations.
- The retained paint-text tests exercise runtime measurement and ellipsis at grapheme boundaries,
  small-underscore contrast, crop-label ink spacing, runtime family/weight projection, and the
  runtime surface measurement owner. Layout tests require runtime artifact faces, valid raster
  face indices/physical ppem, finite glyph origins, and apply grayscale snapping before artifact
  projection. `placement.rs` keeps non-finite origins on the named fallback, rounds only the
  grayscale path, and preserves fractional origins for subpixel smoothing.
- Independent probes passed for preference-derived family/weight routing, generation identity,
  absence of the retired family assertion, grapheme and contrast guards, artifact-face metadata,
  and grayscale/subpixel origin behavior. Scoped Rust 1.94.1 `rustfmt --check` and
  `git diff --check` passed for all four attributed paths.
- Independent review result: `Critical=0, Important=0, Moderate=0`. This is source-contract
  evidence only. Managed Windows Editor focused tests, actual platform-font discovery and
  raster/subpixel output, Runtime rich-text/rich-link unique-export compilation, upward Editor
  M1/14/15 acceptance, canonical failure return, and coordinator closeout remain pending; no
  historical binary or unrelated transitive fixture evidence is promoted.

### 2026-09-25 current-source r2 reconciliation

- Successor Session `failure-roll-01a084c8-editorui03-retained-family-r2` owns this failure record only. The current retained paint-text and Runtime text/export chain is listed as exact existing files in `related_code`; no source path was edited or re-attributed.
- Current SHA-256 values are sealed for all 17 listed files: `paint_text_tests.rs` `e7e2c770f0e342529d7b702c0666287c3669d130a07af4dba3e210ca14ad8be7`, `paint_text/font.rs` `1d4e45a257d5d52f30e19b151ebd0511297f3827611f424a3f62b0aa2580b99f`, `paint_text/font/tests.rs` `6cbb4511d5d10dcbc3166f6d796ef4325ef73fbe9f1d8603d551cd02295fbd43`, `paint_text/draw/placement.rs` `e00b6db199b5e098f4403b49315d1d686dd072c820edbcd864b87b818e603c12`, and `paint_text/draw/layout/tests.rs` `435dd60fa7bb11f63e3dc9f3e092553d37009f40b0994aabc9247ae181b1eb7c`.
- Runtime owner hashes are `shaped_run.rs` `930cb6e3980a7c59236991ccbc368bce16e143d256d9e33bc5484f50098ee277`, `model/mod.rs` `7eb760f67becdcc638793794ca6f407c75a9719cdeee436dd80a8b80fe6c6f39`, `layout/rich.rs` `2c62090bf15cc2e20c7e37806fe453ea5d0403c1949360cb0692cfec45706275`, `layout/rich/tests.rs` `9249e65af31f6984174873b265967e1f73a451260c79c1abffb50bbb0c1ce5f1`, `layout/rich/materialize.rs` `c941eda6230392ed08f231a3ed25072743317bf57873a8103fa777c10d125d41`, `layout/mod.rs` `271849b177faddc6e597c0a9c54f14187827c9e44ae38e27aae3cd431d796118`, `text/rich/bbcode_table.rs` `73f1d08659a21bc3b97ea221e7d95f40f6c67aa3f987974552ac8ac53bde9077`, `text/mod.rs` `69ae0db7abce20ce43c069fc1e84f42015bc22c4c99ff0e04593773d048714ff`, `ui/text/layout_engine/rich_table.rs` `ae8fc62223d152d0a47c8fefb14d2e4baaef4aef1d9ea58453daf09b208cebd9`, `rich_layout.rs` `7d01791c6c4716f434e4352e7bd7fdedfe7377a85a01f9962d899b44efbf81d3`, `measurement.rs` `945fa2e69834139781664465c77c1dc537785705b97a804df8d44c4311b559a2`, and `ui/text/rich_text.rs` `a755e30b76d5cda8aec354d8fb40bd799a6220ddb9b37677c803093802da4253`.
- Existing foreign dirty edits are present in four of the five retained layout/font files and ten of the twelve Runtime chain files; `paint_text_tests.rs`, `text/rich/bbcode_table.rs`, and `ui/text/layout_engine/rich_table.rs` are currently clean. This session preserves that provenance and does not claim any source changes. The exact focused filters now resolve to actual test functions, while managed Cargo, platform-font discovery, rich-link export compilation, and upward Editor M1/14/15 gates remain pending.

### 2026-09-26 successor intake (failure-roll-01a084c8-editorui03-retained-family-r3)

- The stale r2 lifecycle was cancelled through the coordinator after its
  heartbeat expired with no active lease. Successor
  `failure-roll-01a084c8-editorui03-retained-family-r3` now owns only this
  failure record. Ownership transfer fingerprint is
  `a7f7ca51c57165489629b27f51843a4ed8368c3efd996ac7fb2335efa998cbd8`, and
  pre-review snapshot `3926` sealed the record at SHA
  `7fc203ae10fd66c260d0e50697d3b4cfbbed3661d57c05ec2c1a84bd5d0d6ec4`.
- All seventeen current source paths from the r2 source manifest were
  rehashed and still match their sealed values. Existing foreign dirty edits
  across retained paint-text and Runtime rich-text paths remain unclaimed and
  unedited; the coordinator lease covers only this failure document. No stale
  binary, compile failure, or prior source ticket is promoted to dynamic
  acceptance.
- The static ticket `757c977d485643aa87c92d0a07727e57` and prior review remain
  historical source-contract evidence. Fresh managed Editor/Runtime focused
  tests, platform-font discovery, rich-link export compilation, upward
  Editor M1/14/15 gates, independent review of this successor, canonical
  `fixed-*` return, closeout, and WeCom notification remain pending. The
  failure stays open.

### 2026-09-26 independent successor review (review-editor03-gizmo-private)

- Reviewer `/root/review_editor03_gizmo_private` rechecked successor snapshot
  `3927` (`63bc2f1604b46b44da3b76bf2c9537f5197b5109b7a4fa81169f85c9cfece99b`),
  stale-r2 cancellation, ownership-transfer fingerprint, all seventeen
  attributed source hashes, and the preserved foreign-dirty provenance.
- No stale binary, compile result, or historical ticket was promoted to
  acceptance. Managed focused Editor/Runtime tests, platform-font discovery,
  rich-link export compilation, upward Editor M1/14/15 gates, canonical
  `fixed-*` return, coordinator closeout, and WeCom notification remain
  pending; this review is source-contract evidence only.
- Independent review result: `Critical=0, Important=0, Moderate=0`. The
  successor remains open pending its managed validation and closeout lifecycle.
