# Incremental patch validation

Use this workflow for ZirconEngine engineering changes sent to Jenkins. The user
requires incremental patches and coordinator-owned source preparation,
application, compilation and validation, with a reusable source baseline.

## Source ownership

- Agents must not create source snapshots, backup trees, source archives or
  validation checkout copies for this project. This applies to selected-file
  backups as well as whole-project copies, including `.codex/outbox`, temporary
  directories and locations outside the checkout. A writable destination does
  not grant an exception to this rule.
- Edit authorized canonical files directly. Preserve ownership and before/after
  hashes; hand off scoped patches and metadata. Do not store complete before/after
  source copies in evidence files as a substitute for snapshots, or call
  `snapshot.capture` and `snapshot.materialize` from the agent session.
- The independent Jenkins coordinator owns baseline provisioning, candidate
  materialization, source bundles and their retention or cleanup. Agents use its
  issued source identities and verified receipts. Preserve historical
  coordinator snapshots and receipts; removing session-generated backups does
  not authorize deleting coordinator data.

## Patch handoff

- Edit the shared checkout with `apply_patch` or another scoped patch operation.
  Preserve preexisting and foreign changes. Do not derive task ownership from a
  whole-checkout diff; select the authorized paths and changes explicitly.
- Bind the patch to an immutable base revision, before/after content hashes,
  dependency inputs, an explicit path allowlist and a request identity. Include
  additions, deletions, renames and binary payloads when the task requires them.
  Untracked files enter the handoff only when explicitly selected.
- Store the patch and metadata under the registered outbox or output namespace.
  Read access to an input does not authorize creating a staging copy beside it.
  Store no accompanying source backup. Source preparation belongs to the
  coordinator under the policy above.

## Jenkins responsibilities

- Provision and verify a registered source baseline once when needed. Reuse it
  for subsequent requests; update only affected content and declared dependency
  inputs. Bind each resulting candidate to its exact source identity.
- Serialize writers to the same source candidate and compiler-cache generation.
  Check the expected base and file hashes before applying the patch. Conflicts,
  unexpected local changes or uncertain ownership stop application and retain
  diagnostic evidence; do not force overwrites or guess a merge resolution.
- Apply or merge a conflict-free patch into the Jenkins validation candidate,
  then run the authorized build and affected tests. This candidate integration
  does not authorize committing, pushing or merging a production Git branch.
- Keep source staging, temporary files, artifacts and all compiler caches inside
  the sandbox's explicit grants. Compiler output and caches must physically use
  a registered namespace under drive-root `D:\cargo-targets`, `E:\cargo-targets`
  or `F:\cargo-targets`; reject aliases and repository-local targets.
- Retain the base, patch and resulting-source hashes, actual commands, queue/build
  identity, results, archived bytes and native process termination evidence.
  Enqueuing work is not a passing build or validation receipt.

## Asynchronous lifecycle

Submit once through an implemented, verified patch interface. Keep that exact
request and attempt through reconcile, cancel and verify. Continue independent
authorized work during execution, and use completion events or bounded status
checks under the [receipt lifecycle](../../zircon-project-skills/cross-session-coordination/references/receipt-lifecycle.md).
Do not repack a project, create a replacement identity or rerun an unchanged
validation merely because the original build is pending. Required acceptance
stays open until applicable terminal evidence is verified.

## Current capability boundary

The activated pilot entry `tools.jenkins.jenkins_coordinator` exposes bundle
submission; it is not an incremental patch handoff. Coordinator implementation
and activation may change independently of this guide. Inspect the selected
interface and its current activation and acceptance evidence before using it.
If coordinator enablement or verified patch support is missing, report that
condition and preserve any prepared patch; do not claim automatic merge/build
support, invent a CLI option or create a source copy or bundle locally.
Implementing or activating a missing backend follows the user's task scope and
the existing Jenkins acceptance gates.
