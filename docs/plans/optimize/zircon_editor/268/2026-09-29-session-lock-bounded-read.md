---
title: Editor268 bounded project session-lock parser
category: zircon_editor
report_id: Editor268-P1-58-session-lock-bound-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: bounded_input_product_gate_pending
---

# Editor268: bound session-lock record admission and reading

## Change

`SessionGuard` previously used `read_to_string` on `.zircon/session.lock`, allocating for the whole file before the shared decoder validated its fields. It now opens the file and reads through a 1025-byte `Read::take` limit. A record over 1024 bytes returns a path-bearing `InvalidRecord`. Missing files still inspect as `Missing`; open/read errors and invalid UTF-8 retain the `Io` disposition; malformed records remain `InvalidRecord`.

The shared record contract now rejects instance IDs over 128 UTF-8 bytes, and the public decoder rejects source strings over 1024 bytes before line parsing. This makes the reader limit compatible with every record accepted by the constructor and encoder. The existing editor-generated instance ID is shorter than the new bound. The limits deliberately narrow the accepted public format: a formerly accepted overlong instance ID or record is now invalid. The codec file also contains a preexisting newline-encoding correction; its whole-file hash covers both changes.

## Evidence and remaining gate

- Shared source SHA-256: `session_lock/record.rs` = `ee595bd8adb07565a1b7d367044ef3ad3a6bb380e417ccd51100025958d3284e`; `session_lock/codec.rs` = `7e63bac20008b5f0eecb500f4cfb99d9befe8bf41f892b085c68219f7478d833`.
- Editor reader SHA-256: `zircon_editor/src/core/recovery/session_guard/record.rs` = `7493a9a86e128b32f3600317b2abc621f95302ff9d61aa448983a37d827190bb`.
- Shared tests construct a maximum-width valid record, check its encoded size and round trip, reject a 129-byte instance ID, and reject a 1025-byte direct decoder input. The Editor regression writes a 1025-byte real lock file and checks the path-bearing error. The tests are written but have not run in the managed batch.
- Scoped `rustfmt --edition 2021 --check` and `git diff --check` passed. No Cargo command ran for this slice.
- Reader input is byte-bounded; no startup/recovery p50/p95/p99, RSS, allocator, or full product comparison has run. The broader Editor268 parser, filesystem, crash, and performance gates remain open.

Run the shared-interface and Editor library regressions together in the next managed batch after the frozen v11 request has a terminal receipt. The offline successor draft is not validation evidence.
