---
title: Editor52 Borrowed View Descriptor Open
category: zircon_editor
report_id: Editor52-borrowed-view-descriptor-open-2026-08-25
date: 2026-08-25
session_id: root-editor52-capability-binary-lookup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor52 Borrowed View Descriptor Open

## Scope

This slice removes whole-`ViewDescriptor` clones from the registry open and restore paths. It keeps
the current instance ownership API and does not close Editor52's descriptor content-provider,
catalog generation, localization, template validation, or product-truth gaps.

## Implementation

`open_descriptor` and `restore_instance` previously cloned the complete registered descriptor
before capability validation. That duplicated title, icon, persistence policy, presets, templates,
document metadata, constraints, and every required-capability string even when a single-instance
view was already open. The registry now borrows descriptor metadata through validation:

- repeated single-instance open returns the existing owned `ViewInstance` without cloning any
  descriptor metadata;
- first open copies `multi_instance` and `workbench_slot` and clones only the title required by the
  new owned instance;
- restore copies only `multi_instance` before mutating registry indexes;
- capability rejection, instance identity reuse, counter restoration, host mapping, and public
  returned-instance ownership are unchanged.

## Performance Contract

| Evidence | Retired path | Optimized gate |
| --- | ---: | ---: |
| Whole descriptor clones per repeated open | 1 | 0 |
| Whole descriptor clones per restore | 1 | 0 |
| Clone-pressure matrix | 256 capability strings, 11 alternating samples x 2,000 opens | optimized P95 <= 75% of retired P95 |

The 256-capability case is a deliberate extension-metadata pressure matrix, not a claim about the
default built-in catalog size. The ignored release benchmark emits
`EDITOR52_VIEW_DESCRIPTOR_BORROW_BENCH_V1` with both P95 timings, reduction percentage, sample and
iteration counts, capability-string count, and descriptor-clone counts.

## Validation

The managed batch covers capability-gated open, single-instance reuse, restore, all canonical
workbench host mappings, the descriptor-borrow source contract, and the ignored release benchmark
in one Cargo invocation. Exact Rust 1.94.1 `rustfmt --check`, scoped `git diff --check`, and source
guards passed before submission (apart from existing CRLF notices). Test execution, measured P95,
integration SHA, and automatic WeCom performance delivery remain coordinator-owned and pending.
