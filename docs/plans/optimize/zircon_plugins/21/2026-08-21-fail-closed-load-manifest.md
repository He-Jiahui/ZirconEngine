# Plugins21 fail-closed load manifest optimization record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/21-plugin-artifact-marketplace-third-party-package-install-update-trust-non-cargo-product-integration-review.md`
- Finding: `MPA-P0-001`
- Status: `validation_pending`

## Scope

- Reject a native load-manifest entry when its declared id differs from the parsed package id.
- Reject an entry when the parsed package manifest is outside its declared package root.
- Preserve both mismatch diagnostics when one entry violates both contracts.
- Keep duplicate-package handling for two otherwise valid entries with the same package id.

## Contract

- A mismatched entry never enters the discovered candidate set.
- `load_all_from_load_manifest` therefore performs no library-open attempt, creates no runtime registration report, and invokes no native entry for the rejected package.
- Validation collects all applicable id and path diagnostics before returning the single fail-closed admission result.
- Valid load-manifest discovery and true duplicate-package behavior remain unchanged.

## Performance Gate

- The release workload runs 21 alternating valid/rejected sample pairs with 16 full load-manifest admissions per sample.
- Rejected load-eligible candidates fall from 16 per sample to zero, a deterministic 100% reduction in downstream candidate load work.
- Timings use nearest-rank P95; rejected admission P95 must remain within 125% of the normal valid-admission P95.
- Measured P95 timings remain pending the grouped coordinator validation.

## Validation

- The refresh regression requires an id-mismatched package to publish zero candidates while retaining its diagnostic.
- The load-all regression requires a combined id/path mismatch to publish zero discovered/loaded candidates, zero registration reports, both diagnostics, and no `library-open` diagnostic.
- The duplicate regression now supplies two genuinely valid entries with the same id, so it continues to exercise duplicate suppression independently of mismatch rejection.
- The release performance marker is `PERF-MVP-PLUGINS21-LOAD-MANIFEST-REJECTION`.
- Cargo compilation, behavior tests, and release measurements are queued in the multi-task Plugins aggregate; no standalone Cargo run is claimed here.

## Remaining Plan Work

- This slice closes the currently executable id/root mismatch path in `MPA-P0-001`.
- Artifact digest and signer binding, target/ABI receipts, symlink or junction policy, case-folding rules, and TOCTOU-resistant handle verification remain open Plugins21 milestones.
- Marketplace resolution, installation transactions, rollback, revocation, quarantine, and Hub product workflows remain outside this focused loader admission change.
