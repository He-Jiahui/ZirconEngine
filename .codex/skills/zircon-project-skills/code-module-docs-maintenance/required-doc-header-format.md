# Required Doc Header Format

- Every code-facing document under `docs/` must start with YAML frontmatter.
- Put `related_code` first so scripts can quickly map code files to the document.
- Use repository-relative paths.
- Include all files whose behavior is described, not just the file you edited most recently.
- Use this minimum header shape:

```markdown
---
related_code:
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
implementation_files:
  - zircon_runtime/src/engine_module/mod.rs
plan_sources:
  - user: 2026-03-27 <short request summary>
  - .codex/plans/example-plan.md
tests:
  - zircon_runtime/src/tests.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---
```

- `related_code`: every code file this document helps explain or constrain.
- `implementation_files`: the files directly implementing the described behavior. This may be a subset of `related_code`.
- `plan_sources`: the user request, design note, milestone plan, spec, or acceptance target that motivated the implementation.
- `tests`: the tests, fixtures, and acceptance documents that verify the documented behavior.
- `doc_type`: suggested values include `category-index`, `module-detail`, `workflow-detail`, `testing-guide`, or `milestone-detail`.
- Keep the header current whenever code or tests move.
