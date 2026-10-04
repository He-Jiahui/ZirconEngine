---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-22
summary_slug: ui-asset-binding-canonical-loader-api-tests
origin_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/74
fixing_child_dir: docs/plans/optimize/zircon_runtime/74
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --locked --release --lib asset_binding -- --test-threads=1
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -VerboseOutput
---

# ui-asset-binding-canonical-loader-api-tests: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 来源执行切片：Render11 Shader06 realtime IBL managed library validation
- 修复责任计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Render11 Shader06 realtime IBL managed library validation` — Windows managed validate-matrix compilation of zircon_runtime with text_oversized_run_keeps_one_logical_shaped_line reports 13 E0599 errors in zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs because tests call UiAssetLoader::load_str while the canonical loader exposes load_toml_str, plus E0282 at lines 438 and 518 for untyped serialized.try_into().

## 最低共享层根因

The UI asset-binding test suite retained the pre-cutover generic loader spelling and ambiguous conversion inference after the canonical TOML-specific UiAssetLoader contract became the only production entry point.

## 架构修复验收

- Tests use the canonical TOML loader API and explicit compiled-program conversion type where required; no deprecated load_str compatibility method is introduced; the originating managed zircon_runtime validation advances past the reported UI binding E0599 and E0282 errors.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `static_ticket_passed_review_green_cargo_pending`; the coordinator must keep the
dynamic validation gates open and route this Plan to its managed Cargo and upward acceptance stage.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-24 | Runtime74 canonical UI test-consumer hard cut | `source_updated_static_green_cargo_pending` | Migrated all 18 `UiAssetLoader::load_str` consumers in `asset_binding/compiled_program.rs` to `load_toml_str`; annotated the two formerly ambiguous deserializations as `UiCompiledBindingProgram`; migrated `default_interaction_schema` from the removed `UiCompiledDocument::root` field to `template_instance().root`; and supplied the current four-argument `UiBindingMutationTransaction::commit` contract to its benchmark consumer. Static contract scan reports legacy loader calls `0`, canonical calls `18`, untyped corrupted-program conversions `0`; exact `rustfmt --check` passed. No Cargo or coordinator receipt was run, so this remains open and cannot be returned as `fixed-*`. |

## 2026-09-11 rolling repair admission

- Stable fixing session `failure-roll-01a084c8-runtime74-ui-asset-binding` owns this
  record and the canonical loader/test paths. Lease heartbeat and attribution were
  renewed at HEAD `c37155ba304740b3762b20585f77fb53a6da47fb`.
- Snapshot `3411` sealed the exact current bytes:
  `loader.rs` `a671259c79744b3dcb72bbd4cdc8868e559409635b0940c8034f5df7c5309e72`,
  `compiled_program.rs` `723f6ca50e029581047ea9846f71df0ef1866639547a81b3da543bd10bffbbff`, and this
  failure record `009bb3877b91437236db7711da415285e620072d570a33d13bab40bb94eb36d0`.
- Owner-focused request `runtime74-ui-asset-binding-20260911-r1` submitted
  `cargo +1.94.1 test -p zircon_runtime --locked --release --lib asset_binding -- --test-threads=1`.
  Admission rejected it before ticket creation with
  `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; no Cargo,
  dynamic test, upward acceptance, review, fixed return, or closeout evidence exists.
- The external repository was not modified. This lifecycle remains open and the
  session is waiting for a clean external validation state.

## 2026-09-21 successor metadata and formatting repair

- Successor Session `failure-roll-01a084c8-runtime74-ui-asset-binding-r2` received the failure
  record and both source paths through coordinator ownership-transfer fingerprint
  `dd8adca1460349736caa3affa9c1d5e4e767f81cef21ede7ff1c0bbfd4a51c06`. The preview reported all
  three paths eligible and preserved the archived stable owner's provenance.
- The frontmatter had encoded `loader.rs`, `compiled_program.rs`, and two historical line numbers
  as one semicolon-delimited `related_code` value. The record now contains two canonical repository
  paths and two explicit acceptance commands, so the coordinator index and later source manifests
  no longer treat the scope as one nonexistent path.
- A metadata contract first failed on the missing separate test path, then passed after the repair.
  Current source still has exactly 18 `UiAssetLoader::load_toml_str` consumers, zero legacy
  `UiAssetLoader::load_str` consumers, and five explicitly typed
  `serialized.try_into()` conversions to `UiCompiledBindingProgram`.
- An initial diagnostic mistakenly used Rust edition 2024 and reported edition-specific import
  ordering/assertion wrapping differences. The workspace declares edition 2021; authoritative
  `rustfmt +1.94.1 --edition 2021 --check` passes, and invoking that formatter preserved both source
  hashes byte-for-byte. Scoped `git diff --check` also passes. Managed Cargo and the originating
  Render11/Shader06 upward test remain blocked by the unchanged foreign `E:\Git\zr_vm` worktree and
  are not claimed by this metadata/static continuation.

## 2026-09-21 static ticket and independent review receipt

- Coordinator validation ticket `f5587c10895744f7804a670bcbce9389` reached terminal `passed` at
  `2026-09-21T17:33:11.046897+00:00`. Its immutable manifest hash is
  `2ff7ab86b7fdc233adcc82e3fd650d4c16a095d3098f839eddd05e32bdbd86c3`; coverage is deliberately
  partial (`fullCoverage=false`, `upwardAcceptance=false`) and proves only the current-source static
  loader/conversion and metadata contract.
- Independent review of snapshot `3737` returned Critical `0`, Important `0`, Moderate `0`. The
  reviewer rechecked the three sealed hashes, the two valid `related_code` paths, both acceptance
  commands, 18 canonical `load_toml_str` calls, zero legacy `load_str` calls, all five explicitly
  typed `UiCompiledBindingProgram` conversions, the absence of a compatibility alias/shim, edition
  2021 rustfmt, and scoped diff hygiene.
- This receipt does **not** claim either dynamic gate. The managed focused `asset_binding` Cargo test
  and the originating `text_oversized_run_keeps_one_logical_shaped_line` upward test remain pending;
  the foreign dirty `E:\Git\zr_vm` checkout is unchanged. Therefore no `failure return`, canonical
  `fixed-*`, closeout, commit, or notification is authorized yet.
