---
status: in_progress
---

# Runtime Root Surface Validation Worklist

## Root and generated-code obligations

Keep core/runtime and core/framework ownership explicit. Root facades must not re-export graphics module aliases through pub(crate) use graphics::{ or mask them with #[allow(unused_imports)]. SceneRenderer, GraphicsError, WgpuRenderFramework and ViewportFrame consumers use their real graphics owners.

The spine inventory covers config_store.rs, frame_clock.rs, time, job_scheduler.rs/tasks, error, event_bus.rs, lifecycle.rs, state, modules and diagnostics; channel_util.rs belongs to framework primitives. Each root/facade decision stays explicit. core_module_tree_matches_decided_spine_shape and lib_rs_declares_no_graphics_module_aliases_at_crate_root remain independent gates.

Generated output needs `// @generated <generator> - do not edit by hand`. Domain terminology and handwritten generator templates (export_build_plan/generated_files.rs and related templates) are not generated artifacts. Migrate template and consumer in the same slice. platform_host_files/ and root_surface.md keep their actual ownership.

### Recorded inventory targets

The core_spine_root_generated_boundary snapshot is dated 2026-08-24. Retain its exact metric labels below as guard targets, not as a new scan. text and operation are public runtime-module entries; runtime_diagnostics is feature-independent private support. Cargo acceptance remains pending.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Recorded targets

These retained targets belong to the original source snapshots, not a new measurement. The source dates/identities below keep distinct inventories separate.

- `crate-visible graphics alias debt 0/0`; source 2026-08-24:L52.
- `runtime root public modules 21/21`; source 2026-08-24:L52.
- `generated export templates 10/10`; source 2026-08-24:L52.
- `missing_guard_test_anchors = []`; source 2026-08-24:L52.
- `generated-code guard tests 7/7`; source 2026-08-24:L52.
- `generated allowed adapters 6/6`; source 2026-08-24:L52.
- `root_surface guard tests 6/6`; source 2026-08-24:L52.
- `guard_test_anchor_count = 26`; source 2026-08-24:L52.
- `generated migration debt 0/0`; source 2026-08-24:L52.
- public `pub use` sites 2/2; source 2026-08-24:L52.
- `core public modules 5/5`; source 2026-08-24:L52.
- `generated behavior 6/6`; source 2026-08-24:L52.
- `core root entries 6/6`; source 2026-08-24:L52.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- `mirror_docs_guard_present = true`, `root_entries guard tests 13`, `retired core root entries 0`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `02-core-spine-and-root-surface.md`, SHA-256 `0c92a71ba762cf9bdf7b114a589a35eb2add738d20c135a7299637254f6fd24d`, selected source lines 52-271.
