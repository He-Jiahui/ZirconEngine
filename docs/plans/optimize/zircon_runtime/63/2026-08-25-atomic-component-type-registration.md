# Runtime63 Atomic Component Type Registration Optimization Record

- Date: 2026-08-25
- Owner: `root-runtime63-atomic-component-registration-20260825`
- Source plan: `docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md`, RSR-P0-001
- Status: implementation complete; combined managed validation pending

## Problem

`World::register_component_type` published a descriptor to the live
`ComponentTypeRegistry` before the stricter reflection registration was
validated. A duplicate reflected field or another late validation failure
therefore left a descriptor without its reflection adapter, changed the schema
generation, and made an immediate retry fail as a duplicate type.

The straightforward transactional repair would clone both live catalogs,
validate and mutate the copies, then swap them into the World. That preserves
correctness but copies all existing descriptors and reflection registrations
for every added type.

## Change

- `TypeRegistry` now separates fallible `validate_new_registration` from
  infallible `publish_prevalidated`; ordinary `register` composes the same two
  phases.
- `World::register_component_type` validates the component descriptor and
  reflection registration before touching live state.
- The existing component-registry delta stages one target-local dynamic
  `ComponentId` without publishing it. After all checks pass, the component ID,
  descriptor, and reflection registration publish through no-fail paths.
- A rejected duplicate-field registration must preserve all three registries
  and both catalog generations, then allow the same type ID to register
  successfully with a valid schema.

## Deterministic Performance Evidence

The release workload starts with 512 registered component types and adds 64
more types per sample. The safe clone/swap baseline clones both existing
catalogs for each addition; the delta-preflight implementation copies no
existing catalog entries.

| Measure | Clone/swap baseline | Delta preflight | Change |
|---|---:|---:|---:|
| Initial existing catalog entries | 1,039 | 1,039 | 512 dynamic descriptors + 512 dynamic reflections + 15 builtin reflections |
| Existing catalog entry copies per 64-type batch | 70,528 | 0 | -100% |
| Registration complexity for batch size `B`, catalog size `N` | O(B * (N + B)) copies | O(B log(N + B)) indexed admission | removes whole-catalog copy term |
| Alternating release sample pairs | 11 | 11 | same workload |
| Nearest-rank P95 | pending | pending | optimized <= 60% of baseline |
| Optimized 64-type batch budget | n/a | pending | <= 3 seconds |

The benchmark emits
`RUNTIME63_ATOMIC_COMPONENT_REGISTRATION_BENCH_V1` with every raw sample,
P50/P95 values, and the exact entry-copy counts. Timing remains pending the
managed Windows release batch and must not be reported as passing before its
terminal receipt.

## Acceptance

- Duplicate reflected fields return an error without changing descriptor,
  reflection, component-ID, or generation state.
- The rejected type ID is immediately reusable by a valid registration.
- Existing successful callers still publish a descriptor, reflection adapter,
  and dynamic component ID together.
- The managed release command runs the Runtime63 registration correctness and
  ignored performance tests together with exact-file formatting and diff
  checks; no per-task Cargo validation is used.

## Remaining Scope

This slice closes the public half-registration failure in RSR-P0-001. It does
not add provider generations, stable field IDs, schema migration, compiled
property plans, or Editor catalog receipts from later Runtime63 milestones.
