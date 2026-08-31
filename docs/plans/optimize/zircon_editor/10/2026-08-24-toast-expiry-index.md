---
title: Editor10 Toast Expiry Index Optimization
category: zircon_editor
report_id: Editor10-toast-expiry-index-2026-08-24
date: 2026-08-24
session_id: root-editor10-notification-projection-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor10 Toast Expiry Index Optimization

## Scope

This slice replaces full-map expiry scans in the bounded toast center with a dedicated deadline
index. It does not claim the parent plan's visible-lifetime policy, monotonic delivery sequence,
severity reserve, duplicate aggregation, durable journal, typed action, or product notification
entry point is complete.

## Implementation

The center now owns one mutex-protected state containing the identity map and a
`BTreeMap<Duration, BTreeSet<NotificationId>>` expiry index. Publish and snapshot cleanup inspect
only the earliest deadline, pop expired deadline groups, and remove their exact identities from the
authority map. Multiple notifications with one deadline are retired as one group.

The boundary remains unchanged: `expires_at == now` is expired, while a later deadline remains
visible. Snapshot order is still the existing `NotificationId` order. Each live toast now retains
one additional shared-string identity in the expiry index; this bounded metadata cost replaces an
O(live toasts) scan on every publish and cleanup.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Publish 10,000 live toasts and take one snapshot | 50,005,000 expiry checks | <= 10,001 earliest-deadline probes; <= 5 s | >= 99.98% probe reduction |
| No-expiry publish cleanup | O(live toasts) | O(1) earliest-deadline check | full-map `retain` removed |
| Expire one shared deadline group | O(all live toasts) | O(expired ids log live ids) | exact grouped removal |

The ignored Windows-native release evidence prints `EDITOR_TOAST_EXPIRY_BENCH_V1` with exact probe
counts, reduction basis points, and elapsed nanoseconds. Exact runtime values are accepted only
from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, grouped deadline/boundary behavior, existing
  toast regressions, and ignored release evidence are prepared for a shared coordinator batch with
  the Editor11 rolling-file segment cache.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, and commit integration remain pending.

## Remaining Parent-plan Work

Toast lifetime still starts at publish rather than first visibility, current selection still uses
identity order, duplicate fixed IDs still fail instead of aggregating, and expired/error records do
not yet flow into a durable searchable notification journal.
