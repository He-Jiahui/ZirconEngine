---
status: in_progress
---

# Runtime Dependency Validation Worklist

## Dependency validation obligations

Keep backend selection and unavailable-backend behavior explicit. SharedTextService's active backend must agree with text_shaper_stack_uses_shared_text_service_for_font_backends and shared_text_shaper_matches_public_layout_entrypoint. Jolt admission preserves empty_jolt_feature_slot_reports_unavailable_not_ready and unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick.

tech_stack_source_inventory.py owns source/test membership, tech_stack_anchor_inventory.py owns anchors, and tech_stack_boundary owns missing/risk aggregation. tech_stack_dependency_guard.rs independently checks archival policy using repository identity derived from CARGO_MANIFEST_DIR. No historical result closes these current gates.

此处仅展示当前现状的概述；01/2026-07-09-tech-stack-and-dependency-governance-output-records.md is dated provenance only.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `01-tech-stack-and-dependency-governance.md`, SHA-256 `45de182d4a364a8c36138bf4f9a38e84e3f1e8db2ac6bbce4749d41fb7fd6ad6`, selected source lines 123-229.
