# Shader 04 material command contract acceptance

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M2 | material option, disabled-pass, and queue command integration | completed | 2026-07-14 | Added a combined renderer regression proving default and option-bit materials produce separate Forward variants, a material with Shadow disabled produces no shadow command, and a transparent material routes to `Transparent3d`. The registered variants preserve option bits `[0, 1]`; four command requests produce three unique variants and one memory hit. |
| M2 | SH04-M2-T renderer material command testing stage | passed | 2026-07-14 | Status anchor: `shader_plan04_material_option_disabled_pass_queue_command_contract_passed`. The exact current-source Windows test passed 1/1, 0 failed, 7965 filtered. The first red run exposed an incorrect test expectation: the same default material legitimately reuses its Base/Shadow shape accounting, so the accepted assertion is three unique variants, four requests, one memory hit, and zero compile misses. `rustfmt --check` and scoped `git diff --check` passed; the test owner remains below 500 lines. |

## Scope and limits

This record closes the SH04-M2 renderer command integration acceptance gap identified
in the root plan. It does not claim SH04-M3's broader compute/fullscreen executor
migration or the whole Shader architecture goal complete.

The test uses the actual `OpaqueBasePassProcessor`, `ShadowPassProcessor`,
`TransparentPassProcessor`, `MaterialDisabledPasses`, `PipelineKey`, and
`MeshPipelineVariantRegistry`; no test-only renderer path is introduced.
