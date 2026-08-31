# Frameworks01 M1 `zr_math` physical hard cut (2026-08-24)

## Status

- `source_implemented`
- `hard_cut_complete`
- `static_boundary_green`
- `zr_math_locked_build_and_tests_green`
- `runtime_interface_math_contract_green`
- `look_at_structural_correctness_repair_green`
- `runtime_product_build_blocked_by_foreign_zr_rhi_wgpu_errors`
- `app_editor_workspace_validation_not_admitted`
- `milestone_not_accepted`
- `service_commit_not_requested`
- `wecom_not_sent`
- `performance_claims_not_admitted`

## Outcome

Frameworks01 M1 now has a physical, low-dependency canonical math owner at
`zircon_runtime/crates/zr_math`. Runtime Interface no longer owns math algorithms: it retains only
the versioned `SchemaId`-bearing coordinate, unit, and precision DTOs and explicitly projects the
approved `zr_math` API. `zircon_runtime::core::math` is likewise an explicit product projection.

The old Interface implementation files were deleted in the same batch. There is no compatibility
module, wildcard re-export, copied implementation, or temporary dual owner. The resulting crate
dependency DAG is:

```text
zircon_runtime ----------------------> zr_math
       |
       +--> zircon_runtime_interface -> zr_math
```

`zr_math` has no reverse dependency on either product crate. Public exposure flows in the opposite
direction through the Interface and Runtime projections, but that does not create a dependency from
the foundational owner back to either facade.

## Architecture evidence

The preflight inspected 13 files, 1,556 lines, and 46,405 bytes with tree SHA-256
`6640a92d7b4d982232ef2e5ecedae16de8f3acf636b96fde85599cea8408a8c6`. It found the fatal
pre-cut coupling: pure convention vocabulary and `SchemaId`-bearing ABI DTOs shared one Interface
module. Moving that module whole would have created `zr_math -> zircon_runtime_interface`.

The implemented split follows the reviewed reference pattern:

- Unreal Runtime Core Math: one foundational implementation owner and stable public projections;
- Bevy `bevy_math`: a low-dependency independent math crate with curated exports;
- Zircon: pure conventions and algorithms in `zr_math`, versioned ABI identity in Runtime
  Interface, and explicit symbol lists at both product facades.

`zr_math` depends only on `glam`, `serde`, and `thiserror`; `serde_json` is test-only. Its crate root
forbids unsafe code. Root/Runtime/Interface manifests and `Cargo.lock` are wired atomically while
preserving the pre-existing Frameworks05 lock additions. After the structural look-at repair below,
the resulting physical crate contains 14 files, 1,306 lines, and 44,264 bytes including its manifest
and owned tests.

## Test-driven implementation evidence

The new boundary guard was run before source movement and produced the expected RED state: 2
failures and 2 errors because the crate did not exist and Interface was still the owner. After the
hard cut, `tools/tests/test_frameworks_01_math_crate_boundary.py` is GREEN at 4/4 (0.017 s on the
final static rerun). It
asserts workspace wiring, the dependency-neutral owner, removal of old Interface implementation,
absence of `SchemaId`/reverse dependencies in `zr_math`, the Interface schema split, and explicit
non-wildcard product projections.

Managed Windows validation used only coordinator-owned targets on `D:` or `F:`:

- job `62af0dde42374ea0929adee8d44ad523`: first `zr_math` run built production successfully, then
  exposed one owned test dependency defect (`serde_json` missing); fixed by adding a dev-dependency.
- job `2b7182872671426e98946ac8858ffd8d`: no-locked `zr_math` build 26.29 s and lib tests 6.79 s,
  both GREEN.
- job `26406e9497634622b2c206f03ca19c36`: locked `zr_math` build 0.94 s and lib tests 1.52 s,
  both GREEN on the reused pool.
- job `6a7c0627725544fba312d0a39498e233`: locked Runtime Interface production build 17.60 s and
  `math_contract` compile/test 4.41 s, 6/6 tests GREEN.
- job `246fdaf5d6c443f9b71149d744b5675e`: locked Runtime production build compiled the new math and
  Runtime Interface layers, then failed in foreign `zr_rhi_wgpu` current source after 83.52 s.
- job `888e3f0dfdfc40a6b2147b085ef08f6b`: the first focused look-at characterization compiled and
  produced the expected RED assertion failure on the old degenerate-basis implementation.
- job `b4b2f82bd20b46d1974b46e6ae8aca3d`: both focused infallible look-at regressions were GREEN after
  the structural repair.
- job `f250a797b95548cea0aff340e9a585d5`: final locked `zr_math` production build completed in 2.56 s
  and the complete library-test batch completed in 1.07 s, both GREEN.

