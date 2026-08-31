---
title: Editor10 Decision Message Single-pass Formatting
category: zircon_editor
report_id: Editor10-decision-message-single-pass-2026-08-25
date: 2026-08-25
session_id: root-editor10-notification-projection-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Decision Message Single-pass Formatting

## Scope

This slice removes repeated whole-template replacement from localized decision projection. It
does not introduce the parent plan's typed localization argument enum, plural/unit formatting,
versioned cross-ABI codec, durable notification journal, or product-wide delta projection.

## Implementation

Decision message projection now scans the translated template in one forward pass and resolves
known numeric placeholders through a borrowed lookup closure. It creates an accumulating output
buffer only after the first known placeholder is found and writes `u64` values directly into that
buffer. It does not allocate a temporary argument collection or per-argument placeholder string.

Templates with no arguments or no matching placeholders return the translated `Arc<str>`
unchanged. Repeated known placeholders, unknown placeholders, malformed outer braces, and a known
placeholder nested inside an unknown outer brace retain the former global-replacement behavior.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 100,000 projections with 8 arguments | 800,000 full-template replacement passes | 100,000 forward template passes; <= 3 s | 87.5% full-template-pass reduction |
| Template with no matching argument | owned `String` copy followed by a new `Arc<str>` payload | reuse translated `Arc<str>` | message payload allocation removed |
| Matched arguments | initial owned copy plus one whole-message result per argument | one lazy accumulating buffer with direct numeric writes | per-argument whole-message buffers removed |

The ignored Windows-native release evidence prints `EDITOR_DECISION_MESSAGE_FORMAT_BENCH_V1`
with projection count, argument count, legacy and optimized pass counts, reduction basis points,
elapsed nanoseconds, and the elapsed-time ceiling. Exact elapsed time remains pending coordinator
terminal evidence.

## Validation

- Exact Rustfmt and scoped `git diff --check`: passed.
- Reused-storage, repeated-placeholder, unknown-placeholder, and nested-brace regressions are
  prepared with the ignored release evidence for one shared Editor10 coordinator batch.
- No local Cargo lane was launched and no compilation is being monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Remaining Parent-plan Work

Decision arguments remain limited to bounded `&'static str -> u64` facts. Typed strings, paths,
counts, plural rules, units, locale-aware formatting, versioned codec policy, and delta-based row
projection remain open in the parent plan.
