---
name: jenkins-coordination
description: Run functional Jenkins coordinator tests or submit bounded commands through the independent Jenkins pilot and Windows tray.
---

# Jenkins coordination

Use the independent Jenkins pilot and current tray as the only operational path. The retired local coordinator, service, tray, scheduled tasks, leases, attribution APIs, and queue must remain untouched.

## Incremental validation

For engineering validation, use the user's [source ownership and incremental patch workflow](references/incremental-patch-validation.md): agents edit the authorized checkout and hand off scoped patches and hashes; the independent Jenkins coordinator prepares and reuses the source baseline, applies patches and runs the relevant build and tests. Agents must not create source snapshots, backups or source bundles. Keep required acceptance pending while continuing independent work and reconciling the original request.

A bundle-only pilot interface does not satisfy this handoff. Verify that the selected coordinator accepts incremental patches and prepares its own sources before submission. If the route is unavailable or unverified, report that condition and preserve the prepared patch; do not create a local snapshot or bundle to work around it.

## Enablement

Use `python -B -m tools.jenkins.jenkins_coordinator --repo-root <absolute-repo-root> status` to check `.codex/state/jenkins-coordinator/active.json` before activated pilot work. Require:

- `enabled: true` and `functionalTestsAllowed: true`;
- the configured repository and pilot root match the current physical identities;
- the current storage owner and keeper are live with exact PID creation times;
- immutable driver, source snapshot, and tray acceptance evidence match the active configuration.

If a check fails, perform read-only inspection and report the blocking evidence. Do not start, adopt, or repair an unknown process from a PID alone.

## Functional flow

For existing activated pilot requests, use the `tools.jenkins.jenkins_coordinator` entry and fixed templates only:

- `python-static` for Python checks;
- `managed-cargo-check-v2` for Cargo checks;
- `fault-probe` for controlled failure tests.

Bind each new request to a fresh `RequestIdentity`, coordinator-issued source identity, driver digest, and journal record. Preserve historical request IDs, attempts, receipts, artifacts, and failed results. If a request is pending or requires reconciliation, reconcile that exact request and attempt; never submit a fresh retry to hide an unknown outcome.

Run the configured, hash-verified `python.exe` with `-B` from the repository root, using `-m tools.jenkins.jenkins_coordinator --repo-root <absolute-repo-root>`. The pilot entry provides `status`, `submit --request-file <absolute-json> --bundle <absolute-zip>`, `reconcile --request-file <absolute-json>`, `cancel --request-file <absolute-json>`, and `verify --request-file <absolute-json>`. Use the same request file for the entire lifecycle. A bundle must already have been issued by the coordinator for that request; agents must not invoke `snapshot.capture` or `snapshot.materialize` directly. Functional tests needing captured source fixtures also require an implemented coordinator preparation path. Include new files only through an explicit allowlist. A submitted queue ID or terminal Jenkins result alone is not validation acceptance: require `verify` to accept exact local and archived bytes and native termination evidence.

When the configured service is stopped, use the tray or its `start` management entry within the task's authorization. It performs the existing keeper recovery protocol after confirmed native death. Never replace the configured root, manufacture an activation receipt, or automatically start a service merely to answer a read-only status request.

Keep all compiler products and caches under root-level `D:\cargo-targets`, `E:\cargo-targets`, or `F:\cargo-targets`. Do not use repository-local targets, aliases, nested lookalike roots, or other drives. Do not print credentials, agent secrets, or raw Java command lines.

## Acceptance boundary

Coordinator operation enables functional tests and bounded command submission. It does not grant whole-workspace, milestone, migration, or production acceptance, and it does not authorize commits, pushes, publication, or external notifications. Record those gates separately while preserving retired coordinator data and historical receipts.

## References

- [Activated pilot entry](../../../tools/jenkins/jenkins_coordinator.py)
- [Independent pilot](../../../tools/jenkins/pilot/)
- [Windows tray and recovery workflow](../../../docs/tooling/jenkins-tray.md)
- [Pilot operator contracts](../../../docs/tooling/jenkins-coordinator-pilot.md)
- [Acceptance plan](../../../docs/plans/jenkins-coordinator-pilot.md)
