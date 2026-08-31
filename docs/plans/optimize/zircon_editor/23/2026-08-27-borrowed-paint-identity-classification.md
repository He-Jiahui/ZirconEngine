---
title: Editor23 Borrowed Paint Identity Classification
category: zircon_editor
report_id: Editor23-borrowed-paint-identity-classification-2026-08-27
date: 2026-08-27
session_id: root-editor23-precompiled-collection-type-traits-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Borrowed Paint Identity Classification

## Scope

This batch removes owned normalization from three retained-host paint identity paths: search-field
recognition, alert-tone selection and danger icon-button selection. It preserves ASCII
case-insensitive token matching, classifier precedence, alert tones, icon-button colors and search
field behavior. It does not change component DTOs, layout geometry, command generation or public
APIs.

## Changes

- Search-field identity now finds the fixed `search` token through borrowed byte windows instead of
  allocating a lowercase copy for each examined identity field.
- Alert-tone selection scans each borrowed severity key directly and retains warning, error,
  success and info precedence.
- Danger icon-button selection scans `control_id`, `icon_name` and `validation_level` directly,
  removing the compound formatted string and its lowercase copy.
- Three Rust regressions preserve mixed-case search, warning and danger semantics. One Python
  contract prevents allocating normalization from returning to any of the three paint paths.

## Deterministic Performance Evidence

The independent release model classifies 65,536 representative nodes per task. Each task uses 21
paired samples with alternating baseline/optimized order; each reported sample is the median of
three sub-runs to reject fixed scheduler pauses. The checksum and allocation pass is separate from
timing; fixture construction is outside the measured functions.

### Search Identity

| Evidence | Owned lowercase | Borrowed windows | Result |
| --- | ---: | ---: | ---: |
| Checksum | 32,768 | 32,768 | identical |
| Allocations | 196,608 | 0 | 196,608 fewer; 100% reduction |
| Run 1 P50 / P95 | 35.1729 / 43.8404 ms | 5.7507 / 10.2654 ms | 83.65% / 76.58% faster |
| Run 2 P50 / P95 | 35.1860 / 43.0800 ms | 5.6637 / 9.1010 ms | 83.90% / 78.87% faster |
| Run 3 P50 / P95 | 33.9750 / 70.0217 ms | 5.5849 / 11.2486 ms | 83.56% / 83.94% faster |

### Alert Tone

| Evidence | Owned lowercase | Borrowed windows | Result |
| --- | ---: | ---: | ---: |
| Checksum | 163,840 | 163,840 | identical |
| Allocations | 65,536 | 0 | 65,536 fewer; 100% reduction |
| Run 1 P50 / P95 | 20.7158 / 27.9423 ms | 5.3192 / 7.9633 ms | 74.32% / 71.50% faster |
| Run 2 P50 / P95 | 21.1211 / 41.5565 ms | 5.5354 / 7.7243 ms | 73.79% / 81.41% faster |
| Run 3 P50 / P95 | 21.3246 / 24.0276 ms | 5.3473 / 6.7775 ms | 74.92% / 71.79% faster |

### Danger Icon Identity

| Evidence | Format plus lowercase | Borrowed fields | Result |
| --- | ---: | ---: | ---: |
| Checksum | 32,768 | 32,768 | identical |
| Allocations | 196,608 | 0 | 196,608 fewer; 100% reduction |
| Run 1 P50 / P95 | 30.8754 / 41.7101 ms | 8.8159 / 12.8159 ms | 71.45% / 69.27% faster |
| Run 2 P50 / P95 | 31.1474 / 52.7725 ms | 8.4504 / 15.0530 ms | 72.87% / 71.48% faster |
| Run 3 P50 / P95 | 32.0288 / 68.1672 ms | 8.1924 / 15.3269 ms | 74.42% / 77.52% faster |

The model isolates paint identity classification. These reductions are not whole-frame render
timings. Managed acceptance requires each exact checksum and allocation count, zero optimized
allocations, and at least 60% P50 and P95 reduction for every task.

## Acceptance

- TDD RED observed exact 4/4 failures before implementation; the same source contract passes 4/4
  after implementation.
- Python compilation, six Rust source/test formatting checks, model compilation, three calibrated
  model runs and scoped diff checks pass locally.
- The three mixed-case Rust regressions are submitted in one coordinator Cargo invocation together
  with source contracts, formatting, performance gates and the scoped diff check.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Editor23 still owns the broader UI asset, binding, accessibility, theme, icon, flow and font-atlas
authoring review. This record accepts only the three paint identity classifiers above.
