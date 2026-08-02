---
title: WOS188 Overload ability content
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS188 Overload Ability Content

## Scope

Add source Mage level-14 `overload` to the generated WOC M4 ability projection.
Its source profile is an off-GCD Arcane self buff: zero cost, instant,
30-second cooldown, value `0.4`, and duration `10`.

The paired choice `mag_r14_overload` already grants this stable ability id
through the generated talent modifier catalog; this milestone makes that grant
resolvable by the WOC ability catalog without hand-authored content drift.

## Delivery Order

1. Add a source-pinned content guard before changing the retained-id list.
2. Extend the generator scope and expected count from 82 to 83, then regenerate
   the JSON and Zr catalog/effect projections.
3. Verify generated identities and the existing ability-content regressions.
4. Implement the amp/cost runtime transaction in a separate slice; content
   projection does not claim that behavior.

## Status

| Milestone | Scope | Status | Date |
|---|---|---|---|
| WOS188a | Source-pinned content contract | completed | 2026-08-02 |
| WOS188b | Generated ability projection | completed | 2026-08-02 |
| WOS188c | Static regression | completed | 2026-08-02 |
| WOS188d | Independent secondary review | completed | 2026-08-02 |

## Dynamic Validation

No dynamic behavior changes in this content-only slice. The later runtime
package will run only through `zr_vm:project`.

## Verification

The projection fingerprint is `a1b0389b8630d9c`; both generators and the
WOS86/WOS177/WOS179-WOS186 adjacent static guards pass against the pinned
source commit.
