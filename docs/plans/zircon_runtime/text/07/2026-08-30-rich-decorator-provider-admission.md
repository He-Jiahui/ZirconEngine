# Rich decorator provider admission

Date: 2026-08-30

Status: `panic_boundary_implemented_static / decorator_output_admission_implemented_static /
retained_run_metadata_admission_implemented_static / managed_behavior_validation_pending`

## Scope and decision

The exact-tag HashMap removes provider-count-dependent lookup work, but lookup speed does not isolate
custom provider behavior. `RichTextDecorator::decorate` is third-party synchronous code: it could
panic, return a large family/link/feature payload for an active tag, or cause that payload to be cloned
into many retained runs. Source/output/token/run-count budgets did not account for those paths.

The minimum infrastructure closure is deliberately two-level:

1. Every resolved decorator call runs behind `catch_unwind(AssertUnwindSafe(...))`. A panic returns
   `RichTextParseError::DecoratorPanicked { tag }`; failed compiled single-flight cells follow the
   existing terminal-error/removal path rather than publishing a partial artifact.
2. One accepted decorator result may expose at most 64 KiB of dynamic family/link/icon-font/feature
   metadata by default. Rejection happens before the result enters the active tag stack or emits an
   inline run.
3. Materialized runs may retain at most 32 MiB of dynamic metadata per request by default. The builder
   charges each non-merged run before text or run publication; adjacent equal metadata that resolves to
   one retained run is charged once.
4. UI maps decorator panic to `TextLayoutError::LayoutFailed`. Capacity failures continue to map to
   `RichTextBudgetExceeded`/`ZR-TEXT-LAYOUT-012`, so provider failure is not mislabeled as a byte quota.

The retained charge uses semantic lengths, including `OpenTypeFeature` element bytes. It does not claim
to bound a provider's private temporary allocations while the callback is running. Deadline,
cancellation, allocator quota, process/plugin isolation, owner leases, unregister/revoke, and
panic-abort builds remain separate work.

## Evidence

Typed Rust regressions cover a panicking custom decorator, parser reuse after the panic, per-call link
metadata rejection, and cumulative metadata rejection on the second differently linked run. They are
written but have not run because this session is not contending with the unrelated active Cargo job.

The combined Python Runtime Text static suite passes 38/38. It verifies the catch boundary, typed
errors, both budget fields, builder cumulative owner, UI error split, and Rust regression presence.
Targeted Rust 2024 formatting passes. Current production file sizes are admission/decorator/parser/
builder 469/185/720/183 lines, all under the 800-line review budget.

An isolated no-op dynamic-callback boundary profile also compared the former vector lookup/direct
callback with the new HashMap lookup/`catch_unwind`/callback path. At 16/256/4,096 decorators, new p50
was 146/149/154 us for 4,096 calls versus old 541/7,869/112,965 us; the largest lane improves 733.54x.
This shows that unwind isolation does not restore provider-count-dependent dispatch, but it does not
measure a real callback, panic execution, or metadata allocation.

Managed Cargo, panic behavior in the selected product profile, callback timeout/cancellation,
allocation/RSS/power, real WGPU framebuffer, and a new rendered PNG under `docs/tests/runtime/text`
remain pending. No milestone acceptance or commit is claimed by this static slice.
