---
related_code:
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - docs/plans/zircon_plugins/08/failure-2026-07-19-runtime13-script-call-table-hardcut-consumer.md
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --no-default-features --features script --locked --jobs 1 runtime13_performance_ -- --nocapture --test-threads=1
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --lib --features backend-zr-vm zr_vm_backend_has_one_plugin_owned_dense_production_path --locked --jobs 1 -- --nocapture --test-threads=1
doc_type: milestone-detail
---

# Runtime13 M4 Generation-Owned ScriptCallTable

Plan: `docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`

Milestone: M4 current architecture and performance convergence

Status: `atomic_runtime13_plugin08_candidate_applied_managed_gates_pending`

Files: `zircon_runtime/src/script/vm/host/host_export_registry.rs`, `zircon_runtime/src/script/vm/host/script_call_table.rs`, `zircon_runtime/src/script/vm/tests/host_exports.rs`

Date: 2026-07-19

## Implementation

`HostExportRegistry` now owns one immutable `ScriptCallTable` for each checked registry generation. Successful registration increments the generation and rebuilds the table once under the same registry mutex; reads clone only the cached table snapshot. The table keeps a nested borrowed-name index and `resolve` returns a borrowed `ScriptCallSite`, eliminating descriptor/callback deep copies and repeated table validation from normal lookup.

Direct registry calls clone the immutable table while holding the registry lock, release the lock, and only then execute the callback. A controlled callback re-entry regression proves the callback can call `script_call_table()` without deadlocking. Registration failure remains atomic and the old fallible table-rebuild/owned-resolve API is removed rather than retained as a shim.

## Validation, review, and open handoff

- Current exact-three hashes are `742F90413E074570693B21F31C4252FA2CEB62B6CA0580B96D3CBA9EC3D98D1B`, `E23219CC7C81E21842569B43FED6811DD84E5EC25C7309FC50C38DC906C48018`, and `11A60AA924D7411D71F4C6F444D8A1794555F4DDCFEDFB66CBADF020B4B2C51A`.
- Atomic source snapshot `684` froze the current code candidate before this record correction; current baseline attribution freezes the corrected Runtime13 exact-three together with the Plugin08 consumer, source guard, and open handoff record. Scoped rustfmt/source guards/diff-check passed.
- Independent review found no code issue in the atomic candidate. The callback re-entry regression covers lock release, and its Windows overload timeout is raised from one to five seconds without weakening deadlock detection. The remaining review finding was this record's stale pre-migration wording; that wording is corrected here and final re-review remains pending.
- Canonical Plugin08 handoff `runtime13-script-call-table-hardcut-consumer` remains open for managed evidence, but its code candidate is applied: Plugin08 consumes the direct table value, resolves a borrowed call site, and clones it exactly once at the native-function registration boundary. Its source guard binds the exact `resolve -> cloned -> ok_or_else` chain and rejects callback-time registry lookup.
- Failure artifact `docs/plans/zircon_plugins/08/failure-2026-07-19-runtime13-script-call-table-hardcut-consumer.md` remains the single lifecycle owner and is included in the current atomic attribution; no fixed return is claimed yet.
- Superseded reservation `dddbb129131c40479fa894ed882111de` was released before the timeout correction. Reservation `34580722a32f4c15a1bbddd4e7446b55` expired without a job on 2026-07-19 and cannot be cited as evidence. Atomic validation copy `157a0ff1f5544eaf804a8f79839f8490` failed before producing a durable source root and carries no evidence. Reservation `fbdaaea1da5b403c806f22bf00cd3a49` was already consumed by job `b31856d50d94491fad40047b4b8b5a4a` / run `5d129587a5ef497b97b04962b413d513` and released `exit 101`: validation copy `b02b9aaf995d4dd4a9274740204c71f8` omitted the external `zr_vm` sibling, so the focused test did not execute. Main-checkout retry job `cccbbab6eecc43ca8a6b780d6196da4f` / run `6d193794552342f38eb59cbc25d4fd64` also released `exit 101` before the target because the shared lib-test build had 19 unrelated compile errors.
- The current exact-seven source fingerprint before the latest gate was `a9b4a524ce3afe3f892f64f8f79ab207902300b62a072e5f4c82d3282a825226`. Replacement reservation `960b69f5b7024610a022acdefdb17c59` was consumed by job `a3dbb677be3846ad9538e1edffc2dfaa` / run `afac3a9afdb0400489163c56f6bd7232`; it released `exit 101` with no live PIDs after about six seconds because `zircon_plugins/Cargo.lock` was not current and `--locked` correctly refused to update it. Raw stdout was empty and the focused target did not execute. The lockfile is outside this exact-seven candidate and must be reconciled by its workspace owner without removing `--locked`; no Cargo pass is claimed.

## Boundary

The exact-three Runtime13 source set must not be committed independently while Plugin08 consumes the removed API. Acceptance requires the Plugin08 fixed return, source-bound Runtime13 and Plugin08 feature gates, fresh independent review, and one legal atomic hard-cut commit sequence. Restoring the old `Result` API or owned `resolve` is forbidden.
