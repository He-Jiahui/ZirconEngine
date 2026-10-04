---
title: Runtime Skeleton Path Borrowed Segments
category: zircon_runtime
report_id: Runtime877-skeleton-path-borrowed-segments-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime877 Skeleton Path Borrowed Segments

## Finding and optimization

Animation skeleton path construction previously cloned each ancestor name into
a temporary `Vec<String>` before reversing and joining. It now retains borrowed
`&str` segments in the same traversal and joins them into the required final
path. The parent walk, missing-index/ancestor `None`, empty names, duplicate
names, Unicode text, and slash ordering are unchanged. The temporary vector
and required result remain; only per-ancestor owned copies are removed.

## TDD and deterministic evidence

The combined Runtime877/Editor896 contracts were RED `1/8` and GREEN `8/8`.
A lower regression compares the retired path on valid and invalid ancestry,
empty names, duplicate names, CJK, and emoji. For 4,096 paths of depth 32,
the deterministic model removes `131072` cloned ancestor strings; it does not
claim to remove reference slots or the final path. Ignored
`RUNTIME877_SKELETON_PATH_BORROWED_SEGMENTS_BENCH_V1` records 101 alternating
Release p50/p95/p99 pairs and requires optimized p95 <= 110% of the retired
path; it has not yet been run.

The pair plus adjacent animation/export source contracts pass `48/48`.
Exact Rustfmt, Python bytecode compilation, and scoped diff checks pass. There
was no per-task Cargo run. A combined current-source Runtime/Editor/App batch
v24 was launched asynchronously as PID `33516` at
`2026-09-21T23:12:59.6906963+08:00`; no v24 receipt has been read or monitored.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/manager/pose.rs` | `152BD4CE4E4F8F1DB28C888F605E8FFE114E5660ED5C3B68BD54ED8E57758D12` |
| `zircon_runtime/src/animation/manager/pose/borrowed_path_tests.rs` | `FCE9AF07C9299D3C9258EB1C2F871BD0FC05DB3B515968B4649EE4572136AC77` |
| `tools/tests/test_runtime877_skeleton_path_borrowed_segments_performance_contract.py` | `D8E31CC8CEE98FB8F5B5BA3EB5D160599FA1548C97FB3CE1015A73E556602322` |

## Acceptance boundary

A single bounded v24 receipt read after independent work confirms the managed
Runtime development build exited `0` on its sealed source snapshot. The
static clone-elimination target is met; Rust lower tests, ignored Release
percentiles, allocator measurement, and animation-product p50/p95/p99 remain
pending. This `-SkipTest` build does not satisfy those acceptance gates.
