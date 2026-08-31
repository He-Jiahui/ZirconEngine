---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: runtime-interface-pointer-route-stale-serde-field-attributes
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/12-ui-authoring-accessibility-input-diagnostic-status-operation-public-contract-current-source-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/12
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/pointer/route.rs
tests:
  - cargo build -p zr_resource --locked
  - cargo test -p zr_resource --locked --lib projection_snapshot_
---

# Frameworks01: Runtime Interface pointer route keeps serde field attributes after manual serde hard cut

## Failure receipt

Frameworks01 ran the managed Windows I1 support-layer validation from the shared current source:

```text
validate-matrix.ps1 -Package zr_resource -LibTests -TestFilter projection_snapshot_
  -TargetDir E:\cargo-targets\frameworks01-resource-identity-i1-green -VerboseOutput
```

The coordinator receipt is:

- job `c373ffe7a3164d06bd9eaabb1f75086b`;
- validation Session
  `validate-matrix:019ffe2b-296a-7023-9433-8654b9ea8f18:successor:814a10c25ca4470f8d8b98bae4f78982`;
- start `2026-08-31T07:39:50.117852+08:00`;
- finish `2026-08-31T07:41:59.762994+08:00`;
- release `2026-08-31T07:42:02.649508+08:00`, exit `1`;
- target and scratch remained under `E:`; no artifact was written to `C:`.

Cargo stopped while compiling `zircon_runtime_interface`, before `zr_resource` was compiled. Rustc
reported five `cannot find attribute serde in this scope` errors at `route.rs:53`, `55`, `66`, `68`
and `70`.

## Current-source diagnosis and ownership

Current `route.rs` SHA-256 is
`a75a6782f5b3baaca2246b19092a132ae3240039647121c929765e2faa26c18d`. Its shared-worktree diff
hard-cuts `UiPointerRoute` from derived serde to manual `Serialize`/`Deserialize`, but leaves
field-level `#[serde(default)]` attributes on the production struct. Those attributes are valid only
inside a serde derive input; the manual wire DTO already owns the required defaults.

Coordinator ownership matrix request `ee816b4bb31848dba6e77dae30e83a0b` reports the exact path as
`modified / unowned / attribution_missing`, with no live lease. Frameworks01 did not claim or edit
the file. The failure was routed to RuntimeInterface owner task
`01a00797-56e0-70f1-a57c-dc3fb65263e8` with the exact hash, diagnostics and job receipt.

## Acceptance

- Establish one legal owner for the whole current pointer-route blob; do not split or reattribute
  only the five lines.
- Remove the stale production-struct serde field attributes while preserving the manual wire DTO's
  defaulting semantics and pointer-route serialization compatibility required by the owning plan.
- Run the owner-focused pointer route serde tests and compile `zircon_runtime_interface`.
- Return the final file hash and integration receipt. Frameworks01 then reruns the exact managed
  `zr_resource` command above.

## Constraints

- Frameworks01 must not claim, rewrite or commit this foreign mixed blob.
- Do not restore derived serde merely to make the attributes compile; the current manual serializer
  intentionally projects the shared routing path back to the wire `bubbled` field.
- This failure does not invalidate the I1 static GREEN. It blocks managed compile/test evidence only,
  so Frameworks01 continues non-validation Resource support work.

## Current-source repair receipt

RuntimeInterface03 now owns the complete current `UiPointerRoute` blob under lease request
`bb5441736be9431ba4c6665fcdf0ce27`. The manual serde hard cut is internally consistent:

- `UiPointerRoute` has no field-level `#[serde(...)]` attributes;
- the derived `WirePointerRoute` deserialization DTO retains the six required `#[serde(default)]`
  defaults for backward-compatible wire input;
- serialization still emits the stable `bubbled` route projection and preserves borrowed route
  traversal without an allocation in the hot path.

Current source SHA-256:

`zircon_runtime_interface/src/ui/surface/pointer/route.rs`
`1b784b39603ed6eb8c8670e84c0a21708d3c10c202c42e3078ba3b8dde985ea2`

Scoped static contract, rustfmt, and diff checks are green. A fresh managed Windows Rust 1.94.1
`--locked --release` `zircon_runtime_interface` gate is queued; this record remains `open` until
that gate and the originating Frameworks01 `zr_resource` rerun both pass.

The shared batch is ticket `e5ad38bacd664fdb87ad7d4fa9acb22c`, submitted by request
`92e7978469ca4402831ac77ddef98c80` (receipt `fc62c260f6254b9aafcc6702cba4f517`).
