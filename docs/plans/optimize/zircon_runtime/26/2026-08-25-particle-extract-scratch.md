---
title: Runtime26 Particle Extract Scratch Reuse
category: zircon_runtime
report_id: Runtime26-particle-extract-scratch-2026-08-25
date: 2026-08-31
session_id: root-runtime26-particle-scratch-release-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime26 Particle Extract Scratch Reuse

## Scope

This slice reduces per-emitter allocation traffic in Runtime26's Scene particle extraction path,
aligned with PARTICLE-P1-027 and the plan's scale-qualification direction. It does not claim the
parent plan's world runtime, typed asset, CPU/GPU authority, renderer family, or product-scene work
is complete.

Current baseline is `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, epoch `575`.

## Implementation

Visible authored particle and world-HUD sprites are now appended directly to the frame-owned
`sprites` vector. Each emitter records its starting index and uses the resulting stable slice for
empty detection and radius calculation, preserving the existing global sort and output order.

GPU bounds use a fixed array sized to `PARTICLE_COMPONENT_IDS` instead of a per-emitter vector.
Only populated entries participate in center fallback and radius aggregation. Component parsing,
GPU frame aggregation, emitter order, bounds values, and sprite sort keys remain unchanged.

## Performance Evidence

| Evidence | Before | After / target | Reduction |
| --- | ---: | ---: | ---: |
| 100K emitters with sprite and GPU bounds data | 200K transient per-emitter buffers | 0 transient per-emitter buffers | 100.00% buffer elimination |
| Sprite output storage | per-emitter growth plus frame-vector extend | direct frame-vector append | intermediate copy/extend removed |
| GPU bounds scratch | heap-backed `Vec` with at most two entries | two-slot stack array | heap growth removed |
| Focused release P50 | measured legacy path | at least 15% faster | pending terminal evidence |
| Focused release P95 | measured legacy path | at least 5% faster and <= 500 ms | pending terminal evidence |

The ignored Windows-native release evidence runs 4 warmup pairs and 21 alternating legacy/optimized
sample pairs over 100,000 emitters. Both paths call the same sprite collector with the same JSON
input and must produce the same checksum. It prints `RUNTIME26_PARTICLE_EXTRACT_SCRATCH_BENCH_V1`
with raw nanosecond arrays, P50/P95, transient-buffer counts, checksum, reductions, and the absolute
target. Exact wall-clock evidence is accepted only from the coordinator's terminal result.

## Validation

Validation request ID: `a7600d4e79fc40138f6b905288cf95ac`.

- Historical static RED proved the storage-reuse contract rejected both legacy per-emitter vectors.
- Current RED: the new benchmark-quality contract passed the production guard and failed the three
  missing sampling, comparison, and machine-evidence guards.
- Current GREEN: `python -m unittest tools.tests.test_runtime26_particle_extract_scratch_performance_contract -v`
  passes 4/4.
- Frame-owned sprite storage, fixed GPU-bound scratch, and the paired ignored 100K-emitter release
  gate are prepared for the asynchronous coordinator batch.
- Scoped `rustfmt --check`, `git diff --check`, and the optimized source contract pass locally.
- No local Cargo lane is launched and no coordinator compile is monitored in real time.
- Final validation ticket, terminal marker values, integration commit, and WeCom delivery remain
  pending.

## Documentation Decision

The public rendering documentation does not promise the internal extraction scratch-storage
algorithm. Render output and ordering contracts are unchanged, so this scoped optimization record
is the only documentation change.

## Remaining Parent-plan Work

World-scoped runtime ownership, typed particle assets, deterministic simulation, persistent GPU
allocation, material/renderer families, history, lifecycle, budgets, and full product-scale
qualification remain open under Runtime26.
