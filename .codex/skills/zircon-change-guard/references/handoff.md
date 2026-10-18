# Handoff

The [source ownership policy](../../jenkins-coordination/references/incremental-patch-validation.md) owns the patch lifecycle. This page fixes the files an agent leaves behind and the checks it makes before and after submission.

## Outbox layout

Store handoff material under `.codex/outbox/<session-id>/<request-id>/`, after confirming its write grant in the current runtime:

| File | Content |
| --- | --- |
| `change.patch` | The exact unified diff the guard checked |
| `guard.json` | `change_guard.py --json` output for that diff |
| `patch-request.json` | Jenkins patch intake payload |
| `manifest.md` | The manifest below |

Do not store source copies, before/after file bodies or archives there.

## Manifest

```yaml
task: <one line>
authorization:
  paths: [<declared allowlist>]
  controlPlane: none            # or the user's words and date
  commit: none                  # or the user's words and date
checkout: {path: <absolute>, branch: <name>, base: <sha>}
foreignChanges: [<paths seen in git status and left untouched>]
guard:
  status: <actual guard.json status>
  patchDigest: sha256:<hex>     # from guard.json
  warnings: [<rule path:line reason>]
  suppressed: [<rule path:line reason>]
markers:
  closed: [{id: CR-..., fixedIn: <path>, test: <test path::name>}]
  opened: [{id: CR-..., path: <path>}]
tests: {added: [], changed: [], reachableFrom: <crate root>}
coverage: {packages: [], tests: []}
jenkins: {requestId: , attemptId: , patchRequestRef: , queue: , stages: {}, status: pending}
pending: [<every gate still open>]
```

`patchDigest` is SHA256 of the exact `change.patch` bytes, including CRLF. Regenerate `guard.json` after any edit or newline conversion; review its complete file list, declared paths and `writeGrantCheck`. A contributor-mode receipt is not an agent write-scope check.

## Patch intake

1. Check the coordinator with `jenkins-coordination` and confirm the selected driver in `.jenkins/state/deployment/driver.json` supports patch intake. Verify activation and the actual supported commands, not just the existence of a job or launcher. If the route is unavailable, preserve the scoped patch and metadata, report the gap and continue independent authorized work.
2. Build `patch-request.json`. The intake in `tools/jenkins/patches.py` requires:

   ```json
   {
     "identity": {
       "repositoryId": "<id>", "sessionId": "<id>", "requestId": "<new id>",
       "attemptId": "<id>", "generation": 1,
       "stageImplementationDigest": "<64-hex digest of the selected driver>"
     },
     "unifiedPatch": "<contents of change.patch>",
     "beforeHashes": {"<path>": "<sha256 of the base file bytes, or null for a new file>"},
     "declaredDependencies": ["<unchanged files the change relies on>"],
     "coverage": {"packages": ["<crate>"], "tests": ["<filter>"]},
     "externalInputs": [],
     "baseHead": "<base sha>"
   }
   ```

   `beforeHashes` must name exactly the old and new paths the patch touches, including both sides of a rename. Hash bytes from the bound base and preserve foreign-change attribution; do not infer ownership from a whole-checkout diff. The intake only requires `coverage` to be a non-empty object without `sourceDigest` or `coverageDigest`; the keys shown are an example, so use the ones the selected driver's compose step reads.
3. Run, with the same request file for the whole lifecycle:

   ```powershell
   $py = '.jenkins/runtime/python/python.exe'
   & $py -B -m tools.jenkins patch register --input-file <abs>\patch-request.json
   & $py -B -m tools.jenkins patch submit   --input-file <abs>\patch-request.json
   & $py -B -m tools.jenkins patch query    --input-file <abs>\patch-request.json
   ```

   Add the returned `patchRequestRef` to the request file before `submit`, `query` and `reconcile`. Use `reconcile` after any failed or unknown result; do not register a new request to replace it.
4. Jenkins checks the session's recorded authorization for `implementation`, `validation`, `commit`, `push` and `notify`. Those records come from the user. Use the implemented authorization-owner route and current receipts; `tools/jenkins-management/authorization_owner.py` is a production caller of `State.authorize_session`. An `action_not_authorized` response leaves the action unaccepted and requires checking the real grant or reporting a capability gap. Never call `authorize_session` directly, manufacture authorization records in `.jenkins/state` or supply `"source": "user"` evidence on the user's behalf.

## After submission

- Record each stage receipt Jenkins actually returns: patch, syntax, format, build, tests, acceptance. A queue ID, a `registered` status or a terminal job color is not acceptance.
- `commit` runs only when the flow was started with `ALLOW_COMMIT=true` and `COMMIT_AUTHORIZATION`, and the session holds the `commit` action. Use existing user authorization if it covers this change; otherwise obtain it after preparing a concrete result for review. Passing checks do not grant it.
- If the patch base moved (`patch_base_changed`) or a file changed underneath (`foreign_edit_detected`), reconcile the original request and inspect its receipts before preparing a replacement patch against the new base. Rerun the guard on the resulting checkout and follow the coordinator's identity protocol. Do not force or hand-merge another session's changes.

## Contributor CI integration

When CI integration is authorized, use `patch --contributor --patch <file>`; `/dev/null` is not a permission-disable switch. Check the full contributor diff against the final PR checkout. Keep merge-base/PR-head identity and the exact patch bytes in evidence. This skill does not install or activate a CI job.
