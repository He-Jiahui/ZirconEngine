---
title: Editor13 Drawer Tab Dedup Index
category: zircon_editor
report_id: Editor13-drawer-tab-dedup-index-2026-08-25
date: 2026-08-25
session_id: root-editor13-drawer-tab-dedup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Drawer Tab Dedup Index

## Scope

This slice optimizes legacy activity-drawer canonicalization for Editor13's canonical schema and
10K-tab scale direction. It does not claim the parent plan's layout transaction, restore,
migration, durability, plugin placeholder, or native-window work is complete.

## Implementation

Legacy `BottomLeft` and `BottomRight` drawer rows still merge into the canonical `Bottom` slot in
stable map and tab order. The merge now builds one capacity-sized `HashSet` from existing tab IDs
and admits each incoming tab with a single indexed membership operation. The output `Vec` remains
the ordering authority, so hash iteration order cannot affect persisted or projected layout order.

Duplicate incoming tabs remain suppressed and the first occurrence remains authoritative. Existing
tabs are not reordered or normalized beyond the pre-existing migration behavior.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K existing + 10K incoming tabs, 5K overlap | 100,000,000 string comparisons | 20,000 membership-index probes; <= 500 ms release | 99.98% membership work reduction |
| Full existing-tab scan per incoming tab | O(N x M) | O(N + M) expected | quadratic migration path removed |
| Output order | first-seen order | first-seen order | unchanged |

The ignored Windows-native release evidence prints `EDITOR13_DRAWER_TAB_DEDUP_BENCH_V1` with input,
overlap, merged count, legacy comparison count, indexed probe count, and elapsed microseconds.
Exact wall-clock evidence is accepted only from the coordinator's terminal result.

## Validation

- Static RED proved the old loop still called `Vec::contains` for every incoming tab.
- First-seen ordering, duplicate suppression, the indexed source contract, and the ignored release
  scale gate are prepared for a multi-task coordinator batch.
- Scoped `rustfmt`, `git diff --check`, and the membership-index/source marker checks pass locally.
- No local Cargo lane is launched and no coordinator compile is monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Documentation Decision

The retained layout documentation does not promise the internal duplicate-membership algorithm.
Canonical drawer semantics and stable tab order are unchanged, so this scoped optimization record
is the only documentation change.

## Remaining Parent-plan Work

Transactional layout commands, all-or-nothing restore, schema v2 migration, input budgets,
unknown-plugin placeholders, durable profiles, monitor-aware placement, and full 10K-tab restore
qualification remain open under Editor13.
