# Runtime VerticalRl Paragraph Layout Implementation Plan

**Goal:** Complete Text03 VerticalRl paragraph constraints through the existing neutral paragraph owner.

## LB-M5 S0: Contracts

- [x] Add VerticalRl first-column indent, continuation, nested indent, Center, and Right alignment tests.
- [x] Preserve existing horizontal rich-block tests unchanged.
- [x] Record the pre-fix VerticalRl failures or shared-source compile blocker.

## LB-M5 S1: Logical paragraph extent

- [x] Expose vertical column constraints from `paragraph_layout.rs` without duplicating merge/clamp/list policy.
- [x] Make align-only paragraph overrides participate in block paragraph ownership.
- [x] In `vertical.rs`, wrap with first/continuation heights and resolve each column with its paragraph align/inset.
- [x] Map inset to physical y/height and alignment to top/center/bottom while preserving right-to-left x placement.
- [x] Keep parser and renderer unchanged.

## LB-M5-T: Validation and records

- [x] Run exact formatting/scoped diff checks and structure budgets.
- [x] Run focused VerticalRl paragraph tests and existing horizontal rich-block regressions on Windows.
- [x] Add a real rendered VerticalRl paragraph proof only if the existing product framebuffer cannot visibly demonstrate the new indent/alignment behavior.
- [ ] Update Text03 output records, concise status, module docs, and active session note.

## Status

| Slice | Status | Evidence |
|---|---|---|
| LB-M5 S0 | completed | Three VerticalRl contracts cover first-column indent/continuation, nested indent, and Center/Right inline-axis alignment; current-source Windows binary passes 3/3. |
| LB-M5 S1 | completed | The existing paragraph merge/clamp/prefix owner now exposes column constraints; vertical wrapping and placement consume those scalar logical constraints without parser/renderer changes. The complete rich-block filter passes 10/10, including all seven horizontal regressions. |
| LB-M5-T | in_progress-external-wgpu-blocker | Exact formatting/scoped diff checks pass; touched production/test owners are 278/284/195 lines and the product root remains 794 lines. A real SDF/WGPU proof command with CJK center-indent and end-aligned paragraphs plus pre-render layout gates is authored. The current exporter binary reaches WGPU device construction but fails before UI rendering because the concurrent renderer layout registers both lightmap sampler and volumetric params at binding 25 in `zircon-forward-shadow-receiver-layout`; no PNG is accepted from this run. |
