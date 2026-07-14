# Shader 04 material ABI failure return closeout

Plan: docs/plans/zircon_runtime/shader/04-material-binding-and-renderer-contract.md
Milestone: M1
Status: completed
Files: ["docs/plans/zircon_runtime/shader/04/2026-07-14-material-abi-expectation-closeout.md", "zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs", "zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs"]

## Scope Delivered

- Replaced the stale 144-byte material-uniform expectation with the canonical `GPU_MATERIAL_UNIFORM_MIN_SIZE` owner.
- Replaced the obsolete binding-11 rejection fixture with semantic validation of the required clearcoat-normal texture and sampler bindings.
- Returned the Editor02 failure handoff without truncating the production ABI or restoring compatibility diagnostics.

## Fresh Testing Evidence

- Coordinator ephemeral test job `027d2cf8eaa440cd8bcd8698b7c38906` ran both exact current-source tests; each passed 1/1 with exit 0.
- The earlier managed fresh lib-test and segmented scene run accounted for 1698 passing, 6 ignored, and only 2 foreign Plugins08 failures; no Shader04 failure remained.
- Scoped `rustfmt --edition 2021 --check` and `git diff --check` pass for the four-file commit candidate.

## Review

- The tests now derive size from the unique GPU ABI owner and verify meaningful resource semantics.
- Production material layout behavior is unchanged by this failure return; Render18 M1 committed the corresponding advanced PBR ABI before this closeout.
- The historical SH04-M1 zmaterial v2 and three-layer override implementation is already in HEAD with its recorded build and focused tests; this return closes its remaining product-test expectation drift without claiming SH04-M2 or SH04-M3 complete.

## Status And Completed Items

| Milestone | Item | Status | Evidence |
|---|---|---|---|
| M1 | zmaterial v2 and layered override main path | completed | Existing plan evidence records the v2 hard cut, parent folding, runtime overrides, and focused tests. |
| M1 | Material ABI expectation failure return | completed | Two exact current-source product tests passed 2/2. |
| M1 | M1-T testing stage | completed | Managed coordinator validation plus the focused ABI tests are accepted; later SH04 milestones remain open. |
