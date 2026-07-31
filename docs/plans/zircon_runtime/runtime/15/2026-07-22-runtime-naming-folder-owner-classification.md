# Runtime15 Runtime Naming Folder Owner Classification

status: static_validation_passed_waiting_scoped_commit
date: 2026-07-22
base_head: 6debc3e43aed7ed3ee9c7e25e38388bdd209981a

## Scope

This Runtime15 child slice repairs current-source classification drift in the
repository runtime naming audit. It changes the audit owner and its focused
Python regression only; runtime scene components and diagnostic-log production
behavior remain unchanged.

Exact commit scope:

- `.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py`
- `tools/tests/test_runtime_init_level_naming.py`
- `docs/plans/zircon_runtime/runtime/15/2026-07-22-runtime-naming-folder-owner-classification.md`

## Failure Evidence

The current-source audit reported 24 unclassified editor references:

- 23 `editor_hint` reflection metadata locations under the folder-backed
  `zircon_runtime/src/scene/components/scene/` owner;
- one inline diagnostic-log test scope named `editor` in
  `zircon_runtime/src/diagnostic_log/level.rs`.

Focused RED command:

```text
python -m unittest tools.tests.test_runtime_init_level_naming.RuntimeInitLevelNamingTests.test_runtime_editor_metadata_owners_are_explicitly_classified
```

Result: exit 1, 1 test executed, and the assertion proved
`editor.unclassified_location_count = 24`.

The cause was owner-path drift in the classifier: it still recognized the
deleted flat `scene/components/scene.rs` owner rather than the current
folder-backed children. The diagnostic classification also omitted the
current diagnostic level owner.

After that correction, the current legacy report exposed a second classifier
boundary drift: five graphics/render references and two scene references were
all inside directly `#[cfg(test)]`-guarded items, but the legacy classifier did
not receive the cfg-test line context already used by the editor classifier.
The focused RED called the legacy classifier with this context and failed with
`TypeError: unexpected keyword argument 'in_cfg_test_item'` (1 test, exit 1).

## Implementation

- Hard-cut the scene reflection rule to the current
  `scene/components/scene/` child tree.
- Keep the rule token-bounded: every matched editor token must be exactly
  `editor_hint`.
- Classify `diagnostic_log/level.rs` through the existing curated
  facade/diagnostic owner decision only when the reference is inside a direct
  `#[cfg(test)]` item.
- Apply that same item-level boundary to legacy references: direct cfg-test
  items are test fixtures, while the identical graphics or scene path without
  cfg-test context remains its production migration-debt owner.
- Update current-owner inventory assertions and add direct negative coverage
  proving that `editor_authoring_state` and a sibling
  `scene/components/scene_metadata.rs` path remain unclassified. The same
  diagnostic path with a production `editor_authoring_state` token is also
  required to remain unclassified.
- Add a minimal temporary-repository integration regression proving that a
  cfg-test `legacy_route` and a production `legacy_route` in the same graphics
  file classify independently; no exact-path whitelist or whole-file exemption
  is used.
- Parse cfg-test item boundaries from a Rust lexical code view that blanks
  comments plus normal/raw/character literals while preserving lines. The
  item scanner tracks multiline `()`/`[]`/generic-const signatures, separates
  const/static initializers from const functions and type aliases, and ends an
  item only at its real body or top-level semicolon.
- Do not reintroduce the deleted flat path, rename schema fields, add an
  exemption, or suppress the audit.

## Validation

| Gate | Result |
|---|---|
| `python -m py_compile` for the audit and focused test | passed |
| scoped `git diff --check` | passed; line-ending conversion warnings only |
| focused former RED | 1 passed / 0 failed, 29.610s |
| prior `python -m unittest tools.tests.test_runtime_init_level_naming` | 4 passed / 0 failed, 76.943s after the editor review correction; this predates the two new legacy-boundary tests |
| current runtime naming audit | editor unclassified 24 -> 0; editor gate `classified`; scene-reflection metadata 35; curated facade/diagnostic 1 |
| legacy cfg-test focused RED | 1 test / exit 1; classifier rejected `in_cfg_test_item` |
| legacy classifier + temporary-repository integration | final fast boundary batch 4 passed / 0 failed, 0.027s |
| reviewer parser regressions | RED proved brace-literal overrun, `[u8; 4]` multiline-signature underrun, `(1 < 2)` comparison overrun, and direct cfg-test const comparison overrun; each exact regression passed after its lowest parser fix |
| current seven-location directional scan | 7/7 references have `in_cfg_test_item=true` and classify as `test-fixture`; production negatives remain graphics/scene debt |
| current full runtime-naming regression | `python -m unittest tools.tests.test_runtime_init_level_naming`: 6 passed / 0 failed, 174.484s |

Current source hashes after implementation:

| Path | SHA256 |
|---|---|
| `runtime_naming_boundary.py` | `bd724473648459190bfe8d4ea759a004fc1752c2387b7950ae5f7f07cdcf14c5` |
| `test_runtime_init_level_naming.py` | `0b74fd1b243f5123c76272f5cd00380fc5b91c8281a0f04cd461aaa7d612347f` |

No Cargo result is claimed because this slice changes Python audit tooling and
its Python regression only.

The initial editor-classification review reported Critical 0 / Important 1 / Minor 0:
the diagnostic level path had been classified for the whole file. The rule is
now `in_cfg_test_item`-bounded and the requested production-token negative
test is present. The later legacy-boundary review found four parser defects in
sequence: literal braces, delimiter-contained semicolons, comparison operators
inside signatures, and const/static initializer comparisons. Each was first
reproduced as a production-negative RED and repaired at the shared parser.
Final code/test re-review returned Critical 0 / Important 0 / Minor 0 and also
covered static mut, const unsafe fn, type aliases, generic const blocks, and
multiline where clauses. Record-only final review returned Critical 0 /
Important 0 / Minor 0 before the replacement exact3 snapshot.

## Remaining Boundaries

- The runtime naming audit's six-test current-source regression is green. The
  seven known legacy locations classify as test fixtures, while the production
  negatives remain graphics/scene migration debt; this closes the aggregate
  naming-classifier evidence gap without claiming those independent production
  migrations complete.
- The aggregate module-convention gate retains independent hard-cutover debt;
  historical non-network-server M2 classification is outside this exact3 and
  is neither reopened nor claimed by this slice.
- The audit owner is now 818 lines after the lexical parser hardening. An
  attempted exact3-to-exact4 session expansion for a parser child owner was
  rejected with `session_write_scope_immutable`; no unowned child file was
  created. A future split must use an audited scope transfer/session rollover,
  not an out-of-scope file write.
- Managed `milestone prepare --milestone M2` was attempted while snapshot 861
  exact3 was current and was safely rejected before any staging because the M2
  workflow node declares the historical non-network-server exact3. This record
  was subsequently corrected to distinguish that historical prepare evidence
  from the current exact3; prepare was not retried against the unchanged
  topology. The Runtime15 parent plan also has an unrelated Performance-owned
  dirty status row. The original workflow/parent owner must publish a current
  native slice before this exact3 can use the managed commit path; this record
  does not absorb or rewrite that foreign parent change.
- Runtime15 parent completion and the Runtime15 source-cubemap Cargo gate are
  not claimed by this child record.