The Runtime blocker is five errors outside this session's ownership:

- `production/diagnostics/query.rs`: missing local `tracker`;
- `resource_validation.rs`: missing `BindingResourceType` import and stale `entry.resource` field
  (the current field is `resource_type`);
- `production/diagnostics/readback/completion_order.rs`: derived `Default` imposes an unsatisfied
  `DiagnosticBatchCompletion: Default` bound at two construction sites.

A broader Runtime Interface lib-test job (`80ba979fd8b0493ab0ccf0a80c49594a`) also built production
successfully, then failed only in foreign UI test materialization (`binding_value_contracts.rs` and
`UiTextShapeArtifact`). These errors were not modified or masked by Frameworks01.

Because the lower Runtime product gate is not green, App/Editor/workspace validation was not used as
acceptance evidence. This follows support-first diagnosis and avoids turning an upper-layer queue
into the only work item.

## Resource-I/O coordination result

UI12's three reported `atomic_write` E0432 errors are a mixed/stale validation fingerprint, not a
Shader06 migration. Both its recorded base and current HEAD publish
`crate::core::resource::io::atomic_write`, Frameworks01 defines that facade as the supported IBL
entry, and the reported line fingerprints do not match the current consumer blobs. The public
facade remains; no IBL file was rewritten by this session.

## Look-at structural correctness repair

The post-cut review counted 47 tracked external `Transform::looking_at` call sites across Runtime,
Editor, App, and plugins. Migrating all of them to the checked API would cross multiple active Plan
owners and would conflate deterministic camera construction with error-reporting policy. The owned
repair therefore preserves the two explicit contracts:

- `Transform::try_looking_at` remains the strict admission boundary and reports non-finite, zero,
  and collinear inputs;
- `Transform::looking_at` remains the recovery boundary, but no longer uses `normalize_or_zero` to
  feed a degenerate basis into quaternion construction. A missing forward axis becomes `-Z`; an
  invalid up axis becomes `Y`; and a collinear up axis is replaced by the cardinal axis least
  aligned with the normalized forward direction. The final basis and quaternion are normalized.

This matches the reviewed mature-engine structure without copying an engine-specific coordinate
convention: Unreal `TRotationMatrix::MakeFromXY/MakeFromXZ` detects nearly parallel inputs and
selects an arbitrary nonparallel cardinal vector; Bevy `Transform::look_to` replaces invalid axes
and uses an orthonormal fallback; Fyrox separately protects projection and matrix inverse
singularities. Zircon's least-aligned-axis selection maximizes the fallback cross-product magnitude
for the three cardinal candidates and is deterministic under equal components.

The two regression tests prove that coincident eye/target inputs retain translation while producing
a unit camera orientation with `-Z` forward, and that a collinear up vector produces a finite,
orthonormal basis while preserving the requested forward direction. Static boundary guard 4/4 and
the complete managed crate build/test batch remain GREEN after the change.

## Remaining algorithm review boundary

The module review still found three algorithm-design questions that require characterization before
any optimization or semantic rewrite:

- the infallible `perspective` path clamps only some degenerate inputs while the checked path rejects
  them, so projection consumer policy is not yet uniform;
- `NumericPolicy::STRICT` uses exact-zero floors, while production domains may need scale-aware
  tolerances derived from scene extent and operation conditioning;
- affine admission uses an exact homogeneous-row comparison, which needs a corpus of composed,
  imported, and round-tripped matrices before selecting a tolerance model;
- the render-narrowing receipt is structurally ready for `f64 -> f32`, but under the current
  `Real = f32` profile its range and error branches cannot provide meaningful narrowing data.

These are not treated as micro-optimization opportunities. A later algorithm milestone must first
inventory consumers, define correctness oracles, capture representative scene/import/render
workloads, and establish cold/warm CPU, allocation, cache, and power baselines with a coherent
product build. Unreal/Bevy source structure can guide ownership, but parity or optimal complexity
must be demonstrated with Zircon data rather than inferred from API resemblance.

## Performance and acceptance boundary

The timings above are validation observations, not a compile benchmark: the runs used different
cache states and there is no accepted pre-cut cold/incremental baseline. The look-at change is a
correctness repair and has no admitted profiling baseline, so this record makes no runtime latency,
throughput, power, bottleneck-removal, optimality, or Unreal/Bevy parity claim.

Source implementation and focused evidence are complete. M1 remains unaccepted until the foreign
Runtime product blockers are resolved, required product gates run on a coherent current snapshot,
independent review accepts the exact blobs, and the coordinator performs service commit and WeCom
notification.
