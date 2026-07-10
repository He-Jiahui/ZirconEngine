---
name: support-first-regression-testing
description: Enforce bottom-up regression diagnosis and repair for hierarchical features and tests. Use when an upper-layer feature, integration, constructor, or end-to-end test fails and the real fault may live in lower-layer shared support behavior such as parsing, symbol resolution, function implementation, type inference, runtime helpers, or shared execution paths. Requires listing possible lower-layer support functions first, fixing the lowest broken shared layer, then re-running validation upward instead of inventing an upper-layer-only success path.
---

# Support First Regression Testing

## Overview

Use this skill to force bottom-up thinking when a higher-level test fails. The goal is to repair the shared support layer first, then verify the higher-level feature again through the normal path.

## Core Rule

- Never make an upper-layer feature pass by adding a unique correct path while a lower-layer shared support function is still broken.
- Never treat a constructor, integration point, or orchestration layer as allowed to bypass a broken primitive just to satisfy its own test.
- Fix the lowest broken shared layer first, then re-run regression checks upward.

## Workflow

1. Record the failing upper-layer behavior.
   - State the failing feature, test, or scenario.
   - State what the expected normal path should have been.

2. Enumerate all plausible lower-layer support functions before fixing anything.
   - List the supporting layers that could feed the failure.
   - Typical candidates include parsing, AST construction, binding, symbol lookup, function implementation, argument passing, type inference, type checking, runtime helpers, dispatch, constructor glue, object initialization, shared data structures, and reusable execution paths.
   - Add project-specific candidates instead of assuming the list is complete.

3. Find the lowest shared failing layer.
   - Add or run focused checks for each candidate layer.
   - Prefer the lowest layer whose failure would also explain the upper-layer symptom.
   - Stop climbing upward once a lower shared support bug is confirmed.

4. Fix the lower shared layer first.
   - Repair the shared support behavior itself.
   - Keep the upper-layer code on the normal path unless the lower-layer fix proves an upper-layer issue remains.
   - Reject any patch whose only virtue is that the upper-layer test turns green while the shared lower layer remains wrong.

5. Regress upward in order.
   - Re-run the focused lower-layer test first.
   - Re-run the immediate parent behavior next.
   - Re-run the original upper-layer regression after the lower layers are green.
   - Only investigate upper-layer logic if the normal path still fails after the lower shared layer is fixed.

## Example

- If a constructor test fails, do not immediately add constructor-specific logic.
- First list the supporting layers: argument evaluation, function implementation, field assignment, initialization helpers, dispatch, type checks.
- If the real problem is the underlying function implementation, fix that function and its focused tests first.
- Then re-run the constructor regression to confirm the constructor now works through the shared correct path.

## Red Flags

- "Just special-case the constructor."
- "Patch the upper layer so this one scenario passes."
- "This feature can have its own path even if the shared helper is wrong."
- "I can make the top-level test green first and clean up later."

## Reporting

- State the upper-layer failure you started from.
- State the lower-layer candidates you enumerated.
- State which lowest shared layer was actually broken.
- State which lower-layer checks passed after the fix.
- State which upper-layer regressions you re-ran after the lower layer was fixed.

## Related Skills

- `zircon-dev` for repository-specific build, test, and validation workflow.
- `superpowers:systematic-debugging` for root-cause-first debugging discipline.
- `superpowers:verification-before-completion` before claiming the regression is fixed.
