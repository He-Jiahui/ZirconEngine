---
title: Editor15 Projection Key Clone Elision
category: zircon_editor
report_id: Editor15-projection-key-clone-elision-2026-08-25
date: 2026-08-25
session_id: root-editor15-projection-key-clones-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor15 Projection Key Clone Elision

## Scope

This batch optimizes two allocation paths in the typed read-only Material and RendererData
projections retained by Editor15. It does not claim the parent plan's product entry, operation
factory, document transaction, compiler, artifact, preview, save, VFX, or full qualification work
is complete.

## Implementation

Material property and texture-slot projection now stores borrowed shader-schema names in its
temporary `BTreeSet<&str>`. The output rows still own their stable display names, while the
duplicate name clone previously retained only for membership checks is removed.

RendererData diagnostic grouping now probes each material or shader bucket with the borrowed
`AssetReference`. It clones a resource key only when inserting a distinct bucket, instead of
cloning the same locator-bearing key for every diagnostic before `HashMap::entry` discovers that
the bucket already exists. Diagnostic order and returned map/value types are unchanged.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 10K shader properties + 10K texture slots | 40K schema-name clones | 20K output-row name clones | 50.00% schema-name clone reduction |
| 100K diagnostics sharing one material and one shader | 200K `AssetReference` key clones | 2 unique key clones | 99.999% key-clone reduction |
| Material projection focused release target | unbounded | <= 750 ms | pending terminal evidence |
| Diagnostic grouping focused release target | unbounded | <= 500 ms | pending terminal evidence |

The ignored Windows release evidence prints
`EDITOR15_MATERIAL_PROJECTION_BORROWED_KEYS_BENCH_V1` and
`EDITOR15_DIAGNOSTIC_GROUPING_UNIQUE_KEYS_BENCH_V1` with workload sizes, legacy and optimized
clone counts, reduction percentages, elapsed microseconds, and targets. Exact elapsed time is
accepted only from the coordinator's terminal result.

## Validation

- RED recorded owned schema-name clones and per-diagnostic `HashMap::entry(key.clone())` calls.
- Existing projection tests continue to cover row content, diagnostic ownership, grouping, and
  duplicate shader references.
- New source contracts require borrowed schema names and borrowed grouping probes.
- Static GREEN, scoped `rustfmt --check`, and `git diff --check` pass locally.
- Both ignored release workloads are prepared for one Editor batch validation.
- Final terminal marker values, integration commit, and WeCom delivery remain pending.

## Documentation Decision

No public authoring or runtime contract changes. This numbered optimization record is sufficient
for the internal allocation change.

## Remaining Parent-plan Work

Editor15 still requires capability truth, real operation factories, canonical graph schemas,
transactional documents and save, semantic compilation, immutable artifacts, live runtime preview,
typed diagnostics, jobs, plugin lifecycle, large-asset qualification, and accessibility coverage.
