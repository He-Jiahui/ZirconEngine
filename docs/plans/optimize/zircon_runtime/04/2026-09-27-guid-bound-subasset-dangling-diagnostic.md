---
title: Runtime04 GUID Bound Missing Subasset Diagnostic
category: zircon_runtime
report_id: Runtime04-guid-bound-subasset-dangling-diagnostic-2026-09-27
date: 2026-09-27
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: not_measured
---

# Runtime04 GUID bound missing subasset diagnostic

The original P1-8 parent fallback had already been removed: an exact labeled
locator miss returned `DanglingSubasset` with sorted source candidates. One
remaining branch still used the persisted path hint when a known parent GUID
carried a requested label. If that hint had disappeared, the resolver returned
`Conflict`; if another asset occupied it, the error could use that asset's
candidates or return `Conflict` when it had the requested label. Migration maps
`Conflict` to `RegistryConflict`, losing the missing-label diagnostic.

The known-parent-GUID branch now checks the requested label against the GUID
entry's source. A missing label returns `DanglingSubasset` with the persisted
GUID, original hint and label, and sorted labeled entries from that same
source, regardless of the hint's current owner. An exact labeled entry under
the parent still produces `Conflict` because its UUID differs from the
persisted parent GUID. Already labeled GUIDs with mismatched labels also stay
`Conflict`. The missing-GUID path-hint branch and path-only repair are unchanged;
this slice never changes or infers a semantic GUID/subasset identity.

The existing resolver fixture now covers missing and occupied stale hints,
including an occupied source with the same requested label; same-source exact
labels and already labeled GUID mismatches still reject. It also retains the
missing-GUID live-hint and adds occupied/stale-hint cases. The existing
migration error mapping already maps `DanglingSubasset` to
`DanglingReference`, while genuine `Conflict` remains `RegistryConflict`.

Run the existing resolver test
`resolution_keeps_guid_authoritative_and_reports_path_candidates` and the
related `retired_migration_` resolver tests in the next grouped managed Runtime
test batch. Rustfmt and scoped diff checks are the only local checks for this
slice. No Cargo or product measurement has run; behavior acceptance and any
performance claim remain pending. The previous migration-index performance
profile and Runtime04 pack/export gates are separate and are not closed here.
