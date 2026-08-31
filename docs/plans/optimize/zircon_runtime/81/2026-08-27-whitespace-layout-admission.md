---
title: Runtime Text Whitespace Layout Admission
category: zircon_runtime
report_id: Runtime81-whitespace-layout-admission-2026-08-27
date: 2026-08-27
session_id: root-runtime81-whitespace-layout-admission-20260827
implementation_status: implementation_complete
validation_status: static_validation_complete_managed_pending
---

# Runtime Text Whitespace Layout Admission

## Finding

The render-command layout resolver used `!text.trim().is_empty()` as its admission rule. A command
containing only spaces, tabs, or hard separators therefore kept its source but never received a
`UiResolvedTextLayout`. The owner-overlap prewarm collector used the same rule, so whitespace that
did reach another layout path also missed normal prewarm.

Whitespace is not an empty document. Spaces own shaped advance and caret/selection geometry; tabs
own tab-stop expansion; CR/LF/CRLF and Unicode hard separators own physical line boundaries, height,
baseline, caret, selection, and IME geometry. Trimming may be a wrapping, justification, form
validation, query, or label policy, but it cannot decide whether layout exists.

## Implemented Boundary

Both render-command resolution and owner-overlap prewarm now admit every non-empty source with a
valid frame. A truly empty ordinary display source still skips layout. The existing editable-owner
empty-source route remains intact because it must publish editable/caret state even without glyphs.
The `should_justify_line` trim guard is intentionally unchanged: whitespace-only lines receive
layout but are not expanded by justification.

The regression resolves three commands together: spaces and a hard separator must receive layout,
while an empty display source must not. It also checks that a spaces-only command enters the normal
prewarm path rather than forcing synchronous shaping during resolution.

## Evidence And Open Work

Rust 2024 formatting for the touched render owner/tests, scoped diff checks, exact source guards, and
the 246/483-line file budgets pass. Managed Cargo, tab/separator corpus, caret/selection/IME
integration, real WGPU pixels, and PNG evidence remain pending. Whitespace glyphs generally paint no
visible pixels, so final visual acceptance must use neighboring visible glyphs or selection/caret
geometry rather than a pure-text strategy screenshot.

Status: `nonempty_whitespace_layout_admission_implemented /
owner_prewarm_whitespace_admission_implemented / empty_display_source_fast_path_preserved /
justify_policy_unchanged / static_checks_complete / managed_validation_pending`.
