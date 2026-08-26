---
handoff_kind: failure
status: open
created_at: 2026-08-22
summary_slug: text03-compiled-binding-contract-compile
origin_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
fixing_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
origin_child_dir: docs/plans/zircon_runtime/text/03
fixing_child_dir: docs/plans/optimize/zircon_runtime/74
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/template/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/tests/asset_surface_index/binding_ownership_performance.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -SkipBuild
---

# Runtime74: compiled binding contract blocks Text03 lib tests

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 来源执行切片：Text03 managed Windows library-test acceptance
- 修复责任计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 交接原因：Runtime74 owns the compiled binding facade, runtime compiler and binding-ownership performance test that fail before the Text03 test executes.

## 失败现象与复现证据

Text03 ran the managed Windows library-test command on 2026-08-22 using coordinator job
`6dabb1edfc62419b84bfb9131394ba6b` in the shared D-drive test pool. Cargo reached
`zircon_runtime` lib-test compilation but exited 101 before executing
`text_oversized_run_keeps_one_logical_shaped_line`.

The first deterministic compiler failures were:

```text
error[E0432]: unresolved import `zircon_runtime_interface::ui::template::UiCompiledAssetId`
 --> zircon_runtime/src/ui/template/asset/compiler/binding_program.rs:6:35

error[E0425]: cannot find value `TARGET_BINDING_COUNT` in this scope
 --> zircon_runtime/src/ui/tests/asset_surface_index/binding_ownership_performance.rs:117:25
```

The same compile reported 20 errors overall (`E0061`, `E0063`, `E0282`, `E0425`, `E0432`, and
`E0599`) before any Text03 test body ran. The managed target was
`D:\cargo-targets\zircon-engine\pool\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`;
no C-drive or repository `target` output was used.

## 最低共享层根因

`UiCompiledAssetId` is canonically declared beside the other dense compiled binding IDs in
`zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs`, but the public
`zircon_runtime_interface::ui::template` re-export list omits it. Runtime74's
`2026-08-22-binding-reload-transaction.md` explicitly owns that compiled-IR contract, the runtime
binding compiler, and the binding-ownership performance test.

The performance test also declares `TARGET_BINDING_COUNT` inside
`compiled_binding_ownership_lookup_p95_beats_program_scan` while `sample_lookup` needs the value
outside that function. This is a Runtime74 test-owner scope error, not a Text03 layout condition.

## 架构修复验收

- Re-export the canonical `UiCompiledAssetId` from the interface template facade with the existing
  compiled binding ID types. Do not introduce a runtime duplicate, compatibility alias, or a
  renderer-local representation.
- Place the binding-ownership expected-count contract where both the ignored test and its helper
  can consume it, preserving the declared 4,096/16/128 workload rather than weakening its
  assertion.
- Diagnose and repair the remaining lib-test compiler errors under the same Runtime74 ownership
  boundary before declaring this handoff fixed.
- Re-run the exact managed Text03 command so Cargo reaches and executes
  `text_oversized_run_keeps_one_logical_shaped_line`.

## 禁止临时方案

- Do not introduce a duplicate `UiCompiledAssetId`, compatibility alias, or renderer-local ID.
- Do not weaken the 4,096/16/128 workload or accept a build that never executes the named Text03 test.
- Do not bypass the managed Windows validator with raw Cargo or absorb unrelated Text03 layout work.

## 修复结果与回传

Text03 remains `implementation_complete / resolving_failure / managed_validation_pending`. Its
layout p50/p95 runs and WGPU product framebuffer export must wait for this compile gate; a build
that does not execute the named text test is not acceptance evidence.
