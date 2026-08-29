---
handoff_kind: fixed
status: fixed
created_at: 2026-08-30
summary_slug: control-http-body-framing-boundary
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_control_http.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_invalid_content_length_is_typed
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_truncated_content_length_is_typed
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_unsupported_transfer_encoding_is_typed
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_legacy_command_endpoint_uses_same_framing_boundary
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_legacy_command_endpoint_rejects_truncated_body
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_legacy_command_endpoint_types_invalid_json
  - python -m unittest tools.session_coordinator.tests.test_control_http.ControlHttpTests.test_legacy_command_endpoint_types_invalid_utf8
resolved_at: 2026-08-30
---

# Coordinator01: control HTTP body framing boundary

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：`TOOL-COORD-P2-009` HTTP/JSON/descriptor malformed-input review
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Coordinator01 owns the shared control HTTP adapters; this bounded
  framing defect is independent of product and Cargo owners.

## 失败现象与复现证据

The HTTP adapter parsed `Content-Length` with an unguarded `int()` call, so a
malformed value became an internal 500 instead of a stable client error. It
also ignored HTTP/1.1 `Transfer-Encoding`; a chunked request was read as a
zero-length body, leaving encoded bytes on the persistent connection. RED
covered malformed length, duplicate length, transfer encoding, and oversized
numeric headers, and short-body framing through direct adapter/handler tests.

## 最低共享层根因

The control adapter had no single framing contract. The `/control/v1/*` path
and legacy `/command` path performed separate body handling, and neither
validated the wire framing before JSON dispatch. Their malformed-body error
classification also diverged, with legacy JSON decode errors exposing parser
text as `invalid_request` instead of the stable `invalid_json` contract.

## 架构修复验收

- malformed and conflicting lengths return typed 400 responses;
- any transfer encoding is rejected instead of being silently treated as empty;
- oversized numeric lengths are classified before integer conversion;
- a body shorter than its declared length is rejected as
  `incomplete_request_body`;
- legacy invalid UTF-8 and JSON decode errors use the same typed `invalid_json`
  response as the control router;
- valid bounded bodies continue to be read exactly once, with the one MiB cap;
- both control and legacy command paths use the same parser and focused tests.

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth,
  test-only bypasses, or call-site exceptions.
- Do not weaken framing limits or return raw parser exceptions to callers.

## 修复结果与回传

- 根因：Control and legacy command HTTP paths lacked one bounded framing parser, leaving Content-Length conversion, duplicate lengths, transfer encoding, and short reads inconsistently classified.
- 架构修复：Route both entrypoints through one exact, one-MiB Content-Length parser that rejects transfer encoding, duplicate/malformed/oversized lengths, and incomplete bodies with typed sanitized errors.
- 验证：Full Control HTTP suite passed 36/36, command protocol passed 16/16, FailureGraph passed 33/33, and py_compile plus scoped diff checks passed.
- 回传：Returned Coordinator01 control HTTP framing with one bounded parser and uniform typed errors across control and legacy command endpoints.
