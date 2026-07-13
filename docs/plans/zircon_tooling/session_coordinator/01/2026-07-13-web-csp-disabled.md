# Web CSP Disabled for MUI Runtime Styles

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | 关闭控制台 CSP | `completed` | 2026-07-13 | 浏览器控制台明确报告 `style-src 'self'` 拒绝 MUI/Emotion `_insertTag`；按用户要求移除 HTML 响应的 CSP，保留其他安全响应头。静态资源聚焦回归 `3/3` 通过。 |

## 原因与处理

生产控制台依赖 MUI/Emotion 在运行时插入组件样式。此前的静态 CSP 不含 nonce，也不允许 inline style，导致全部 MUI 组件样式被浏览器阻止，只剩全局背景 CSS 和裸文本。

本次直接关闭 CSP，不改前端结构，也不加入 `unsafe-inline`。`X-Frame-Options: DENY`、Permissions Policy、`X-Content-Type-Options: nosniff`、loopback Host/Origin、认证 Cookie 与 CSRF 校验继续生效。`Referrer-Policy` 改为 `same-origin`，修复原先 `no-referrer` 与后端同源读取校验互相冲突导致的 403；跨源请求仍不发送 referrer。

## 关联文件

- `tools/session_coordinator/control_plane/assets.py`
- `tools/session_coordinator/tests/test_control_assets.py`
- `docs/cli-and-tooling/workflow-control-center.md`
