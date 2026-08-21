# Plugins02 bounded ZNN loader optimization record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md`
- Finding: `P0-04`
- Status: `validation_pending`

## Scope

- Admit hostile `.znn` header counts and byte lengths before parser allocation or weight copying.
- Replace infallible parser reservations with typed fallible reservations.
- Remove the temporary combined tensor-id vector from op decoding.

## Contract

- Artifact bytes are capped at 512 MiB.
- Weight blobs are capped at 512 MiB and op tables at 64 MiB.
- Tensor and op counts are each capped at 1,048,576.
- A declared op count must fit the encoded op table at the eight-byte minimum record size.
- `u64` offsets and lengths must convert to the host address width without truncation.
- Admission or allocation failure returns `NnModelFormatError`; no partially decoded model is published.

## Performance Gate

- The op tensor-id hot path performs two vector allocations instead of three per op, a deterministic 33% allocation-count reduction.
- Tensor ids are written once instead of into a combined vector and then copied into inputs/outputs, a deterministic 50% write-count reduction.
- The release gate uses 21 alternating legacy/optimized sample pairs and nearest-rank P95.
- Acceptance requires optimized P95 to remain within 110% of legacy P95. Measured timings remain pending the grouped coordinator validation.

## Validation

- Behavior coverage includes oversized declared op count, impossible op-count/table cardinality, oversized weight blob, binary roundtrip, and existing malformed format cases.
- The release performance marker is `PERF-MVP-PLUGINS02-ZNN-BOUNDED-LOADER`.
- Cargo compilation, behavior tests, and release measurements are intentionally queued in the multi-task Plugins aggregate; no standalone Cargo run is claimed here.

## Remaining Plan Work

- This slice closes the immediate pre-allocation count and weight-copy risk in `P0-04`; shared project-level neural memory quotas and mapped/shared immutable weight ownership remain open architecture work.
- The ONNX frontend budgets in `P0-03`, atomic artifact publication in `P0-02`, and validated executable IR in `P0-05` remain separate milestones.
