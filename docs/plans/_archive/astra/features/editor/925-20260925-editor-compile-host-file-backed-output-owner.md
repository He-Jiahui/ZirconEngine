---
related_code:
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
implementation_files:
  - zircon_editor/src/core/export/stages/compile_host.rs
tests:
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/export/stages/compile_host/zero_copy_tail_finalization_tests.rs
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/256/2026-08-29-zero-copy-command-output-tail.md
related_records:
  - docs/plans/astra/features/editor/924-20260925-editor-admission-byte-fixtures-and-commandlet-path-contract.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Editor925 Compile Host File-backed Output Ownership

## 计划完成列表

| Scope | Completed change | Evidence and remaining gate |
| --- | --- | --- |
| Synchronous Export CompileHost runner | Redirect the child process's stdout and stderr directly into their existing owned output-log files. After `wait`, sync both files and read them in bounded chunks to produce the byte counts, BLAKE3 digests, 64 KiB tails and output manifest. | Replaces two ad hoc pipe-reader threads with zero in this runner without changing its public result shape. The child writes directly to disk instead of blocking on unconsumed pipes. The additional post-exit read is an I/O cost and needs Windows product/release measurement before any throughput or latency claim. |
| Output correctness and source contract | Preserve the existing wrapped-tail finalization contract, add a Windows child-process regression that compares both redirected logs and manifest digests, and add a focused source guard for thread-free stdout/stderr ownership. | The guard was first added against the old implementation and its required `Stdio::from`/no-thread assertions were confirmed red by source inspection. Exact-file rustfmt and diff whitespace checks are passing; Rust regressions and product performance remain pending a grouped managed batch. |

The v32 filtered `core::jobs` library batch had already sealed its source
before this implementation. Its thread-ownership failure named both this
Export runner and Play's still independent output-reader owner. Editor927 now
moves the Play readers onto a dedicated Runtime task-pool owner with an
explicit two-reader completion barrier; this slice itself only removes the
Export violation and does **not** claim the whole guard or the parent process
shutdown barrier. The 09 plan also calls out long-running process/GPU/file
ownership as a separate product-level integration gate. No tooling files or
coordinator state were changed by this slice.

The normal `zircon_build.py` subcommands use synchronous subprocess calls.
If an abnormal build leaves a descendant process holding an inherited output
handle after the root exits, this file-backed runner snapshots the logs at
root-process completion; the old pipe readers waited for EOF. Fault injection
must prove the descendant/termination policy and final manifest semantics
before this output contract is accepted. The pending development suite cannot
substitute for that lifecycle or I/O-performance gate.
