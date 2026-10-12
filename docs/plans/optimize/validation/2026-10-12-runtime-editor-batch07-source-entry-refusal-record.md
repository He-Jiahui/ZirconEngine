# Runtime and Editor batch 07 source-entry refusal record

**Status:** Source preparation refused before request creation. Batch 07 was not dispatched; Native, performance, product, and Jenkins acceptance remain pending.

## Decision and scope

The frozen batch 07 source-preparation review has `sourcePreparationAccepted: false`. Its refusal is recorded in the [root review](../../../../.codex/outbox/runtime-editor-native-batch07-original-contract-v1/source-preparation-review-v1.json), SHA-256 `cc7888b8f28456007809ea837eaa997495e7253e3b42c99e28d09763912b6c68` (2,889 bytes). The independent [Native supervision peer review](../../../../.codex/outbox/runtime-editor-native-batch07-original-contract-v1/native-supervision-peer-review-v1.json), SHA-256 `a87701c50582484deb501a06b7d5eaee174f5da69faf152f5b5979e4316c80ca` (12,683 bytes), also records refusal.

This is a preparation refusal, not a Native result. No actual request directory or context transition was created, no Native command or main process was invoked, and no Rust cases or performance samples ran. The frozen v1/v2 proposals and preparer remain unchanged.

## Review findings at the recorded review

The expected pinned helper closure contained 19 files. The preparer's first helper read-guard exited 1 after finding 17 exact matches and two changed, unreviewed files: `tools/dev/local_cargo.py` and `tools/jenkins/frontend.py`. It stopped before the original 507-input gate. The peer review separately performed one physical scan of the 507 frozen input names plus four added named repair guards (511 total). At that review, it found 12 effective repair paths and the separately reviewed 59-byte foreign `zircon_runtime/src/lib.rs` change, while identifying three unreviewed tooling-input mismatches: `tools/dev/local-cargo.ps1`, `tools/dev/local_cargo.py`, and `tools/jenkins/frontend.py`.

Those counts describe the supplied review artifacts at their recorded time. They do not claim that all Source paths still match now; later Source adoption may have changed files. The two helper files and all three tooling inputs remain preserved as unowned changes. No helper was restored, rewritten, or blindly re-baselined. The earlier Runtime 03 contexts remain byte-identical; metadata backups are metadata, not Source copies.

The peer's entry observation says the current local CLI rejects the independent-preview/evidence options and routes ordinary commands to Jenkins. Its HEAD comparison does not establish the exact body delta from the frozen raw identity. The user has indicated that an existing coordinator entry will be provided; no executable entry is part of this refusal record. A later authorized batch must bind the then-current Source and a verified entry before preparation or dispatch.

## Frozen command contract remains pending

The proposed validation contract retains all 15 original commands and 14 original producers, plus the two Runtime 03 matrix producers, for 16 total producer identities. It retains the existing arguments, features, flags, selectors, markers, matrices, samples, workloads, thresholds, and timeout scopes. The planned v2 floors remain 59 unique ordinary cases for command #2, 193 for command #15, and 12 ignored-library matches for command #9. These are pending contract values; none were measured in a dispatched run.

Native validation, Rust cases, accepted matrix samples, full performance results, product acceptance, Jenkins acceptance, and the overall milestone all remain pending. The source-entry refusal must not be represented as test success or a performance result.
