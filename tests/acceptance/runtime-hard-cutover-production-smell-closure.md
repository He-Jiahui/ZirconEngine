---
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - zircon_runtime/src/asset/assets/texture/external_source_cubemap/decode.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - tools/tests/test_hard_cutover_migration_smells.py
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
output_records:
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md
  - docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md
---

# Runtime Hard-Cutover Production-Smell Closure

Date: 2026-07-10

Accepted behavior:

- production scans exclude `#[cfg(test)]` items but still scan all live production items;
- DDS uses explicit pre-DX10 protocol vocabulary;
- `.zshader` errors use explicit schema-version and removed-field migration vocabulary;
- editor point-as-pixel persisted-value migration is an exact-file, one-way data upgrade policy, not a compatibility API;
- Hyper's third-party HTTP/1 module path remains an exact Net backend policy, not a global exception.

Verification: Python TDD 2/2; scoped rustfmt passes; direct hard-cutover report has legacy references 3 in two allowed policies, compat 0, shim 0, unclassified 0, migration debt 0, gate `classified-and-clear`, and `risks = []`.

The final full `audit_runtime_structure.py --json` run reports an empty risk summary across every audit section.

The current default-profile production library also passes `cargo check -p zircon_runtime --lib --locked --jobs 1` in 10m45s with 418 existing warnings and no error. This is a library check, not a full package/workspace test claim.
