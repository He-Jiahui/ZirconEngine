---
related_code:
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/server.py
implementation_files:
  - tools/session_coordinator/patches.py
plan_sources:
  - user: 2026-07-17 remove global coordinator blocking while preserving safe local queue recovery
tests:
  - tools/session_coordinator/tests/test_server.py
doc_type: module-detail
---

# Deferred patch queue resilience

Queued patches are durable, but they are not a right to mutate the shared
workspace after their Session becomes stale, completed, archived, cancelled, or
finalizing. Queue processing skips such a patch and leaves it `queued` for an
explicit Session resume or other owner action.

This is deliberately non-fatal to the triggering operation. In particular,
`lease.release` first releases the caller's path and then opportunistically
processes eligible queued patches. An ineligible patch must never turn that
successful release into an error response or keep the caller's lease until its
TTL expires. Conflict, changed-base, scope, and normal patch-application checks
remain unchanged for writable Sessions.
