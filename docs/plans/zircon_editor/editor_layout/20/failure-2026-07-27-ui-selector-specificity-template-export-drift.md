---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: ui-selector-specificity-template-export-drift
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor_layout/20-style-cascade-and-computed-style.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor_layout/20
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/template/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/v2/style.rs
tests:
  - cargo test -p zircon_editor --lib --locked commandlet
  - focused selector/template interface compilation and Layout20 cascade regressions
---

# Layout20: UiSelectorSpecificity template export compile drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：plugin-list commandlet current-source managed gate
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/20-style-cascade-and-computed-style.md`
- 交接原因：template selector specificity is a Layout20 public cascade contract. Runtime template compiler and v2 style resolver must consume the interface export rather than locally re-declare it.

## 失败现象与复现证据

The source-bound Plan08 reservation `80a650bfd267428b804a2b1732e2c798` ran job
`a6ceedf9324f4976b54a96f806f12992` / run `bbc3a39561d544329ea15aaf55fae384` in
`F:\cargo-targets\zircon-engine\pool\87d15ca5f29387ac5ee4477904317dc1cc3345eb14aae1810bbda08aa5f8a0ff`.
It naturally released at `2026-07-26T18:29:59Z` with `exit 101`, before commandlet tests
could run. Raw stderr reported two `E0432` errors: `style_apply.rs:6` and `ui/v2/style.rs:8`
could not import `UiSelectorSpecificity` from `zircon_runtime_interface::ui::template`.

The failure is source-time evidence only: after that run terminated, the active Layout20 session
hard-cut the public re-exports in `template/mod.rs` and `template/asset/mod.rs` at
`2026-07-26T18:33:31Z`. A fresh immutable-source gate is still required; the failed Plan08 run
cannot validate the later source.

## 最低共享层根因

`UiSelectorSpecificity` is authored in the template asset style contract, while the public
`ui::template` boundary was temporarily missing its canonical re-export. Two runtime consumers
therefore saw a private or absent type despite using the intended public module path.

## 架构修复验收

- Keep one public `UiSelectorSpecificity` export chain from the template asset owner through
  `zircon_runtime_interface::ui::template`; both runtime consumers compile through that path.
- Do not duplicate the selector specificity DTO, introduce a local fallback import, or make the
  runtime consumers depend on a private template child module.
- Run focused Layout20 selector/cascade regressions and a fresh immutable-source compile gate.
- Return to and rerun the original Plan08 commandlet command after the Layout20 result is accepted.

## 禁止临时方案

- Do not add a compatibility alias, duplicate specificity struct, or consumer-local selector
  arithmetic.
- Do not treat the pre-export `exit 101` run as evidence for the later source.

## 修复结果与回传

Open state: `Layout20 current-source repair awaiting managed validation`; no fixed return or
Plan08 commandlet pass is claimed.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-27 | Plan08 managed gate -> Layout20 export handoff | open | Preserved the terminal Plan08 run id, raw E0432 locations, immutable-source boundary, and post-run Layout20 source-time change. The originating commandlet tests did not execute. |
| 2026-08-13 | Layout20 current-source export audit | `open / implemented_static / validation_pending` | 当前源码仅在 `template/asset/style.rs` 定义 `UiSelectorSpecificity`，由 `template/asset/mod.rs` 和 `template/mod.rs` 逐层窄回导出；`style_apply.rs` 与 `ui/v2/style.rs` 均消费公开 `ui::template` 路径，没有 DTO 复制、私有 child import 或 consumer-local fallback。该结论来自源码静态审计；未运行受管 Cargo，未声称来源 commandlet 或 failure return 通过。 |
