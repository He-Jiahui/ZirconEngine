---
title: Editor Scene Selection Current Source and Duplicate Toggle M0
date: 2026-08-23
scope:
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/scene/selection
status: static_complete_dynamic_pending
canonical_owner:
  - docs/plans/optimize/zircon_editor/74-editor-scene-selection-authority-primary-active-range-filter-named-set-history-document-world-scope-lifecycle-performance-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/TypedElementRuntime/Public/Elements/Framework/TypedElementSelectionSet.h
  - dev/UnrealEngine/Engine/Source/Runtime/TypedElementRuntime/Private/Elements/Framework/TypedElementSelectionSet.cpp
---

# Editor Scene Selection Current Source and Duplicate Toggle M0

## 1. Coverage

The current `zircon_editor/src/scene/selection` surface is **6/6 Rust files**, **397 physical / 332 non-empty lines**, **12,161 bytes**, and **7 test markers**. Its workspace-relative `path + NUL + raw bytes + NUL` SHA-256 is `67f89166858414c94aefd8b03f9826f5d8f04932e0a33383cae5e34b4e37b4d7` after the M0 below. All six files and Editor74's cross-product findings were read directly.

The clean `zircon_editor/src/scene/mod.rs` root was also reviewed: **1/1 file**, **5 physical / 4 non-empty lines**, **107 bytes**, fingerprint `6322142bbff690d8e19b30c62627772f831b8c8b85ca3871afd18f5a72e27be6`. It only declares `modes`, `selection` and `viewport`; it has no independent runtime behavior.

The selection folder was clean before this pass. The adjacent `scene/modes`, viewport controller, pointer and picking trees contain extensive foreign/shared migration edits and are preserved; this slice does not claim currentness for those 138 files.

## 2. Accepted foundation

`SelectionModel` keeps separate Edit/Play domains, stable insertion order through `IndexSet`, a primary item constrained to membership, per-domain generation and whole-model revision. Replace, Extend, Toggle and Clear report whether they changed state, and no-op Replace/Clear suppress revision changes. These invariants should move into the future authority rather than be discarded.

## 3. M0: duplicate toggle canonicalization

Before this pass, `apply_active` collected arbitrary input into a `Vec` and Toggle applied every occurrence. `[20, 20]` added then removed entity 20, returned `changed=true`, incremented domain generation twice and model revision twice, but left the membership unchanged. Downstream generation-driven projection could perform two units of work for a net no-op, and the final result contradicted set-based selection semantics.

The mutation entry now collects into an insertion-ordered `IndexSet` before dispatch. Each unique target is toggled at most once; Replace and Extend retain their existing order/dedup semantics. For `N` input occurrences and `U` unique entities, Toggle work and revision increments change from `N` to `U`; the deterministic regression case changes two flips/two revisions to one flip/one revision. The new test locks membership `[10, 20]`, primary `20` and exactly one revision increment.

This is a bounded correctness/work reduction, not the planned Selection Authority. It does not close stale IDs, document/world identity, range/eligibility, transaction or consumer publication issues.

## 4. Unreal constraint

Unreal `TypedElementSelectionSet.cpp` batches selection synchronization around array/list operations, reserves for batch select, returns an aggregate changed disposition and exposes normalized selection as a separate projection. The applicable constraint is canonical set input plus one observable batch result. Zircon should not copy UObject or legacy notification mechanics.

## 5. Remaining structural plan

1. Introduce one per-document/session `SceneSelectionAuthority` with qualified world/document generation, typed request/admission/receipt and immutable snapshot/delta publication.
2. Route viewport, hierarchy, automation, transaction and plugin mutation through one atomic commit. Eliminate public direct mutable access and bool results that are later overwritten as always-changed.
3. Preserve 64-bit/opaque target identity end to end; remove the binding path's `u64 -> u32 -> u64` narrowing and reconcile stale/deleted targets in the authority.
4. Compile policy projections for FullOrdered, Primary, TopLevelRoots, TransformRoots, CopyDeleteRoots and RenderableTargets instead of letting each consumer rescan/reinterpret selection.
5. Add Replace/Extend/Toggle/Subtract/range/query/named-set/history gates with duplicate-heavy and 1/10k/1m candidate workloads. Record request p50/p95/p99, visited/unique targets, allocations, revision/delta count, observer work and UI frame effect.
6. WPR/ETW on a current-source editor measures pointer/region selection CPU, locks, allocations, publication fan-out, RSS and energy. RenderDoc verifies highlight/picking parity for the same selection generation only; it cannot prove selection CPU performance.

## 6. Verification and status

- Six-file `rustfmt --check` and scoped `git diff --check` passed.
- Two existing Python selection performance contracts passed; they cover option-source early return and one-pass shell content selection, not the Rust duplicate-toggle behavior.
- The new Rust regression test was added but not executed because a managed Cargo validation session is unavailable. No real editor/WPR/RenderDoc measurement was produced.
- M0 is statically implemented and dynamically pending. Editor74 remains the canonical structural owner, and no milestone commit or WeCom acceptance message is due yet.
