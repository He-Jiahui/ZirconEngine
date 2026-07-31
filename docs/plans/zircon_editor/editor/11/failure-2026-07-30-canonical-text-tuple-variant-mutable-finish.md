---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: canonical-text-tuple-variant-mutable-finish
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/zircon_editor/editor/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/serialization/text/canonical_writer.rs
  - zircon_runtime_interface/src/serialization/tests/write_contract.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime -SkipTest
  - cargo test -p zircon_runtime_interface --locked --jobs 1 -- --test-threads=1
---

# Editor11: canonical tuple variant mutable finish

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：PF-M1 current-source Runtime build before cold/warm WGPU PNG and RenderDoc evidence
- 修复责任计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 交接原因：`zircon_runtime_interface::serialization::text::canonical_writer` owns the shared `TempSpool` lifetime and mutable sink access. Render17 must consume that public interface and cannot add a renderer-local serialization bypass.
- 生命周期键：`canonical-text-tuple-variant-mutable-finish`

## 失败现象与复现证据

The managed Windows Runtime build started successfully in
`D:\\cargo-targets\\zircon-engine\\pool\\841a130ffbd3fd2e938e76b488988119044b676acced751dae7166d95d7f1025`, then stopped while compiling `zircon_runtime_interface`:

```text
error[E0596]: cannot borrow `self.spool` as mutable, as `self` is not declared as mutable
  --> zircon_runtime_interface/src/serialization/text/canonical_writer.rs:298:37
295 |     fn finish(self) -> Result<(), CanonicalTextWriteError> {
298 |                 CountingWriter::new(self.spool.file_mut()?, ...);
```

Reproduction:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipTest
```

The same file's `CanonicalArray::finish(mut self)` does not mutate its receiver, while `CanonicalTupleVariant::finish(self)` does. The current source therefore has the two receiver mutability requirements inverted.

## 最低共享层根因

`CanonicalTupleVariant::finish` owns a `TempSpool` and writes its closing bracket through `TempSpool::file_mut()` before transferring the spool to `write_single_object`. Its receiver must be mutable for that write. This is a compile-time ownership invariant in the shared canonical text writer, below all Render17 graph, UI, frame-profile, PNG, and RenderDoc paths.

## 架构修复验收

- `CanonicalTupleVariant::finish` has a mutable receiver for its closing spool write, and `CanonicalArray::finish` does not retain a redundant mutable receiver.
- Editor11's focused canonical writer and interface serialization gates compile and pass without reintroducing an owned whole-document or consumer-local writer path.
- The original managed `zircon_runtime` build passes the `zircon_runtime_interface` boundary, after which Render17 reruns its PF-M1 focused test, ignored WGPU PNG export, and RenderDoc capture.

## 禁止临时方案

- Do not add a Render17-specific serializer, feature gate, alias, fallback, or test-only bypass around `canonical_writer`.
- Do not weaken canonical output, finite-float, bounded-buffer, or sink-error contracts to hide the compile error.
- Do not restore a complete owned text document as a substitute for correctly mutating the temporary spool.

## 修复结果与回传

Open state: `待修复`; no Render17 build, PNG, or RenderDoc gate is claimed passed.

## 产出记录与时间

- 2026-07-30 | `open / lowest-owner-compile-repaired / full-interface-pending` | Current source now keeps `CanonicalArray::finish(self)` non-mutable and uses `CanonicalTupleVariant::finish(mut self)` for the closing write through `TempSpool::file_mut()`. The fix is the shared serializer ownership contract itself; no Render17-local serializer, feature branch, fallback, or owned-document restoration was added. Editor11's current-source focused gate passed `4/4`, the broader serialization gate passed `62/62` active tests with only the explicit 512 MiB case ignored, and Runtime15 job/run `191a43af42de46b18f2d3529a48a875a` / `2635a9abb3774c28ade162b7a25b5a98` compiled the complete current `zircon_runtime_interface` and `zircon_runtime` lib-test boundary without E0596 before its exact filter selected zero tests. Full interface package validation, final exact33 review, failure return, and the originating Render17 PF-M1 rerun remain open; no PNG or RenderDoc acceptance is claimed by Editor11.
- 2026-07-30 | `open / full-interface-green / origin-rerun-pending` | Fresh full-package reservation/job/run `3b301ffebcb54fb881e02291fe40e2d4` / `ac76251fad174b18a80bc3dbaa1745a3` / `dd5e4239cf664f038f51934dc38292a4` compiled and passed the complete Interface package: unit `338/0/1`, integration `3/3`, doc-tests `0`; 402-input pre/post fingerprint was `18a0b4f7ad2a3f6e0255e76a44f8080b9c067cb8e78d8bb189327cec732d962a`. E0596 is therefore closed at its owner boundary. Final expanded-scope review, fixed return, managed commit and the originating Render17 PF-M1 rerun remain open.
