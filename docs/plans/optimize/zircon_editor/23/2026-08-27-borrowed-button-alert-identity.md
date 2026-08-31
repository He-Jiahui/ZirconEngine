---
title: Editor23 Borrowed Button and Alert Identity
category: zircon_editor
report_id: Editor23-borrowed-button-alert-identity-2026-08-27
date: 2026-08-27
session_id: root-editor23-precompiled-collection-type-traits-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Borrowed Button and Alert Identity

## Scope

This batch removes transient identity strings from retained-host button kind/glyph selection and
Material Alert color-token selection. It preserves ASCII case-insensitive matching, global token
precedence, button kinds, glyphs and alert colors. It does not change layout, paint command order,
component data or public APIs.

## Changes

- Button classification exposes six borrowed node fields as a stack array and searches them with
  ASCII case-insensitive byte windows.
- Button kind and glyph classification retain their existing global priority while eliminating two
  compound `format!` plus lowercase strings per painted button.
- Button glyph selection no longer calls the legacy owned-key helper; that helper remains outside
  this integration candidate because its module carries an unrelated concurrent edit.
- Material Alert color selection uses five static `(semantic, colorPascal)` token pairs instead of
  building Pascal-case and formatted tokens in the loop.
- Rust regressions preserve mixed-case button kind/glyph priority and mixed-case alert color
  priority. One Python contract prevents owned normalization from returning.

## Deterministic Performance Evidence

The independent Windows release model classifies 65,536 representative nodes per task. Each task
uses 21 paired samples with alternating order; every reported sample is the median of three
sub-runs. Timings are current-thread CPU cycles from `QueryThreadCycleTime`, so scheduler pauses are
excluded while allocator and string-processing CPU costs remain measured. Fixture construction is
outside the measured functions.

### Button Kind and Glyph

| Evidence | Compound owned keys | Borrowed fields | Result |
| --- | ---: | ---: | ---: |
| Checksum | 1,409,024 | 1,409,024 | identical |
| Allocations | 524,288 | 0 | 524,288 fewer; 100% reduction |
| Run 1 P50 / P95 | 276,840,271 / 288,684,912 cycles | 107,626,623 / 111,634,783 cycles | 61.12% / 61.33% fewer cycles |
| Run 2 P50 / P95 | 268,808,635 / 289,858,699 cycles | 105,213,875 / 112,084,816 cycles | 60.86% / 61.33% fewer cycles |
| Run 3 P50 / P95 | 270,854,086 / 276,556,632 cycles | 106,404,595 / 108,565,020 cycles | 60.72% / 60.74% fewer cycles |

### Material Alert Color Token

| Evidence | Dynamic Pascal tokens | Static token pairs | Result |
| --- | ---: | ---: | ---: |
| Checksum | 196,606 | 196,606 | identical |
| Allocations | 707,782 | 0 | 707,782 fewer; 100% reduction |
| Run 1 P50 / P95 | 274,555,929 / 290,121,254 cycles | 42,194,335 / 43,714,992 cycles | 84.63% / 84.93% fewer cycles |
| Run 2 P50 / P95 | 268,490,513 / 282,613,399 cycles | 41,619,740 / 45,880,221 cycles | 84.50% / 83.77% fewer cycles |
| Run 3 P50 / P95 | 270,449,730 / 276,463,058 cycles | 41,482,392 / 43,942,092 cycles | 84.66% / 84.11% fewer cycles |

The model isolates identity classification and does not represent whole-button or whole-frame
paint time. Managed acceptance requires exact checksums and allocation counts, zero optimized
allocations, button P50/P95 reductions of at least 55%, and alert P50/P95 reductions of at least
75%.

## Acceptance

- TDD RED observed exact 4/4 failures before implementation; the same source contract passes 4/4
  after implementation.
- Python compilation, four Rust source/test formatting checks, model compilation, three calibrated
  runs and scoped diff checks pass locally.
- Both mixed-case Rust regressions are submitted in one coordinator Cargo invocation together with
  source contracts, formatting, performance gates and the scoped diff check.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Editor23 still owns the broader UI asset, binding, accessibility, theme, icon, flow and font-atlas
authoring review. This record accepts only button and Material Alert identity classification.
