# Zircon Feature Design

Design scripting, module/plugin, runtime, editor/runtime, and performance-sensitive behavior from current ZirconEngine contracts and relevant source evidence.

1. Identify the owning subsystem, current semantics, and the concrete design uncertainty.
2. Use [reference routing](../zr-reference-engine-routing/guide.md) to choose the closest source precedent. Add another reference only when it resolves a gap or tradeoff; there is no minimum engine count.
3. Capture exact source/test locations and the behavior they support. Separate upstream facts from the proposed local design.
4. Specify the relevant contracts: ownership, lifecycle, diagnostics, host capabilities, persistence, resource limits, or authoring flow.
5. Implement at the responsible layer. Avoid type-name dispatch, syntax-specific foundation patches, or compatibility wrappers that hide a missing contract.
6. Derive meaningful positive, negative, boundary, and regression cases for the behavior being changed. Include stress tests when repeated loading, hot reload, deep nesting, handles, or limits are affected.
7. Validate at the repository milestone boundary and report remaining gaps without claiming broader acceptance.

If no close precedent exists, state the uncertainty and justify the smallest coherent local design. Do not force a second source or a full evidence matrix for a bounded change whose behavior is already established.

Optional references:
- [Language source locations](references/reference-language-roots.md) for targeted searches.
- [Evidence and test checklist](references/feature-evidence-and-test-checklist.md) for complex semantic changes.
- [Architecture ownership](../zr-architecture-first-engineering/SKILL.md) when boundaries change.
- [Shared support diagnosis](../support-first-regression-testing/guide.md) when a high-level scenario fails.

The search helper `scripts/search_feature_evidence.py` locates relevant source; it is not a mandatory whole-tree scan.
