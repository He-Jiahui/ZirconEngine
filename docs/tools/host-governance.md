# C/D/E ownership governance through Jenkins

`zircon-governance` is a bounded job on the existing controller and Windows agent.
It uses the existing `.jenkins/state/coordination.sqlite3`, selected driver and
runtime Python. It does not restore the retired coordinator. The operator client
is `tools/dev/jenkins_governance.py`; the existing inventory command accepts
`--host-jenkins --operation-id <original-id>` for whole-volume discovery.

The installed job pins implementation hashes and the runtime Python executable.
Jenkins validates those pins before loading State, then binds every operation to
the repository, session, original request, attempt, driver and Jenkins build.
Duplicate submission returns the original operation. `reconcile` observes that
operation; it does not execute a replacement attempt. A changed implementation
requires reinstalling the job and passing its exact functional capability tests
before version, source, permission or cleanup writes.

Reports and the artifact ledger live in
`E:/cargo-targets/zircon-local/reports/disk-governance-20261015/`. Jenkins also
archives inventory and operation receipts. Queue/build success alone is not
claim, native process, integration or permission acceptance.

Whole-volume discovery observes C/D/E concurrently within one retained native
Job. Each volume persists its complete metadata census before Git inspection,
then saves each repository observation separately. An interrupted scan keeps
those checkpoints and remains incomplete. Authentication directories, credential
filenames and nested `.jenkins` controller contents are excluded before Git
refresh and source hashing. Source worktrees rooted inside `.jenkins` are still
observed relative to their own checkout roots. Git network protocols are disabled
for inventory commands.

Whole-volume triage records Git SHA, semantic index digest, status digest and
changed paths. Source bytes are hashed again at bounded claim, takeover,
integration and cleanup admission. This keeps a large unclaimed reference tree
from blocking publication of the central claim list. `scan` accepts
`resumeOperationId` after the original native waiter proves terminal completion.
Jenkins binds the original verified implementation, request, attempt and the
three metadata checkpoint references emitted by that original scan. It rejects
changed checkpoints and re-observes every Git candidate with four bounded readers
in one retained native Job. It does not traverse the volumes again or treat old
Git observations as current inputs. Original attempts and failure receipts remain.
Large link exclusion lists are split into positive directory scopes below the
Windows command-line limit. Index paths and the staged deletion list participate
so deletions remain visible without recursively enumerating HEAD. Status uses deletion/addition entries for renames and
normalizes ordering; the shared index stays untouched.

## Interfaces and workflow

| Action | Contract |
| --- | --- |
| `scan` | Enumerate C/D/E, including hidden paths, without following links or reading authentication contents. Record failures and link targets. |
| `register` | Bind a physical canonical checkout, Git common directory, primary branch, build namespace and validation recipes. Ambiguous primaries need review. |
| `register-inventory`, `configure-repository` | Register unique project checkouts from the bound inventory, retain ambiguous related clones for review, and CAS validation recipes while keeping pending integration recipes immutable. |
| `open-round` | Bind the claim round to the actual inventory bytes and digest. |
| `claim` | Compare source hashes and serialize path ownership with round closure in State. Reject overlaps and unresolved existing source claims. |
| `inspect-scope` | Hash explicit file/directory scopes, including additions and missing tracked files, before claim admission. Reject links, nested authorities and authentication content. |
| `record-response`, `close-round` | Record the explicit central chat reply, then freeze its digest and claim set. No elapsed-time closure. |
| `adopt` | Reconcile historical source hashes, retained refs and native evidence for the original request/attempt. Historical rows and old drivers remain preserved. |
| `seal` | Retain the base, selected index state, binary-safe working files and untracked files in Git objects and a managed branch. Preserve the shared index. |
| `apply-patch` | Read a hash-bound JSON handoff from the claimed session's inbox and CAS its managed branch. Only claimed paths are accepted. |
| `prepare-integration` | Merge Git objects, retain conflicts, reject changes to claimed paths and retain a pending result branch. |
| `validate-integration` | Jenkins prepares the exact result tree and runs registered recipes with a retained native Job and physical build-root checks. |
| `finish-integration` | Require exact result validation, current hashes, writer termination, shared-index isolation and local-primary CAS. |
| `recover-integration` | Observe the original publication identity and reconcile a retained replacement index only when the exact result, source writes, index and original validation still match. Preserve failed receipts and raw preimages. |
| `cleanup-plan`, `cleanup-apply`, `cleanup-reconcile` | Require regenerability or recoverable source proofs and original writer termination. Bind every file to physical identity, size and SHA-256. Delete verified handles while locking ancestors. Absence alone cannot prove deletion. |
| `permission-plan`, `permission-apply`, `permission-probe` | Prepare a project session profile, retain the prior config as a Git object, apply an explicit hash-bound change, and run real parent/child permission probes. |
| `test`, `audit`, `reconcile` | Functional contract acceptance, current records and original-operation observation. |

