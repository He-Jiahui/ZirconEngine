---
title: Editor625 Borrowed Project Package Membership
category: zircon_editor
report_id: Editor625-borrowed-project-package-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor625 Borrowed Project Package Membership

Project plugin selection now builds a preallocated borrowed `HashSet<&str>` from the manager entry
slice. The previous membership index cloned every package ID into a `BTreeSet<String>` before
filtering the manifest selections.

Manifest traversal order, EditorHost target evaluation, duplicate selection diagnostics, and the
ordered `BTreeMap` enablement result remain unchanged. A focused behavioral test keeps the first
duplicate package diagnostic, and source coverage locks borrowed storage with zero owned clones.

The ignored Windows Release benchmark emits
`EDITOR625_BORROWED_PROJECT_PACKAGE_MEMBERSHIP_BENCH_V1` over 17 alternating sample pairs with
32,768 long package IDs. The gate requires borrowed hash-membership P95 to be at most 35% of the
owned ordered-tree P95.

No direct Cargo validation was run. The coordinator owns combined Runtime625/Editor625 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor625 is prepared with Runtime625 under request
`runtime625-editor625-frame-project-membership-performance-20260901io-v1`. Receipt, validation
ticket, measured P95, pushed SHA, and notification result are recorded only after coordinator
completion.
