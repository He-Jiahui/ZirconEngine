# Module Boundary Discipline

Keep one coherent responsibility per module and make subsystem ownership visible in paths. Refactor when the current change introduces mixed responsibilities or depends on a broken boundary.

- Group domain files under an owning subtree, such as asset, inspector, dock, or viewport. Avoid umbrella helpers that mix unrelated domains.
- Keep `lib.rs`, `main.rs`, `mod.rs`, and binding roots focused on declarations, curated exports, and minimal entry wiring. Move parsing, routing, mutation, and other substantial domain behavior to child modules.
- Separate independently meaningful declarations and growing behavior families. Tightly coupled private types and trivial methods may stay together when splitting would obscure their single responsibility.
- Extract encode, decode, parse, route, snapshot, or mutation files when those are substantial distinct responsibilities, not merely to satisfy one-declaration-per-file counting.
- Keep callers on intended public contracts. A curated export is different from a migration-only forwarding path.
- Use line count as a warning under [large-file guidance](../modularize-large-files/guide.md), not as an automatic approval or refactoring gate.
- A small fix in an existing large file does not require reorganizing unrelated code. Explain a material deferred boundary when it affects the task.

For root-specific allowed surfaces, use [root entry rules](../../zircon-dev/workflow/structure/root-entry-files.md). For a concrete mixed UI example, see [binding decomposition](references/binding-rs-anti-pattern.md). Consult [reference routing](../zr-reference-engine-routing/guide.md) when choosing a genuinely new subsystem shape.

During an authorized reorganization, identify the domains, move coherent owners and callers together, narrow the original root, and run the affected structural and behavioral checks. Do not broaden the reorganization merely to prepare for hypothetical future siblings.
