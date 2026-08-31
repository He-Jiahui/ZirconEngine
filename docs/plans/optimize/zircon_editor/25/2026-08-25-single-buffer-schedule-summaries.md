---
title: Editor25 Single-Buffer Schedule Summaries
category: zircon_editor
report_id: Editor25-single-buffer-schedule-summaries-2026-08-25
date: 2026-08-25
session_id: root-editor25-overlay-capacity-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor25 Single-Buffer Schedule Summaries

## Scope

This slice reduces allocation churn while the Editor debug reflector formats Runtime ECS schedule
impacts. It preserves impact filtering and order, stage/reason text, delimiters, empty summaries,
and all public diagnostics contracts. It does not claim to close Editor25's observation-session,
capture, timeline, telemetry, or product-authority gaps.

## Implementation

`schedule_impact_summary` previously formatted every dirty reason into its own `String`, collected
those strings into a temporary `Vec`, joined another reason string, formatted an impact string,
collected all impact strings into another `Vec`, and finally joined the returned summary.

The optimized path uses `fmt::Write` to append stage metadata and dirty reasons directly to the one
returned `String`. The shared dirty-reason formatter now follows the same single-buffer path, keeping
the existing `none` representation without creating intermediate owned strings or vectors.

The regression compares retired and optimized output for required, node-driven, filtered, empty,
single-reason, and multi-reason inputs. A source contract rejects the two retired `collect::<Vec>`
pipelines and requires direct buffered formatting.

## Performance Contract

| Evidence for 4,096 impacts x 2 reasons | Retired path | Optimized gate |
| --- | ---: | ---: |
| Intermediate owned strings per summary | 16,384 | 0 |
| Temporary vector buffers per summary | 4,097 | 0 |
| Alternating release benchmark | 11 samples x 64 summaries | optimized P95 <= 60% of retired P95 |

The benchmark emits `EDITOR25_SINGLE_BUFFER_SCHEDULE_SUMMARY_BENCH_V1` with both P95 timings,
reduction basis points, sample/iteration/impact/reason counts, and retired/optimized intermediate
String and temporary Vec counts.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and production source guards passed before
submission (apart from the repository's existing CRLF notice). One managed Editor batch covers
retired/optimized byte equivalence, the single-buffer source contract, and the ignored release
benchmark. Dynamic P95 evidence, integration SHA, and automatic WeCom performance delivery remain
coordinator-owned and pending.
