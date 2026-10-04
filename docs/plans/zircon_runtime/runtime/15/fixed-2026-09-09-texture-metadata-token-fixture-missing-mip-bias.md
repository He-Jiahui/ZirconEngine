---
handoff_kind: fixed
status: fixed
created_at: 2026-09-09
summary_slug: texture-metadata-token-fixture-missing-mip-bias
origin_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/runtime/15
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/assets/texture/descriptor/tests.rs
tests:
  - asset::assets::texture::descriptor::tests::import_settings_parse_texture_metadata_tokens
  - asset::assets::texture::descriptor::tests::
  - runtime_15_texture_descriptor_settings_parser_is_child_owner
resolved_at: 2026-09-09
---

# Runtime04: texture metadata token fixture omits its asserted mip bias

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 来源执行切片：lower descriptor regressions for the descriptor-filter failure
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：Runtime04 owns the texture import fixture and parser behavior; the Runtime15 structure guard must retain its own separate source scope.

## 失败现象与复现证据

Managed Windows job `2b2e7158e28c4cbdaa15fff4c6517426` executed
`asset::assets::texture::descriptor::tests::` with static linkage, no default
features and locked dependencies: 26 passed, 1 failed, 0 ignored, 6823 filtered.
At `descriptor/tests.rs:400`, `import_settings_parse_texture_metadata_tokens`
expected `mip_bias` 0.5 but received 0.0. The test's TOML contains usage hint,
mip policy, normal convention and compression, but omits `mip_bias` entirely.
The parser already assigns the explicit setting through `f32_setting`; the
omitted setting correctly retains the default value.

Input: `E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime15-texture-descriptor-3298-20260909`,
10,968 files, manifest
`c78e80da5b1c6960c27dce48714dc3aa530c4dff210f3d027e39200202dfe579`.
Artifacts: `results/runtime15-texture-descriptor-lower-3298.{json,log}`.
The independent Runtime15 guard passed 1/1 as job
`31beb7da597f443d9165bf58af3e1dcd` on the same input.

## 最低共享层根因

The token parsing fixture asserts an explicit scalar override without supplying
that override to the production parser. This is a test-input inconsistency,
not evidence that the production default should change.

## 架构修复验收

- Supply `mip_bias = 0.5` and `max_anisotropy = 8` in this fixture and retain
  their exact scalar assertions, every metadata-token assertion and the
  production defaults.
- Pass the full 27-test descriptor module, including this exact test, on a
  managed immutable source input; retain the Runtime15 upward guard result
  against matching production and guard bytes.
- Return the source snapshot and actual receipt to the
  [origin lifecycle](../15/failure-2026-07-17-descriptor-filter-plan-anchor-loss.md).

## 禁止临时方案

- Do not change the default mip bias or anisotropy, weaken either expected
  value, ignore the failing test, or implement fixture-specific production
  behavior.
- Do not include Runtime04 fixture changes in a Runtime15 closeout candidate.

## 修复结果与回传

- 根因：The metadata-token fixture asserted mip_bias 0.5 and max_anisotropy 8 without supplying those TOML inputs. Managed jobs 2b2e7158e28c4cbdaa15fff4c6517426 and 11f08f2ba0b34ffb90f5544136bb1009 each executed 27 descriptor tests, with 26 passing and the original token test failing at the omitted input.
- 架构修复：Snapshot 3305 supplies exactly mip_bias = 0.5 and max_anisotropy = 8 in descriptor/tests.rs, hash dfa7019f6c3be58c11e6508d019ab1bdd534652fdbd34118da54dfd032ef86f7. Every existing scalar and token assertion remains. Production defaults and parser code are unchanged; pre-existing extent-contract changes remain preserved. The changed owner is the fixture file only.
- 验证：Managed Windows static no-default locked job 2c03b7c385ba4b5eb4e31dbec084ad19 passed all 27 descriptor tests, 0 failed, 0 ignored, including import_settings_parse_texture_metadata_tokens. Input runtime04-subasset-final-3305-20260909 manifest 3bfe12829402a3c0e72beaea5fdfff8d2d7c33125d97c8a8dc24f5b69899e15c; exact command and receipt are in results/runtime04-texture-descriptor-lower-3305-r1.json and its sibling log. Current production descriptor hash cc6f7cf54e5a1c0401bbaa369030dd344cc3c0f51325cc2a3a0be4d7e56c1766 and settings hash 5adefcfa9f5603b649a14b2605cb065c86758d459f74b8237a926269d0cdd5e4 match the input and the unchanged Runtime15 guard job 31beb7da597f443d9165bf58af3e1dcd, which passed 1/1.
- 回传：Return this independent Runtime04 fixture lifecycle to Runtime15; the origin descriptor-filter-plan-anchor-loss record already contains the matching lower and upper evidence. The parent lifecycle VM and aggregate descriptor obligations remain deferred. Independent C0/I0/M0 review, owner-ticket closeout binding, coordinator Git commit and SHA-deduplicated WeCom notification remain pending; no commit or notification is claimed by this return.

## 2026-09-09 fixture validation continuation

The first managed rerun on the `mip_bias` correction reached the next asserted
token and exposed a second omitted input: `max_anisotropy = 8`. The descriptor
module reported 26 passed and 1 failed with the received value `1`; the parser
default and production source were left unchanged. Snapshot `3305` adds only
that TOML field and records final test hash
`dfa7019f6c3be58c11e6508d019ab1bdd534652fdbd34118da54dfd032ef86f7`.

On immutable input
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-subasset-final-3305-20260909`
(`3bfe12829402a3c0e72beaea5fdfff8d2d7c33125d97c8a8dc24f5b69899e15c`),
managed job `2c03b7c385ba4b5eb4e31dbec084ad19` executed the full descriptor
module: 27 passed, 0 failed, 0 ignored, including the exact metadata-token
test. Runtime15's independent structure guard remains valid for its unchanged
guard source (job `31beb7da597f443d9165bf58af3e1dcd`, 1/1 on snapshot 3298).

The fixture lifecycle is dynamically green but remains open until the origin
descriptor record receives the evidence, its non-VM review/return gates are
completed, and the coordinator records closeout. The deferred `zr_vm` half is
not claimed here.
