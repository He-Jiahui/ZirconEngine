---
title: Runtime91 Shader Import Provider Dedup Optimization
category: zircon_runtime
report_id: Runtime91-shader-import-provider-dedup-2026-08-24
date: 2026-08-24
session_id: root-runtime91-shader-dependency-dedup-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime91 Shader Import Provider Dedup Optimization

## Scope

This slice removes quadratic URI deduplication from shader import dependency projection. It
advances Runtime91's shader-module dependency path without changing include ownership, ambiguous
provider rejection, dependency ordering, targeted replacement, or registry publication.

## Implementation

`ShaderImportDependencyIndex::dependency_locators` now projects each uniquely owned include to its
provider `AssetId` first. A request-local ID set admits the provider once, after which its locator is
cloned into the ordered output. Repeated import paths and different paths owned by the same provider
therefore retain first-provider order while avoiding repeated URI allocation and full-string scans.

Ambiguous include paths remain excluded before admission. A stale provider ID with no shader record
still produces no dependency, and the request-local set creates no cross-generation state.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| 4,096 imports / 1,024 unique providers | 2,098,176 linear URI comparisons | 4,096 ID hash admissions | 99.8048% fewer admission operations |
| Provider URI clones | 4,096 | 1,024 | 75.0000% clone reduction |
| Output ordering | first provider occurrence | first provider occurrence | unchanged |
| Release p95 | dynamic evidence pending | <= 50% of legacy p95 | coordinator release gate |

The ignored Windows-native release evidence alternates 21 legacy/optimized sample pairs and prints
`RUNTIME91_SHADER_IMPORT_PROVIDER_DEDUP_BENCH_V1` with exact p95 nanoseconds, import/provider counts,
URI comparison counts, ID admissions, and URI clone reduction. Dynamic elapsed time is accepted
only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, provider-order regression, and the production
  ID-admission source contract are performed before coordinator submission.
- The provider regressions and ignored release evidence are grouped with the Runtime91 material
  override index and shared include-analysis slices in one three-task coordinator batch; no
  per-task Cargo lane is launched.
- Terminal marker values, commit integration, optimization-record finalization, and automatic WeCom
  delivery remain pending.

## Remaining Parent-plan Work

Runtime91 still owns shader source authority, module-graph compilation, reflection artifacts,
permutation and pipeline services, persistent caches, prewarm, hot reload, prepared material
integration, and product-scale qualification. Those parent milestones remain separate and are not
claimed complete by this dependency projection optimization.
