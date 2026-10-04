---
title: Editor04 Borrowed Asset Folder Membership
category: zircon_editor
report_id: Editor04-borrowed-folder-membership-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Borrowed Asset Folder Membership

## Scope

The visible-asset projection and catalog delta patch paths previously derived an owned parent
folder identifier for every candidate asset. This slice keeps the existing `res://` and
`package://` semantics but compares the parent path directly against the selected folder ID.
The owned `parent_folder_id_for_locator` helper remains for navigation and other callers that
need a published `String`.

## Implementation

`asset_belongs_to_folder` now delegates to `locator_parent_matches_folder`. The helper uses
borrowed `strip_prefix` and `rsplit_once` views, so the hot path performs no parent-path heap
allocation. Root locators, package locators without a slash, arbitrary non-resource locators,
and empty/mismatched folder IDs retain the previous behavior.

## Deterministic work model

The old predicate allocated one parent-folder `String` for each candidate in both full visible
projection and incremental catalog patching. The new predicate performs bounded slice comparisons
and allocates nothing itself; the surrounding projection still owns the published item snapshots.
This is structural allocation evidence, not a product CPU/RSS or frame-latency measurement.

## Validation

- Rust regressions cover resource roots, nested resource folders, package folders, package-root
  edge cases, arbitrary locators, negative folder matches, and UTF-8 substring-boundary
  equivalence with the previous ASCII-folded `contains` behavior.
- A source regression requires the borrowed predicate and rejects a call to the owned parent
  projection in the asset-filter helper.
- The focused Runtime/Editor asset and Taffy contract batch passed `53/53`; scoped Rustfmt and
  `git diff --check` passed for the changed source, tests, and records.
- Managed Cargo and Windows Release p50/p95/p99 CPU, allocation, and RSS evidence remain pending.

## Remaining Parent Work

Editor04 still owns indexed token/kind/tag queries, lazy expanded-branch generations, independent
browser navigation state, provider capabilities, import/reimport orchestration, preview policy,
and product-scale catalog qualification.
