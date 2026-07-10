# Refactor Rules

- Remove obsolete behavior instead of preserving it behind flags, fallbacks, aliases, duplicated APIs, or migration-only runtime branches.
- Delete superseded helpers, structs, tests, manifest wiring, stale comments, and dead branches in the same change when they no longer serve the current design.
- Treat remaining compatibility-only code for superseded behavior as a blocker during review and acceptance.
- Allow destructive cleanup when it simplifies the current design; do not spend time preserving outdated behavior by default.
- Keep modules explicit. One functionality-level module belongs in one file; when a concern splits into multiple behaviors, move it into a folder-backed subtree instead of piling more code into the same file.
- Keep `.rs` source files concise and independent. If a module starts to carry multiple behavior families, split before line count turns the problem into an emergency.
- Keep declaration files minimal. Define the type and only the smallest obvious helpers there; move parsing, routing, formatting, serialization, conversion, and heavier `impl` blocks into sibling behavior files.
- Keep `lib.rs`, `main.rs`, and `mod.rs` files primarily navigational. Re-exports, wiring, and small entry helpers can stay there; substantial behavior should move to child modules.
- Group similar modules into folders. Avoid wide flat file lists once a subsystem has multiple related parts, and avoid catch-all files such as `util.rs`, `helpers.rs`, or `common.rs` unless the scope is genuinely cohesive.
- Before introducing a new module or folder layout, inspect the closest matching source tree in `dev/UnrealEngine`, `dev/godot`, `dev/bevy`, `dev/Fyrox`, or `dev/Graphics` and align naming and ownership with the nearest mature precedent.
- Favor shapes that survive future Unreal-scale complexity. Do not accept a flat "good enough for now" layout for a subsystem that is likely to keep expanding.
- Do not wait for 1000 lines if responsibilities have already diverged. Treat roughly 1000 lines as an emergency ceiling, not as permission to keep stacking logic.
- Update tests and documentation to match the current behavior only. Remove expectations that exist solely to preserve an old design.
