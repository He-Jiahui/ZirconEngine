# PFO-4d1r Immutable Fullscreen Parameter Buffer

Status: `source_implemented_static_checks_passed_dynamic_wgpu_validation_pending`

Date: 2026-08-27

## Structural Review

`FullscreenPassParameterBindings` currently creates a persistent uniform buffer and immediately
calls its own public-to-graphics `write(&wgpu::Queue, ...)` method. The complete current call graph
has one constructor and no later `write` consumer. Its sole product instance belongs to the
`motion_vector_tile_max_pass_plan()` returned by `OnceLock`; the parameter layout and value
`tile_span = [2, 2, 0, 0]` are immutable for the lifetime of the post-process resource bundle.

Keeping `Queue`, `COPY_DST`, a mutable byte vector, a copied String/discriminant layout, and a
general update method therefore advertises a dynamic data path that the feature neither needs nor
uses. This is an ownership flaw, not a measured frame-time bottleneck.

## Reference Alignment

Unreal's `PostProcessMotionBlur.cpp` allocates typed motion-blur pass parameters through
`GraphBuilder.AllocParameters<...>()` and publishes them with the corresponding RDG pass. Zircon's
tile-span value is even more static: it is part of an immutable built-in pass plan. The closest
current WGPU representation is one mapped initialization owned by the persistent binding bundle,
not a long-lived raw queue writer.

## Design

1. serialize the non-empty fullscreen parameter plan once during binding construction;
2. create the uniform with `wgpu::util::DeviceExt::create_buffer_init` and `UNIFORM` usage only;
3. retain the buffer beside the bind group as the explicit persistent lifetime owner;
4. remove the constructor's queue argument, `COPY_DST`, dynamic `write`, upload staging vector, and
   runtime layout-match cache;
5. keep the neutral fullscreen plan and bind-group ABI unchanged.

## Acceptance Boundary

Focused source checks require exactly one mapped initialization, zero `wgpu::Queue`, zero
`write_buffer`, zero `COPY_DST`, and no dynamic `write` method in production. Scoped rustfmt and diff
checks are required. Cargo, WGPU, product PNG, RenderDoc, profile, and power remain deferred; this
slice makes no frame-time claim.

## Completed Source Work

1. Construction now serializes the immutable plan once and creates the persistent uniform through
   `DeviceExt::create_buffer_init` with `UNIFORM` usage only. The buffer remains explicitly retained
   beside its bind group.
2. The queue constructor argument, `COPY_DST`, mutable upload vector, copied String/discriminant
   layout, layout-match branch, and unused dynamic `write` method were removed. The motion-vector
   tile-max pass still consumes the same group/binding and shader parameter bytes.
3. Focused source counts passed: mapped initialization `1`; `wgpu::Queue` `0`; `write_buffer` `0`;
   `COPY_DST` `0`; dynamic write method `0`; legacy constructor queue argument `0`; external
   fullscreen parameter writes `0`. The owner shrank from 225 to 154 lines.
4. Focused rustfmt and scoped diff checks passed. Cargo, WGPU, PNG, RenderDoc, profile, memory, and
   power were not run, so no runtime or performance improvement is claimed.
