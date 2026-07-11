---
related_code:
  - tools/session_coordinator/web
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/server.py
implementation_files:
  - tools/session_coordinator/web
  - tools/session_coordinator/control_plane/assets.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/http_security.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/workflows/projections.py
  - tools/session_coordinator/server.py
plan_sources:
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/web/src/__tests__
  - tools/session_coordinator/tests/test_control_assets.py
  - tools/session_coordinator/tests/test_artifact_downloads.py
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_security.py
  - tools/session_coordinator/tests/test_control_snapshot.py
  - tools/session_coordinator/tests/test_workflow_projections.py
  - tools/tests/workflow_control_center_smoke.py
doc_type: plan-output-record
---

# M2 Read-Only Web Console Output Records

- Owner plan: `../01-workflow-control-center-and-tray.md`
- Session: `workflow-control-center-20260711-1915`
- Milestone state: `accepted`
- Completion date: 2026-07-11

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 证据 |
|---|---|---|---|
| M2 | M2.1 independent web shell | `accepted` | 独立 React/Vite/TypeScript 包、十个稳定路由、中文只读壳层、健康栏与生产 `dist` 已落盘；无 Hub 运行时、Tauri、DTO 或持久化耦合 |
| M2 | M2.2 strict contracts and SSE | `accepted` | 所有快照域具备 ID、枚举、数组及 nullable 契约；真实生产形状覆盖默认 `goal`、空 topology、独立 workflow、validation-copy manifest、Finalize 分类映射和命令矩阵；断连/重连、去重、缺口重同步测试通过 |
| M2 | M2.3 workflow pipeline | `accepted` | 目标至通知七阶段布局、节点侧栏、依赖/下游、租约、Failure 关联、Artifact 下载、不可变 attempt 历史和显式聚焦/回焦已实现 |
| M2 | M2.4 domain pages and bounded rendering | `accepted` | Sessions、Failure 图、租约/Patch、Cargo/验证副本、里程碑提交、审计与日志页面已实现；日志支持范围加载、暂停/跟随、过滤和固定行高虚拟化，长文本单行裁剪并保留完整提示 |
| M2 | M2.5 production assets and downloads | `accepted` | `/ui/` 深链、no-store/immutable 缓存、API 不回退、opaque Artifact、根目录约束、16 MiB/8 MiB 有界下载与单范围响应已通过测试；dist 审计强制 `/ui/` base、哈希资源和敏感材料缺失 |
| M2 | M2.6 accessibility, docs, evidence | `accepted` | 非颜色状态文本、ARIA 标签、键盘可聚焦虚拟列表、侧栏聚焦恢复、断线状态、操作指南、1280×800 与 1568×1003 截图完成；独立复审为 0 Critical / 0 Important |
| M2 | M2-T acceptance | `accepted` | 下列正式门禁全部通过 |

## M2-T 验证证据

- `npm --prefix tools/session_coordinator/web ci`：成功，147 packages，0 vulnerabilities。
- `npm --prefix tools/session_coordinator/web run check`：26/26 测试通过；TypeScript 严格检查、Vite 生产构建、dist verifier 全部通过。
- 生产资源：`dist/index.html` 含 `<base href="/ui/" />`；生成 `index-pYgooGZC.js` 与 `index-Cyji9sdH.css`，均为内容哈希资源。
- `python -m unittest tools.session_coordinator.tests.test_control_assets tools.session_coordinator.tests.test_artifact_downloads tools.session_coordinator.tests.test_control_http -v`：13/13 通过。
- 扩展生产者/消费者回归：assets、artifact、HTTP、安全、workflow projections、snapshot 共 19/19 通过。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/tests/workflow-control-center-smoke.Tests.ps1 -ReadOnlyConsole`：通过；Observer ticket 消费、深链、哈希资源缓存及无 mutation surface 均符合预期。
- `git diff --check -- tools/session_coordinator tools/session_coordinator/web docs/cli-and-tooling docs/plans/zircon_tooling/session_coordinator/01`：通过。
- 独立代码审查：Critical 0，Important 0。

## 视觉与安全证据

- `docs/tests/workflow-control-center/control-center-1280x800.png`
- `docs/tests/workflow-control-center/control-center-1568x1003.png`
- 两张截图均来自隔离状态目录下启动的真实协调器与生产 `dist`，截图后临时服务和状态目录已删除。
- 浏览器 bundle 不含 bearer token、维护能力或企业微信 webhook；bootstrap ticket 仅在隔离进程内短时消费，未写入仓库或证据。
- Vite 对单个主 chunk 给出体积优化提示，但不影响 M2 正确性；代码拆分作为 M6 负载与发布优化项处理。
