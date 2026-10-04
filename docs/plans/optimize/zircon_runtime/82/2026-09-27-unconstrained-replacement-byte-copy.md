---
title: Runtime82 Unconstrained Replacement Byte Copy
category: zircon_runtime
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: not_measured
---

# Runtime82 unconstrained replacement byte copy

## Scope and invariant

`TextInputConstraints::sanitize_replacement_with_boundary_map` previously decoded and appended every Unicode scalar even when no operation could change the replacement. It now copies the UTF-8 bytes into the required owned `String` when all four conditions hold: no grapheme limit, `Any` character filter, multiline enabled, and no boundary-map request. The receipt remains empty.

Committed insertion/paste reaches this path through `editable_text/state_transition.rs`. IME preedit always supplies a boundary map and continues through the original scalar scan. Length limits, character filters, single-line separator removal, retained grapheme authority, and mapping receipts retain their existing path. Existing foreign import ordering in the source is preserved.

This removes scalar decoding/filter checks/individual pushes from unconstrained replacement sanitization. The necessary owned string copy and its allocation for nonempty input remain; complexity is still linear in replacement bytes.

## Behavioral coverage

The module-local `text_constraints/optimization_tests.rs` was written before the production change. Five ordinary tests exercise:

- Byte identity and empty receipts for ASCII, Unicode combining sequences and ZWJ emoji, empty text, NUL, and all hard-line separators; both source-scan and document-index entry points are covered.
- Grapheme limits through both retained-count authorities.
- Character-filter removal and its scalar receipt count.
- Single-line removal with CRLF counted once.
- Actual preedit cursor/clause mapping for multibyte text and an empty preedit, ensuring the fourth guard cannot skip required mappings.

Dynamic test execution remains pending in the grouped managed Runtime/Editor batch.

## Release comparison and acceptance

Ignored profile: `runtime82_unconstrained_replacement_byte_copy_release_profile`.

The baseline preserves the full prior production sanitizer, including all constraint and boundary-map branches, from source SHA256 `769de443484c2063543c7519dab8973491471dba90d64c50148e07373b25e8ba`; only its test-local method name changes. Its source body was compared with the saved preimage. The measured new path invokes the actual production `sanitize_replacement` method. Inputs and callable values cross `black_box` barriers; output identity and empty-receipt checks, destruction, and fixture preparation occur outside timing.

Two workloads each contain 1,000,000 graphemes: 1,000,000 ASCII bytes and 3,000,000 CJK UTF-8 bytes. Each runs five warmup pairs and 31 measured pairs with alternating old/new order. Marker `RUNTIME82_UNCONSTRAINED_REPLACEMENT_BYTE_COPY_BENCH_V1` reports platform/profile, raw old/new nanosecond samples, and separate p50/p95/p99 for each workload. The pending local gate is new p95 <= 80% of the old p95 for both workloads.

| Gate | State |
|---|---|
| Five behavior regressions | Implemented; managed execution pending |
| Full old/new Release comparison | Implemented; measurement pending |
| Runtime82/RTE native latency, product memory, and end-to-end acceptance | Open; this local comparison supplies no acceptance claim |
