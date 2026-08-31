---
title: Editor06 Plugin Registration Index
category: zircon_editor
report_id: Editor06-plugin-registration-index-2026-08-25
date: 2026-08-25
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Plugin Registration Index

## Scope

This slice removes repeated full registration scans from mutable plugin-catalog lifecycle work. It
does not claim the parent plan's durable enable/disable transaction, unified management authority,
reload generation publication, dependency solver, settings contribution, or scalable product UI.

## Implementation

`EditorPluginCatalog` now maintains a `package id -> first registration index` map alongside its
ordered registration vector. Registration records the first index for an ID, preserving the former
`.find` behavior if malformed or duplicate input reaches the catalog. Project-scoped report
replacement rebuilds the index once after the generation changes.

Lifecycle event recording clones the optional plugin handle before borrowing the indexed mutable
registration. Lifecycle success, lifecycle failure, and package fault queries share one indexed
lookup helper. Catalog order, duplicate admission diagnostics, lifecycle callback order, immutable
snapshot publication, and project replacement semantics are unchanged.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 100,000 lifecycle queries, target last among 1,000 plugins | 100,000,000 registration candidate checks | 100,000 registration-index probes; <= 3 s | 99.9% candidate-check reduction |
| Record one lifecycle event | scan up to all registrations | one index lookup plus exact mutable Vec access | O(n) lookup becomes O(log n) |
| Project catalog generation replacement | retain/extend plus later repeated scans | retain/extend plus one O(n) index rebuild | rebuild cost moved to mutation boundary |

The ignored Windows-native release evidence prints
`EDITOR06_PLUGIN_REGISTRATION_INDEX_BENCH_V1` with plugin count, lookup count, legacy candidate
checks, indexed probes, reduction basis points, elapsed nanoseconds, and the elapsed-time ceiling.
Exact elapsed time remains pending coordinator terminal evidence.

The index retains one owned package ID and one `usize` per unique registration. This bounded-by-
catalog metadata cost is paid once per generation to remove repeated lifecycle scans.

## Validation

- Exact Rustfmt and scoped `git diff --check`: passed.
- Project report replacement, indexed lifecycle recording, and the release evidence gate are
  prepared for a shared Runtime/Editor coordinator batch.
- No local Cargo lane was launched and no compilation is being monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Remaining Parent-plan Work

The catalog and published snapshot still maintain separate read indexes, while product plugin
management still has the authority, persistence, reload, dependency, settings, filtering,
virtualization, and accessibility gaps listed by the parent plan. Those milestones remain open.
