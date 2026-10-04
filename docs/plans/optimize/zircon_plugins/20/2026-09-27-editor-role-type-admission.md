---
status: implemented_pending_validation
source_checked_at: 2026-09-27
session_id: astra-plugin20-role-type-admission-20260927-01a0df17
---

# Editor product catalog role type admission

| Scope | Finding | State | Evidence |
|---|---|---|---|
| Reject explicit non-string package roles before Editor catalog publication | [Plugins20 P0-01](../20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md); [PLUGIN-A2](../../../astra/features/plugins/01-selection-and-product-eligibility.md) M2 | `implemented_pending_validation` | Source fix, reachable focused regression sources, static checks below; managed Cargo and product evidence remain open |

The current build parser treated `package_role = false`, numbers, arrays, tables,
and datetimes as an absent field through `as_str().unwrap_or("production")`.
It now defaults only an absent field to Production and rejects every present
non-string value. Existing string role eligibility and legacy absent-field
compatibility remain intact. No package IDs or directory names decide admission.

`build.rs`'s two existing carrier tests were inside the custom-build target,
where ordinary Cargo testing does not run them. The existing Editor library
test tree now includes that exact build source as a test module. Four reachable
behavior tests cover current carriers, current Production, six explicit wrong
types, absent role, all four valid role strings, and unknown/empty/padded strings.
The module's private `main()` is never called; the only dependency beyond `std`
is `toml`, already a normal Editor dependency as well as a build dependency.

## Source and receipts

Pinned Session base: `bc02eefafead65dbf5050482110e8175250a5e77`, epoch `628`.
Register request `74608a0ccdf145b28493a09052946eb5` initially timed out after
acceptance; the Session was present and subsequent managed lease/status commands
succeeded. Exact lease request `5f053d2af33949bfb52174e3c963f32d`; child-output
authorization request `2e6888ef3624490bbf2d081c54f5a9da`.

| Current source | SHA-256 |
|---|---|
| [build parser and tests](../../../../../zircon_editor/build.rs) | `1bf25a6362705cf31302db9d1063180a79e5cbe3d508ffafb892d1902083a32e` |
| [reachable Editor test module](../../../../../zircon_editor/src/tests/editor_plugin_catalog_consistency.rs) | `4844f0d81e87757a19fa4f54fe8244217de363aea5c8b9732e7434c8cac27340` |

## Verification and open acceptance

- Before implementation, the new behavioral regression was authored while the
  wrong-type default branch was still present. A source discriminator confirmed
  that exact branch; no Rust red test execution is claimed.
- `rustfmt --check --edition 2021` for both Rust files: passed.
- `git diff --check` for both Rust files: passed.
- `python -m unittest tools.tests.test_astra_plugin_selection_and_trust_contract`:
  7/7 passed. These inspect source contracts and do not execute Rust behavior.
- Static harness checks verified the included path resolves to the owned build
  source, that four tests exist there, and that `toml` is a normal dependency.

Required managed filter:
`tests::editor_plugin_catalog_consistency::build_script_catalog_tests::product_eligibility_tests::`.
No managed Cargo request was submitted during the known proxy blocker, and no
Rust test or product pass is claimed. The original catalog/App checks, real
product/export carrier exclusion, Windows native trust/side-effect, and Release
performance gates remain open. The separate
`astra-plugin-target-scope-20260926` Session retains its source and acceptance.
