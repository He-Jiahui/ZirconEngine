---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: artifact-range-numeric-boundary
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/tests/test_artifact_downloads.py
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_downloads.ArtifactDownloadTests.test_extremely_long_range_numbers_are_typed -v
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_downloads.ArtifactDownloadTests.test_non_ascii_range_digits_are_typed -v
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_downloads.ArtifactDownloadTests.test_download_is_opaque_bounded_and_range_capable -v
  - python -B -m unittest tools.session_coordinator.tests.test_artifact_downloads -v
resolved_at: 2026-08-30
---

# Coordinator01: artifact Range numeric boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-009` HTTP malformed-input review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns artifact HTTP range parsing and its typed
  response projection; the defect is independent of artifact producers and
  product Cargo owners.

## 失败现象与复现证据

`ArtifactDownloadService._range()` accepted arbitrary-length and non-ASCII
decimal captures
and passed them directly to `int()`. Three 5,000-digit forms, start-only,
suffix-only, and explicit end, escaped as Python `ValueError` instead of the
typed `invalid_range` contract. In the download path that exception was
misclassified as `artifact_not_found`, hiding a malformed Range header as an
artifact identity failure.

## 最低共享层根因

The regex bounded Range syntax but not numeric width. The parser therefore
relied on the interpreter's process-wide integer conversion limit instead of
the artifact service's durable signed-64-bit size domain.

## 架构修复验收

- decimal range components are bounded before integer conversion;
- start, suffix, and explicit-end overflow all return typed `invalid_range`;
- the HTTP artifact projection returns 416 with the durable artifact size;
- ordinary closed, suffix, and open-ended ranges remain byte exact.

## 禁止临时方案

- Do not raise Python's integer conversion limit or catch the error only at the
  outer artifact lookup boundary.
- Do not weaken Range tests, silently clamp malformed numbers, or expose parser
  exceptions to callers.

## 修复结果与回传

- 根因：Range syntax accepted unbounded Unicode decimal captures and delegated numeric width to Python int conversion.
- 架构修复：Use ASCII-only Range digits and reject values wider than the signed 64-bit decimal domain before conversion across start, suffix, and end components.
- 验证：Artifact download suite passed 8/8, including 5000-digit start/suffix/end and non-ASCII digit regressions; py_compile and scoped diff checks passed.
- 回传：Returned Coordinator01 artifact Range parsing with bounded typed invalid_range behavior and HTTP 416 projection.