Primary selection is existing registration, `origin/HEAD`, then a unique local
`main` or `master`. A directory name does not identify a project or prove a copy.
Worktrees share a physical common directory. Independent clones remain separate
inventory entries; shared historical roots are evidence for review, not automatic
permission to discard a clone.

The central inventory must be published in the chat before recording its claim
response. Claimed work stays under `codex/jenkins/<session>/<round>-<operation>`.
Unclaimed work is preserved by the round's maintenance owner before integration.
Staged, unstaged and untracked versions are retained separately. Conflicts,
changing input and failed recipes keep source and branches recoverable.
Sealing requires the observed `expectedIndexDigest` as well as HEAD, status and
working-file hashes: identical `MM` status and working bytes do not prove the
staged version is unchanged. `inspect-scope` returns that index digest.

## Patch handoff

The session writes JSON under its own `.codex/outbox/jenkins-inbox/<session>/` grant:

```json
{
  "claimId": "registered-claim-id",
  "baseHead": "exact-managed-branch-head",
  "files": [
    {
      "path": "src/example.rs",
      "beforeSha256": "exact-old-blob-sha256-or-null",
      "afterSha256": "exact-new-bytes-sha256",
      "contentBase64": "binary-safe-content",
      "mode": "100644"
    }
  ]
}
```

`contentBase64: null` denotes deletion. The operator submits its fixed path and
hash; Jenkins validates claim, branch, preimages, output hashes and path scope.
Agents do not prepare source snapshots, mutate Git, execute builds or delete
artifacts locally.

## Preservation and acceptance limits

Unknown owners, inaccessible paths, unresolved requests, hardlinks, toolchains,
installed tools, shared dependency caches, current pools, necessary evidence and
canonical source remain protected. Cleanup requires an authoritative
`generated_output` receipt or complete source seal; naming a directory `build`
or marking a request `regenerable` cannot supply that proof. Historical native
receipts without bound write scopes remain needs-review.

Deletion receipts distinguish logical bytes, deleted allocation bytes and the
observed volume free-space change. Concurrent volume writes prevent attributing
that change as actual reclaimed bytes. Unknown actual reclamation remains null;
it must not be reported as a measured saving.

