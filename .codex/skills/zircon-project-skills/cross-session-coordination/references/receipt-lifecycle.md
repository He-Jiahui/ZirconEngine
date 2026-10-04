# Asynchronous receipt lifecycle

1. Submit an authorized operation once with a stable request ID. Keep the returned request, job, ticket, run, or candidate identity.
2. A timeout after acceptance is uncertain transport, not proof of failure. Reconcile that exact operation before retrying. Do not launch competing Cargo jobs, bypass the lane, or create duplicate commits.
3. Continue dependency-independent work while the operation is queued, materializing, or running. An integration SHA proves the snapshot landed; it does not prove full validation passed.
4. When no independent work remains, check delivered scope and outstanding requirements once. Repeat that review only after changes or new findings.
5. Prefer a completion event or bounded status wait for the existing operation. Back off when state is unchanged. Release the active turn for a durable wakeup only when the worker and wakeup/resumption path are actually established; otherwise retain the pending identity and use supported bounded waits.
6. Treat a failed result as evidence to diagnose and route to its owner. Preserve independent work and live ownership. Retry only after reconciling the previous operation and correcting the cause or following its documented recovery.
7. Claim acceptance only after required validation and review are terminal and applicable failures are resolved. Report pending work honestly and follow the current tool's status semantics; do not mark a Goal complete to end a turn.

Neither a pending receipt nor an internal waiting state authorizes new scope, external notifications, a rollback, or indefinite repeated review/testing.
