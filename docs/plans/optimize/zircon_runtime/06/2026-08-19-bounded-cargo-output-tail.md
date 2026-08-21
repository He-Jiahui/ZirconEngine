# Runtime06 Bounded Cargo Output Tail

- Date: 2026-08-19
- Session: `runtime06-export-output-tail-r1-01a00797-20260819`
- Plan finding: `P1-12`
- Status: passed

## Change

`zircon_editor/src/ui/host/export_cargo_process.rs` no longer accumulates the
entire stdout/stderr stream in memory. The existing temporary capture files
still spool the complete streams while the child is running and are deleted
after the final drain; the caller keeps only a bounded tail for
`EditorExportCargoInvocation`.

The same post-exit aggregation API was still consumed by the export wizard.
That path now records each final 64 KiB capture chunk directly into its existing
full output artifact and bounded line tail. The shared `final_output_drain`
function and re-export have been removed, so neither export consumer can
silently rebuild an unbounded in-memory stream at process exit.

Each stream has a fixed 256 KiB tail budget. Once full, incoming bytes evict the
oldest bytes, and the final diagnostic includes a structured human-readable
truncation line with retained, total, and discarded byte counts. Cancellation
diagnostics use the same bounded invocation fields. Existing command/status
semantics are unchanged. Capture chunks are cropped and appended in bulk;
capacity checks and front eviction no longer run once per output byte. The
post-exit drain also streams each 64 KiB capture chunk directly into the tail;
it no longer reaggregates all remaining output before applying the bound.

## Deterministic Delta

| Metric | Before | After | Delta |
|---|---:|---:|---:|
| in-memory stdout retained | full process output | 256 KiB tail | O(output) -> O(1) |
| in-memory stderr retained | full process output | 256 KiB tail | O(output) -> O(1) |
| full stdout/stderr transient spool during child lifetime | yes | yes, unchanged | 0 |
| tail capacity/eviction decisions per capture chunk | up to one per byte | one bulk decision | O(chunk bytes) branches -> O(1) branches |
| 20,000-line over-budget final tail visibility | last line visible | last line visible | 0 |
| Cargo post-exit drain accumulator growth | O(remaining output) | 256 KiB tail per stream plus one 64 KiB read chunk | O(1) |
| wizard post-exit drain accumulator growth | O(remaining output) before artifact write | one 64 KiB read chunk per stream plus existing bounded line tail | O(1) |
| truncation provenance | implicit | retained/total/discarded bytes | explicit |

The acceptance test writes 16 bytes into an 8-byte fixture budget and verifies
the exact tail and truncation accounting. A second fixture crosses the budget
over two capture chunks and verifies the same exact tail/accounting contract.
The process integration test also asserts both 20,000-line invocation strings
are truncated, remain bounded, and preserve the last line. A source guard
rejects the former full-stream aggregation symbol from the Cargo consumer, the
wizard consumer, and shared output-capture support, and requires the wizard to
record each chunk before continuing its drain loop.

## Validation

The Windows coordinator batch contains the focused editor host Cargo-process
tests and the existing export process-support tests. No direct Cargo command
was run from the worktree.

The first ordinary validation copy `256f647229a343a885636360d530188c` stopped
before Cargo because it did not contain a workspace `Cargo.toml`; run
`92b11a3ae212441ca3c23379bfbda607` is setup evidence only. The replacement
Cargo-closure copy is `daddaaaa776941f88cb1c7fd9605972e`.

Run `800fb6fbf91848b799e76150770bf5ad` reached Cargo and stopped before the
target test with 243 pre-existing Editor test-harness compile errors, including
private helper imports and four gateway doubles missing the operation methods.
This is an upstream baseline blocker, not a failure of the bounded-tail code;
the focused test will be rerun from the post-convergence mainline.

That rerun is grouped with App02 and Runtime02 in
`zircon-validation-optimize-followup-batch.ps1`
(`02a3dac8837e3c0193ef2a4713c0902dffbe65be1861847d830004269e1f5174`).
The outer script has zero parser errors and executes nine Cargo groups, 30
Python tests, and one real-adapter GPU run from one post-convergence source
copy.

Unified run `5258f82da9e041f1aca557eebfab2ccb` passed the preceding WGPU timer group, then stopped
in the App02 `zircon_runtime` GPU-timing prerequisite after 54 shared harness compile errors.
Runtime06's bounded-tail tests did not run and no output-tail performance result is claimed.

Current-source job `197e37fe25f94d00915fcd890b03724d`, run
`1562528434194a17879de2abbc2dbebf`, subsequently passed both Runtime06 Cargo groups and emitted
`RUNTIME06_VALIDATION_COMPLETE cargo_groups=2 bounded_stream_contract=1`. Coordinator replay run
`e3880656eefa4064aaa5920b37a1cb4d` pinned that exact terminal evidence alongside the remaining
Main gates. The accepted performance delta is bounded memory, not a timing claim: stdout and
stderr retention change from O(process output) to a fixed 256 KiB tail per stream, while post-exit
working memory is bounded by that tail plus one 64 KiB read chunk.
