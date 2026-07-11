# Session Coordinator Milestone Workflows

The coordinator now owns the complete milestone boundary: immutable plan
topology, current attempt selection, validation/review/Failure/output gates,
scoped Git commit, Goal closeout, and one post-commit notification attempt.

## Topology versions

Numbered plan definitions remain read-only. A plan may provide exactly one
`zircon-workflow` fenced JSON object with schema 1. The importer validates the
workflow slug, unique milestone IDs, dependency existence, cycles, and the
numbered plan owner. Plans without the fence use deterministic `## Milestone
Mx:` headings and bold `Mx.y` checkbox slices.

Every different plan content or semantic topology becomes an immutable version.
The first version is activated for a run. A controlled refresh may activate a
content-only candidate without changing node identity. A structural candidate
may rebuild only a pristine graph; once any non-Goal attempt exists, activation
fails closed and requires a successor Goal/run instead of rewriting history.
The console shows the active version and version history.

Schema 16 was already released for the M3 closed action enum. M4 uses schema 17
for topology/evidence, schema 18 for immutable milestone manifests and same-run
integrity, and schema 19 to upgrade deployed validation bindings. Existing state
upgrades monotonically; applied migrations are never rewritten.

## Evidence gates

Milestone approval is denied until all owned slices are `succeeded` or
explicitly `skipped`, and the current fingerprint has accepted evidence for:

- the named validation stage;
- an independent review with zero Critical and Important findings;
- the applicable Failure audit;
- the numbered plan-output audit;
- the exact commit manifest.

The fingerprint binds the active topology version, current plan content,
current attempt IDs, HEAD, baseline epoch, exact worktree-blob manifest, and the
applicable Failure revision. A retried slice, plan edit, or any changed input
makes older evidence stale. Review records reject an executor reviewing their
own milestone.

Each milestone receives one immutable path/hash manifest per topology version;
the manifest is declared by the exact `Files` JSON field in its numbered child
output record. Gates, validation, and commit consume that same manifest rather
than all Session attribution. The
copied source hash is bound before launch and recomputed before result import,
so repository or copy mutation fails closed. Independent review binds both the
authenticated reviewer Session and target executor Session and rejects equality.
Plan output requires exact structured Plan, Milestone, Status, scope, fresh-test,
and review evidence; browser assertions never satisfy a gate.

## Controlled commit and Goal closeout

`milestone.commit` and `session.complete` are red actions requiring a
Session-bound `committer` role and explicit preview confirmation. Their browser
parameters contain only Session, workflow-run, and milestone identifiers; file
scope and commit text are service-derived.

The milestone service revalidates all gates inside the finalizer's single Git
mutex immediately before the compare-and-swap update of `main`. It builds a
temporary index view from HEAD plus only the attributed manifest, so another
Session's staged or untracked files cannot enter the commit. The previous index
is restored afterwards and foreign staged entries remain intact. Before the
CAS, failure restores the prior index and leaves HEAD unchanged. After the CAS,
the service only performs forward reconciliation and never rolls shared `main`
back.

Goal closeout requires an active non-empty topology whose content still matches
the numbered plan, every milestone to have a reconciled succeeded attempt, no
pending delayed patch, no dirty path covered by its write scope, attribution,
or lease, and no applicable open Failure. One database transaction records the
Goal attempt, releases leases, updates Session/run state, and appends the event.

Before changing `main`, the workflow writes a durable commit intent keyed to the
Git finalizer request. If the process fails after the compare-and-swap, startup
or the current request reads the durable `ref_updated_sha` and atomically
completes baseline/finalizer state before atomically appending the milestone
attempt plus commit artifact. Recovery is forward-only; shared `main` is never
rolled back.

## WeCom notification

After Git, baseline, and milestone evidence succeed, the service creates one
durable reservation keyed by commit SHA and channel, then calls the configured
personal `wecom-push-message` script exactly once. The four lines are built only
from server commit data:

```text
核心内容摘要：<中文里程碑摘要>
提交时间：<commit ISO time>
修改情况统计：<git shortstat>
提交的commit内容：<SHA> <subject>
```

The database stores only message hashes, timing, result codes, and sanitized
errors. It never stores the webhook URL or key. A failed or unknown call is not
retried automatically and cannot roll back the successful commit.
