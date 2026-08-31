---
title: Editor52 Single-buffer Capability Error
category: zircon_editor
report_id: Editor52-single-buffer-capability-error-2026-08-25
date: 2026-08-25
session_id: root-editor52-capability-binary-lookup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor52 Single-buffer Capability Error

## Scope

This slice removes per-capability string clones and the temporary missing-capability vector from
`ViewRegistry::descriptor_capability_error`. It preserves capability order and the exact public
error text. It does not close Editor52's provider, catalog-generation, localization, template, or
product-qualification gaps.

## Implementation

The retired error path filtered every required capability, cloned each missing `String`, collected
those clones into a `Vec`, joined them into another `String`, and formatted the final error into a
third string. The registry now:

- scans once and returns `None` without allocation when every required capability is available;
- borrows the first missing capability before allocating;
- reserves one final error buffer using a conservative upper bound;
- appends later missing capabilities directly in descriptor order without cloning their strings.

The mixed available/missing regression locks byte-equivalent output and the all-available result.
A source contract rejects reintroduction of `.cloned()` or a collected missing-capability vector in
the production function.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Missing capability string clones per 256-capability error | 256 | 0 |
| Structural heap allocations per 256-capability error | 259 | 1 |
| Alternating release benchmark | 11 samples x 2,000 errors | optimized P95 <= 65% of retired P95 |

The 256-capability case is an extension-metadata pressure matrix, not the built-in catalog size.
The ignored release benchmark emits `EDITOR52_CAPABILITY_ERROR_SINGLE_BUFFER_BENCH_V1` with both
P95 timings, reduction basis points, capability/iteration counts, allocation count, and clone count.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and the production source guards passed
before submission (apart from the repository's existing CRLF notice). The managed Editor workbench
registry batch covers the prior borrowed-descriptor open/restore behavior, the new capability error
semantics and source contract, and both ignored release benchmarks in one Cargo invocation. Dynamic
P95 evidence, integration SHA, and automatic WeCom performance delivery remain coordinator-owned
and pending.