Project session profiles grant writes only to the session inbox and its approved
temporary directory. The shared project default is read-only and grants no
session inbox; each session must explicitly select its own authorized profile.
Source, Git, State and build pools are read-only. CLI probe
success does not establish that an existing desktop chat selected the profile.
Protection requires activation and actual probes for that session. Full Access
overrides and independent MCP/browser/app writers remain outside project-only
enforcement. This task does not install machine-managed requirements or change
system accounts/ACLs. See the [official permissions documentation](https://learn.chatgpt.com/docs/permissions).

This implementation's functional capability receipts are distinct from selected
driver runtime acceptance, whole-workspace acceptance and milestone acceptance.
Pending claims, source transitions, validation and cleanup stay open until their
original receipts are reconciled.

## Observed rollout status, 2026-10-15

The rollout is incomplete. The three volume metadata traversals completed, but
the Git observation phase was interrupted. The immutable original checkpoints
remain under `inventory-cde-20261015-4/checkpoints/` in the report root above.

| Volume | Directories | Files | Logical bytes | Traversal failures |
| --- | ---: | ---: | ---: | ---: |
| C | 202,314 | 410,876 | 110,989,468,515 | 123 |
| D | 107,056 | 924,237 | 287,898,188,312 | 45 |
| E | 428,210 | 2,629,996 | 375,126,542,922 | 737 |

The traversal recorded 13,274 links without following them and 763 Git
candidates. There are 721 persisted repository observations: 268 without
repository errors and 453 with errors. Another 42 candidates have not been
observed. Repository errors include failed Git observations, timeouts and two
Windows command-line length failures. An error is never a clean checkout.
Metadata includes 255,288 source candidate directories and 75,007 output
candidate directories; neither count establishes uniqueness, ownership,
regenerability or permission to delete.

Observed unmanaged changes include 58,854 paths under `E:/Git/hybridclr`,
2,434 under `E:/Git/zr_vm`, 733 under
`E:/Git/GhenshinImpactMovementStateMachine`. Three historical Zircon worktrees
were observed, two with changes. These are live observation counts, not current ownership
proof. The canonical Zircon checkout's status query failed; its change count is
unknown. The 42 missing candidates include remaining `zr_vm` references,
`E:/Projects`, university projects and two `E:/zircon-profiles` checkouts.
The final centralized claim inventory has not been published or closed, and no
user claim reply has been transcribed.

`governance-contracts-9` passed all 35 tests against capsule
`fef88bb71bd60652543817e9ab5d591288a3ae158f65c07d1705247db6ba0c0b`.
Its native receipt proves exit zero, joined waiter, pipe EOF and zero remaining
Job processes. All 126 pinned inputs were unchanged. Jenkins reclaimed its
1,261 owned fixture files and 868 directories: 3,262,164 logical bytes and
4,036,656 allocation bytes; actual attributable free-space recovery is unknown.
The subsequent resumption and status-partition repairs have 40 prepared test
cases and passing static checks, but no current functional acceptance. The
queued superseded 39-case build was aborted before State admission.

The original scan build is now `ABORTED`, while its governance operation still
has no native terminal receipt. The selected Jenkins driver changed during a
separate deployment recovery. The observed controller endpoint refused
connections and recovery reported `owned_recovery_upgrade_unproven`; its stopped
controller and agent receipts do not substitute for this scan's native receipt.
Do not manufacture scan completion or restart a retired coordinator to bypass it.

`permission-apply-zircon-2` retained the exact raw configuration preimage at
`refs/jenkins/governance/permission/permission-apply-zircon-2` and created the
registered session inbox and temporary directory. Windows denied the config
write. The configuration SHA-256 remains
`b31490a9954ca9cc2c89d006ef98e8b019b2ea1c64114f03500db9c09af031d9`;
the prepared profile is not applied, real permission probes have not passed,
and the current desktop chat remains unrestricted.

No original project changes have been integrated into local primary branches,
and no preexisting source, worktree or build output has been deleted. Failed
test evidence, interrupted scans and unresolved requests remain retained.
Earlier scan attempt 3 was aborted after its controller/authentication content
exclusion proved insufficient; that attempt supplies no accepted inventory.
Resume with the original deployment and scan receipts, accept the exact repaired
implementation, complete repository review, publish the central claim inventory
and collect the user's one explicit reply before closing claims. Project recipe
registration, legacy writer reconciliation, recoverable import of source copies
and complete worktree/clone retirement still require implementation or acceptance
before the requested one-checkout-per-project cleanup can be declared complete.

## Recovery follow-up, 2026-10-15

The explicit request to repair and restore authorized recovery of the existing
independent Jenkins deployment. `tools/dev/jenkins_recovery_bootstrap.py` records
bounded offline bootstrap evidence without changing the selected driver, source
configuration, queues, historical holds or original upgrade request.

The complete current support discovery finished with 1,514 passing tests and two
skipped tests (1,516 discovered), zero failures and zero errors. All four native
Jobs reached exit zero, pipe EOF and zero remaining processes; source hashes
remained stable. The immutable support receipt is
`.jenkins/state/migration-baseline/governance-recovery-offline-20261015-1.json`,
SHA-256 `2a7cf06ad4ff6a1065127645fa5fa51a7ab4731a9ec193ebeb102c4e50673491`.
It explicitly records `jenkinsExecution: false` and `jenkinsRerunRequired: true`;
offline bootstrap evidence does not replace Jenkins execution or acceptance.

The prior recovery failure was `Another repository runtime still owns the
deployment`; later attempts also detected the changed governance inventory and
test sources. A foreign Cargo validation supervisor subsequently exited. A
separate `full-support-offline-bootstrap-20261015-r1` recovery validation was
then observed running in another chat. That command subsequently finished with
its original native Job terminal, pipe EOF and zero active Job processes; its
regression failed and supplies no passing acceptance. A residual Python PID
41432 still appears in the Windows process inventory. Windows denies both
limited-query and synchronization access (error 5), so its exact native
identity or absence remains unproven. Its system image query identifies the
dedicated repository Python. Strict preparation therefore still stops with
`deployment_inventory_inconclusive`; Jenkins has not been restored. The current
sealed candidate is `a203a929a63c49af6f40daee61172fd86461e9e2e922d083c2993c48aead95b2`.
No selector or original upgrade request was changed. The evidence is retained at
`jenkins-recovery-20261015-1/restoration-status-20261015-1.json` in the report root.
Do not synthesize scan termination, release historical holds or restart the
retired coordinator. The bootstrap uses the existing readable
`test-deps-coordinator` dependency installation; it does not alter dependency
ACLs to gain access to `test-deps-desktop`.

A separate read-only diagnostic followed up all 42 missing repository candidates:
40 returned without repository errors, including 32 with local changes. The
JDK and Rust reference checkouts timed out and remain `needs-review`. Their Git
launchers exited at timeout but left the actual Git children holding output
pipes; only those two exact diagnostic children were terminated, after checking
their creation times and commands. The diagnostic parent exited zero. These
observations are retained in
`jenkins-recovery-20261015-1/read-only-repository-followup-20261015-1.json`; they do
not replace a Jenkins inventory or the original missing scan native receipt.
Git timeout containment still needs repair before another whole-volume scan.
All original source and build outputs remain retained. No additional cleanup
space was reclaimed during this recovery; the current offline fixtures remain
registered pending restoration and actual Jenkins rerun reconciliation.

## Rollout gates

The interfaces are available to the authorized operator through the installed
job. They do not close the central claim round or activate a desktop session.
The whole-volume scan and `audit` produce `inventory-review.json` and
`pending-claims.md`; the operator publishes that inventory in the chat before
transcribing the user's single claim response. Scheduled maintenance invokes
the same audit and retains candidates for review.

Admission and closeout receipts verify each registered canonical checkout,
Git common directory and physical compiler namespace. They do not establish
confinement of every process on the machine. Project validation recipes and
original writer evidence must exist before actual integration or cleanup.
An empty recipe list is a pending gate, not a passing validation.

Permission application can fail at an existing Windows boundary. Such a failure
retains its intent, exact preimage and receipt; it does not authorize changing
ACLs, restarting the controller under another identity, or using a service to
bypass the denial. A prepared configuration and a passing CLI probe remain
separate from activation of a desktop chat.
