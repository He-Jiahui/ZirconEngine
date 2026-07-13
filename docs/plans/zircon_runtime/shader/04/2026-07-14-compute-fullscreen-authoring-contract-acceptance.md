# Shader 04 compute/fullscreen authoring contract acceptance

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | compute/fullscreen named-resource contract at the feature graph boundary | completed | 2026-07-14 | Added one cross-layer regression using the real `ComputeDispatchBuilder`, `FullscreenPassBuilder`, and `RenderFeaturePassDescriptor`. A compute resource declared as `StorageBuffer` but bound as `Texture` returns `ResourceKindMismatch` with the resource name; a valid fullscreen texture input becomes one read-only graph texture resource. |
| M3 | SH04-M3-T compute/fullscreen authoring testing stage | passed | 2026-07-14 | Status anchor: `shader_plan04_compute_fullscreen_named_resource_feature_graph_contract_passed`. Fresh managed Windows Cargo execution passed the exact current-source test 1/1, 0 failed, 7968 filtered. Earlier current-source focused tests also passed compute builder 3/3, fullscreen builder 2/2, compute descriptor projection 1/1, fullscreen descriptor fixture 1/1, and graph compute workload projection 1/1. `rustfmt --check` and scoped `git diff --check` passed. |

## Scope and limits

This acceptance closes the SH04-M3 named-binding diagnostic and feature graph
projection contract. It does not claim that every historical HZB, particle, or
post-process executor has already migrated away from hand-authored backend bindings.
Those executor migrations remain later render-plan work and do not require a second
authoring ABI.
