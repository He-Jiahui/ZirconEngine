# Failure Graph Source-Slice Gates

The coordinator keeps every imported failure open until its owner publishes a
verified `fixed-*.md` return.  A child-record-only handoff may nevertheless
need to commit its repaired source slice before that return exists.

For that narrow case, milestone gate selection excludes the open fixer failure
only when all of these are true:

- `plan_link_mode: child_record_only` is present in the imported handoff;
- its immutable `related_code` list is non-empty; and
- every listed path is in the candidate milestone manifest; and
- the only additional path is one date-named non-handoff record in the fixing
  child-plan directory.

The exception never changes the failure's lifecycle state.  The failure still
blocks unrelated manifests and must be returned through the normal verified
`fixed-*.md` flow after the source commit and upstream evidence exist.

## Cross-plan child-record return destinations

The generated `fixed-*.md` belongs in the originating child-plan directory.
When that directory is correctly protected by an origin Session's active,
resolving-failure, or waiting-validation directory lease, the fixing Session
must not release or take that lease merely to return the artifact. The
coordinator instead permits one narrow lifecycle
transfer only when all of the following are true:

- the fixing Session has live leases for the open failure and its generated
  receipt in the fixing child directory;
- the destination is the exact date-and-summary `fixed-*.md` computed from the
  imported lifecycle key;
- a live lease overlapping that destination belongs to an `active` or
  `resolving_failure` Session whose registered `plan_path` exactly matches the
  failure's `origin_plan`; and
- the coordinator records the origin-owner Session in the return audit event.

The origin lease remains owned by the origin Session throughout. A stale owner,
an unrelated plan, a generic directory lease, or any additional target still
returns `failure_return_lease_missing`. This is a lifecycle transfer for one
canonical artifact, never a general cross-session write exception.

## Milestone manifest lease coverage

A manifest contains exact repository files, while a Session may own a coherent
directory lease. Milestone binding must validate each manifest file through the
same live hierarchy-aware ownership rule used by the lease service: an owned
directory covers its descendants, an exact file covers only itself, and expiry
is always enforced. Binding must not require duplicate child leases or accept a
child lease as authority for a parent directory or sibling file.
