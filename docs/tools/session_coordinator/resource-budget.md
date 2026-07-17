---
related_code:
  - tools/session_coordinator/resource_budget.py
  - tools/session_coordinator/cpu_burst.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/control_plane/snapshot.py
implementation_files:
  - tools/session_coordinator/resource_budget.py
  - tools/session_coordinator/cpu_burst.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/control_plane/snapshot.py
plan_sources:
  - docs/superpowers/plans/2026-07-17-coordinator-adaptive-cpu-burst-lanes.md
  - user: optimize Coordinator validation throughput without global Session blocking
tests:
  - tools/session_coordinator/tests/test_resource_budget.py
  - tools/session_coordinator/tests/test_cpu_burst.py
  - tools/session_coordinator/tests/test_database.py
doc_type: module-detail
---

# Adaptive CPU burst resource budget

## Purpose

The normal CPU validation lane owns a reusable Cargo target and remains the
default path. This module defines the evidence needed before an optional,
isolated burst lane can exist. It does not register Sessions, start Cargo,
change queue order, create target directories, or close global admission.

## Resource evidence

`WindowsResourceProbe` reads cumulative Windows system times twice and derives
CPU use from the idle fraction of the total kernel-plus-user delta. A missing,
negative, or otherwise invalid delta is treated as 100% busy, which safely
denies a burst rather than guessing that the machine is idle. It also reads
available physical memory from `GlobalMemoryStatusEx`.

`burst_decision` requires exactly three valid samples, all at or below 80% CPU,
at least 12 GiB free memory in every sample, at least 100 GiB free on the
isolated target drive, and no active burst reservation. Its result is one of
`allowed`, `burst_active`, `disk_headroom`, `cpu_headroom`, or
`memory_headroom`. These reasons remain scheduler-internal; the browser exposes
only bounded burst WIP and declared pending candidates, so a page refresh does
not perform resource sampling or create a Session-blocking path.

## Target policy boundary

`select_cpu_burst` is pure. It chooses an `E:\cargo-targets\zircon-engine\burst\<reservation>` target only when a request
is explicitly marked eligible, is a CPU `cargo check`, has no caller-supplied
target, and the resource decision is allowed. All other requests remain warm.
This prevents a test, GPU command, or already-bound target from silently
changing caching or disk semantics.

## Current rollout state

Schema 48 persists `execution_mode` and `burst_eligible`, with historical rows
defaulting to warm. It preserves one active warm CPU/GPU lane and adds a separate
single active CPU burst index. The source reservation and consume paths now use
the policy: an eligible check behind an occupied warm lane becomes an ephemeral
burst job only after admission, while denial leaves it untouched in warm FIFO.
The running coordinator remains Schema 47 until its next no-Cargo rollover, so
production behavior is still warm-only until the controlled rollout milestone is
complete.

The control snapshot separately labels warm and burst reservations, reports a
fixed `1` burst capacity, and counts warm pending checks that opted in as
eligible candidates. A full burst slot or a denied admission has no effect on
Session registration, file work, Failure handling, or the normal warm FIFO.

## Test coverage

The resource tests cover valid and invalid CPU deltas, all admission reasons,
and a complete three-sample allowance. The burst tests prove that only an
eligible target-free `cargo check` selects the isolated path. The database test
verifies the mode/eligibility columns and both warm/burst uniqueness indexes.
