# Handoff

The [Agent management standard](../../../../docs/tools/agent-artifact-management.md) owns source, delivery and artifact lifecycle. Use a small local evidence handoff when an actual dependency or ownership transfer calls for one.

## Evidence layout

When a handoff is needed, use an authorized `.codex/outbox/<task-id>/` for scoped patches/metadata or an approved physical `cargo-targets/zircon-local/reports/<task-id>/` output namespace for command evidence. Record actual ownership and permissions before writing.

| File | Content |
| --- | --- |
| `change.patch` | Exact task-scoped unified diff, when needed for handoff |
| `guard.json` | `change_guard.py --json` output for the diff checked |
| `manifest.md` | Compact source, authorization, result and pending-gate manifest |

Do not store source copies, complete before/after file bodies or source archives there.

## Manifest

```yaml
task: <one line>
authorization:
  paths: [<declared allowlist>]
  controlPlane: none            # or the actual user's words and date
  commit: none                  # or the actual user's words and date
  remote: none                  # push, PR, publication and notification grants separately
checkout: {path: <absolute>, branch: <name>, base: <sha>}
foreignChanges: [<preexisting paths/increments preserved>]
guard:
  status: <actual guard.json status>
  patchDigest: sha256:<hex>
  warnings: [<rule path:line reason>]
  suppressed: [<rule path:line reason>]
markers:
  closed: [{id: CR-..., fixedIn: <path>, test: <test path::name>}]
  opened: [{id: CR-..., path: <path>}]
tests: {added: [], changed: [], reachableFrom: <crate root>}
coverage: {packages: [], features: [], targets: [], filters: [], profile: <profile>}
commands: [{command: <exact command>, exitCode: <actual>, evidence: <path>}]
delivery: {commit: <actual SHA or not-committed>, integration: <actual status>}
pending: [<every gate still open>]
```

`patchDigest` is SHA-256 of the exact patch bytes, including CRLF. Regenerate the guard result after any edit or newline conversion; review the complete file list, declared paths and permission result. A contributor-mode receipt is not an agent write-scope check. Missing evidence remains missing, not an invented hash or success.

## Before delivery

1. Verify the actual checkout/base and exact owned increments; additions, deletions, renames and binary inputs stay explicit. Preserve foreign work, index ownership and locks. Do not infer task ownership from the whole-tree diff.
2. Run only the applicable structural checks and declared local validation batch. Record actual source/command/physical outputs, exit codes and native terminal/EOF evidence where applicable. Dry run, launched, queued or client timeout is not passing validation.
3. Commit only when the existing action authorization covers it. Resolve the real Git directory and check its lock, inspect the exact staged task scope, then use native Git and report its actual SHA. Remote actions retain their separate authorization.
4. If the base or a touched file changes, inspect the new source and original operation before preparing an updated patch. Re-run affected checks and do not force another author's changes. Reconcile an unknown asynchronous delivery under its original identity rather than submitting duplicates.

## Historical and contributor boundaries

Historical Jenkins requests retain their original IDs, receipts and failed/unknown outcomes. They are not replayed or relabelled as local successes; [the archived workflow](../../jenkins-coordination/guide.md) is reference only. Independent local evidence does not close unrelated migration or product acceptance.

When CI integration is authorized, use `patch --contributor --patch <file>`; `/dev/null` is not a permission-disable switch. Check the full contributor diff against the final PR checkout, keeping merge-base/PR-head identity and exact patch bytes. This skill does not install or activate a CI job.
