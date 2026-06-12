# Vampire Forest Rendering And Static Batch Design

## Goal

Upgrade `examples/vampire` from a dense jungle-graveyard slice into a more forest-like third-person roguelite scene with shader-driven forest surface detail, billboard-style grass, asset-level static batches, and visually richer monster presentation.

## Approved Direction

Use project-local assets first and add a runtime extract-level static batch DTO. Broad GPU draw-call merging is larger than this example pass, but `GeometryExtract` can already expose deterministic static batch groups keyed by model, mesh, material, and render layer. For this milestone, static batching means authored asset-level merged static patches plus runtime frame-extract aggregation of repeated static instances. This lowers entity/draw pressure for the example and gives the renderer a real batch contract without blocking on WGPU draw emission work.

## Design

- Extend the existing vampire default PBR shader with procedural forest detail paths for jungle ground, foliage, and grass markers while keeping the same shader layout and material contract.
- Add a `forest_grass_billboard.zmaterial` material that marks grass with an alpha-band marker and uses double-sided opaque cards so it stays compatible with the current material pipeline.
- Add a `grass_billboard_static_batch.model.toml` model containing multiple crossed card clusters in one primitive. Scene instances of this model are the static batches.
- Place several named static grass-batch entities around the camera-visible clearing and path edges. They must stay decorative and not alter the baked navmesh walkable corridors.
- Extend `GeometryExtract` with static mesh batch metadata so repeated Static grass entities are grouped for renderer-side consumption.
- Keep monster complexity anchored to GLB primitive models already used by scene and dynamic spawns, and add tests that prevent falling back to simple capsule-only enemies for the playable path.

## Acceptance

- Project import tests parse the new grass material/model and assert multiple static grass-batch entities exist.
- The vampire shader source contains explicit forest/grass detail functions and markers.
- Scene extract includes the grass static-batch entities as renderable static meshes and groups the six grass entities into one runtime static batch.
- The runtime/offscreen screenshot path produces a nonblank forest scene with visible ground, grass, foliage, and GLB actor meshes.
