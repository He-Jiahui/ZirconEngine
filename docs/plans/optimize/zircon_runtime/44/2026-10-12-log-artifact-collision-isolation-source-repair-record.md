# Runtime 44 log artifact collision isolation Source repair

**Status:** The bounded exclusive-create collision-isolation sub-item is present in canonical Source. Compilation and type checking, Windows Native execution, actual fixture cleanup evidence, full performance, product, and Jenkins acceptance remain open.

## Repair scope

The file sink now claims a candidate output with exclusive `create_new(true).append(true)` semantics, so an existing artifact is never reopened. Only `AlreadyExists` advances to the next candidate; other open errors keep the existing fallback behavior. Each candidate tries one primary name and at most 32 alternatives. A checked image-local `AtomicU64` nonce cannot wrap. The generated basename is bounded to 64 bytes; that bound does not cap the complete path.

The chosen actual path remains owned by the active sink. The change preserves active-owner reuse, candidate and disabled-sink ordering, terminal notes, directory-error fallback, timestamps, worker stdio, and streaming FIFO. The nine committed path attributes from `21da363625e644aacca45b8c90621547cd42ba5b` remain preserved.

Two Windows ordinary, non-ignored regression cases were added to `sink/tests/mod.rs`:

- `diagnostic_log::sink::tests::same_timestamp_generations_use_distinct_files_and_join_workers`
- `diagnostic_log::sink::tests::file_collision_errors_preserve_existing_artifacts_and_directory_fallback`

These names and their file-backed worker and cleanup assertions exist in Source; neither case was executed. The existing test prefix remains byte-exact at 2,471 bytes.

## Applied Source evidence

The [actual canonical apply receipt](../../../../../.codex/outbox/runtime44-log-artifact-collision-isolation-repair-v1/actual-canonical-apply-v2.json) is SHA-256 `6cce870ef72399e1a50e42dd70cb9eb8a5608cc5fde07088a2f793c0653b1ffe` (20,384 bytes). It records canonical application and successful strict apply and reverse dry-run checks.

The two postimages are:

- [`sink.rs`](../../../../../zircon_runtime/src/diagnostic_log/sink.rs): 28,185 bytes, raw SHA-256 `a597874d7d8ea31a459590e1022d0ce32d9a50c86bfd33508ac74d247c73f686`.
- [`tests/mod.rs`](../../../../../zircon_runtime/src/diagnostic_log/sink/tests/mod.rs): 32,014 bytes, raw SHA-256 `6df9d49f7867313b1d74ea5f48c3e241b5c1fe411ed8ac40335e4d4ba82eccdc`.

The frozen [binding](../../../../../.codex/outbox/runtime44-log-artifact-collision-isolation-repair-v1/binding-v2.json) is SHA-256 `54a0459c876cbd090f472c8765c310a3af50f60dde7447bfc97eb1107882aee4` (39,801 bytes). The [independent Source review](../../../../../.codex/outbox/runtime44-log-artifact-collision-isolation-repair-v1/independent-review-v2.json) is SHA-256 `44be1c8fde203e83213809a2d4f6a0fb7deb03caa0d444f58106f9d8d64c3e8d` (37,304 bytes); it passed with zero P1 and zero P2 findings. Its 33 guards comprise the two canonical Source preimages and 31 read-only guards; all 31 remained unchanged (22 raw-byte exact and 9 literal-path-only).

The receipt records HEAD `5a547d4e2727d27ddf99578f548efb7e6d19d915`, unchanged index SHA-256 `931f84f2161056ce079e6ce793d1fd98e0284a0827ee0f36df510d44a212a161`, and zero Source copies. Its immutable `sourceWriterExitVerifiedByCaller: false` field reflects receipt creation while the writer was still running. Root later independently observed the writer cell exit code 0 and recorded `sourceWriterExitVerifiedByRoot: true` and `sourceChildrenExited: true` in ledger entry 72; the apply receipt was not changed.

## Acceptance still open

No compile or type check, Native case, fixture creation or cleanup, performance sample, product check, or Jenkins acceptance was performed in this Source-only adoption. The preregistered fixture root was not created or used. The apply receipt records `fixtureCreated: false`, `nativeInvoked: false`, all corresponding acceptance gates false, and zero bytes released.

This closes only the exclusive collision-creation Source sub-item under Runtime 44 P1-32. Full P1-32 requirements remain open, including UTC precision, process-generation and nonce guarantees, real cross-process and DLL behavior, security, rotation, and product integration. P1-13 queue, per-field, and process record-byte caps and P1-24's whole-logical-batch hard cap also remain open. Full numeric performance and hardware acceptance remains pending.

The original 15 command contracts remain intact: 14 original producer contracts plus 2 Runtime 03 companion contracts, 16 total. The collision cases add zero producer identities and zero ignored producers; the original 54+1 matrix and critical 20 remain unchanged. The matching [Source completion list](../../../astra/features/runtime/2026-10-12-runtime44-log-artifact-collision-isolation-source-completion-list.md) tracks this sub-item and its outstanding gates.
