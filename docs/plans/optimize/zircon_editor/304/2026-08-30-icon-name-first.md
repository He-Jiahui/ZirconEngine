# Editor304 Icon Name First

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime-editor-357-358-359-303-304-20260830-v2`

## Scope

Icon-node identity now checks the already-materialized `icon_name` before scanning the role string.
Nodes carrying an icon name therefore take the common constant-time branch, while role-based icon
fallbacks and all non-icon behavior remain unchanged.

## Static Evidence

- Named icon nodes: role string match -> non-empty icon-name guard.
- Role fallback for `Icon`, `IconButton`, and `SvgIcon` is preserved.
- Behavior and source-contract tests cover named, role-only, and ordinary nodes.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR304_ICON_NAME_FIRST_BENCH_V1`. It compares role-
first and icon-name-first classification over 2,000,000 runtime nodes with a non-matching ordinary
role across 17 interleaved samples and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`. The
sample measures the actual role-branch checks; it does not claim a full scan of long role strings.

No direct Cargo validation was started. The coordinator owns combined Runtime/Editor validation,
exact timing capture, record finalization, manifest-only staging, commit, push to `origin/main`, and
the one-shot WeCom result containing exact performance data, test result, commit SHA, and branch.

## Editor304 benchmark correction (2026-08-30)

The ignored release benchmark now exercises a runtime `TemplatePaneNodeData` through the production
`is_icon_node` candidate and black-boxes the role/icon inputs. This removes compile-time constant
folding from the comparison; managed Cargo validation and the 30% p95 gate remain pending.

The benchmark fixture was corrected to use a normal non-matching role. The former 1,024-byte fixture
did not incur a full string scan because the literal role comparisons reject by length, so that
description overstated the measured cost model.

## Current batched validation handoff (2026-08-30)

Editor304 is included in the accepted combined Runtime/Editor batch with Runtime357/358/359 and
Editor303: request `141cfc54bef342968017a4441c534e10`, ticket
`b62653294ca7402d88462819c82cfaeb`, source manifest hash
`d946a6a4cf07a559b828841f51929334afae7e1c432d43947c417d1f8055210e`. The queued command runs all
15 focused and ignored tests in one release invocation with five 30% p95 gates. No terminal Cargo
or performance result is claimed until the coordinator completes it.
