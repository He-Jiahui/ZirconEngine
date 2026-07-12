---
related_code:
  - tools/session_coordinator/control_plane/assets.py
implementation_files:
  - tools/session_coordinator/control_plane/assets.py
plan_sources:
  - docs/superpowers/specs/2026-07-11-workflow-control-center-and-tray-design.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_control_assets.py
doc_type: plan-output-record
---

# Web Content Security Policy

Owner plan: `../01-workflow-control-center-and-tray.md`

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | 生产 HTML 严格 CSP 与浏览器能力收束 | `completed` | 2026-07-13 | 真实 `/ui/` 审计发现 `Cache-Control: no-store` 与 `nosniff` 存在，但设计要求的 CSP 缺失。测试先以 `Content-Security-Policy` KeyError 失败；静态 HTML 响应现强制 `default-src 'none'`、same-origin script/style/font/connect、无 form/frame/referrer，并关闭 camera/geolocation/microphone/payment/USB。`test_control_assets` 3/3 通过，hashed asset 缓存策略保持不变。生产 daemon 将在安全滚动重启后采用该响应头。 |
