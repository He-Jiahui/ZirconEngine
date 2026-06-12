# Vampire Forest Rendering Static Batch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add forest shader detail, billboard grass, asset/runtime-extract static grass batches, and richer monster acceptance checks to `examples/vampire`.

**Architecture:** Keep visual content in project-local assets and focused vampire acceptance tests. Use the existing default PBR shader layout, TOML model/material assets, and scene mesh bindings. Add neutral `GeometryExtract::static_batches` metadata for repeated Static mesh instances, while leaving GPU draw-call merging to a later renderer pass.

**Tech Stack:** Zircon Runtime asset TOML, WGSL default PBR shader, `zircon_runtime` project/import tests, vampire scene assets.

---

## Milestone 1: Asset And Shader Implementation

- [x] Add forest/grass shader detail helpers to `examples/vampire/assets/shaders/default_pbr/default_pbr.wgsl`.
- [x] Add `examples/vampire/assets/materials/forest_grass_billboard.zmaterial`.
- [x] Add `examples/vampire/assets/models/grass_billboard_static_batch.model.toml`.
- [x] Add static grass-batch entities to `examples/vampire/assets/scenes/main.scene.toml`.
- [x] Add runtime frame-extract static mesh batch metadata keyed by model, mesh, material, and render layer.

## Milestone 2: Acceptance Tests And Docs

- [x] Extend `zircon_runtime/src/asset/tests/project/example_vampire.rs` to validate grass assets, forest shader markers, static batch scene entities, and GLB monster complexity.
- [x] Extend `zircon_runtime/src/core/framework/tests.rs` to validate Static-only mesh batch aggregation.
- [x] Update `examples/vampire/README.md` and `docs/zircon_runtime/graphics/tests/project_render.md` with the new acceptance target.
- [x] Run focused vampire import/render tests and the ignored screenshot export.
- [x] Try the standalone runtime window path and capture or report the blocking error with logs.

Current acceptance covers asset-level billboard grass batching and runtime frame-extract
static batch metadata. It intentionally does not claim that the WGPU mesh renderer now
emits one GPU draw for every static batch; the current mesh resource path no longer
retains CPU vertex payloads after upload, and `MeshDraw` still binds one model uniform
for one indexed draw.
