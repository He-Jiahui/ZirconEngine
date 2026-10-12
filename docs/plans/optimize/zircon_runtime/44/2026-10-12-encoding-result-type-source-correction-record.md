# Runtime 44 encoding result type source correction

**Status:** Source correction adopted; native validation and performance acceptance remain pending.

## Scope

This follow-up corrects one Rust type path in `zircon_runtime/src/diagnostic_log/sink.rs`. The formatter returned `io::Result<()>`, but the module does not import `io`. The return type is now written as `std::io::Result<()>`. This qualifies the existing standard-library type and does not change formatter behavior.

The earlier parse-only Rust formatting check did not resolve names. Its successful parse therefore did not establish that `io::Result` was in scope. The current correction has an exact source-application receipt, but no compiler or type-check result is claimed.

## Source application evidence

The actual canonical apply receipt is [`actual-canonical-apply-v1.json`](../../../../../.codex/outbox/runtime44-encoding-result-type-resolution-repair-v1/actual-canonical-apply-v1.json), SHA-256 `f0d9be722b5fb446569265c0c82efe466c0412e1703ddc25587c8382ca3a6c75` (2,287 bytes). The corresponding patch is [`candidate-v1.patch`](../../../../../.codex/outbox/runtime44-encoding-result-type-resolution-repair-v1/candidate-v1.patch), SHA-256 `3cd1a87039da13f8b9f6222d2459aaede3e2169d1b8db511f0c014a2cdc691fd` (475 bytes).

The receipt records a strict check and actual apply exit code of 0, an exact physical postimage, an unchanged index, preserved CRLF line endings, and an exact inverse proof. For `sink.rs`, the recorded before image was 25,309 bytes (raw SHA-256 `5a10a21549eddbc2bafbd5f89b32f228bfca8295c233ac49c04f320e75c00c3a`; LF-normalized SHA-256 `23061599c03ead8f9fadac9c4c09e3007e32b613dcdb4d8b749c3abeba718d08`). The actual after image was 25,314 bytes (raw SHA-256 `a8e28a226fc0d1db6117967cbbcc099be73beeec582bd12808bc4eedb6900154`; LF-normalized SHA-256 `609cd58d7940e1e4ab355359aaf83b671daefd4f60ba8e702c0cdae564e4be52`).

The earlier four-packet Source adoption is documented by its [historical actual-apply receipt](../../../../../.codex/outbox/runtime-editor-public-contract-repairs-actual-adoption-20261012-v1/actual-canonical-apply-v1.json). That receipt and the six associated Source records remain unchanged; this document records only the additional type-path correction.

## Validation boundary

The recorded Rustfmt parse exit code was 0. Rustfmt parsing is not type checking. No Native command, Rust test, or performance sample was run for this correction. It adds zero ordinary test cases and zero producers.

The pending Native07 validation batch is a batch name, not a Runtime feature number. Its planned validation contract retains 15 commands and 16 producers, including the prior 11 added ordinary cases; the documented minimum floors remain 59 cases for command #2 and 193 for command #15. This correction does not lower or replace those gates. Full performance validation, product acceptance, Jenkins acceptance, and the overall milestone remain pending.

The earlier Runtime 44 Source work remains described in the [encoding and empty-filter Source record](2026-10-12-oversized-encoding-and-empty-filter-source-repair-record.md) and [feature completion list](../../../astra/features/runtime/2026-10-12-runtime44-encoding-filter-source-completion-list.md).
