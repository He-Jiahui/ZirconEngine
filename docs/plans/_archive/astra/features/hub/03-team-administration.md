---
status: in_progress
review_date: 2026-09-07
plan_family: docs/plans/astra/features/hub
parent_plan: docs/plans/astra/optimize/01-review-and-repair.md
owner_session: astra-full-domain-20260905
plan_sources:
  - docs/plans/astra/features/hub/02-local-service-authority.md
  - docs/plans/astra/layouts/01-responsive-layout.md
---

# Hub 团队管理桌面工作流

## 范围与合同

承接 [本地服务计划](02-local-service-authority.md) S2 的现有 typed mutation 和公开 DTO，在现有账号页面提供成员角色及启用状态编辑、显式所有权移交、创建邀请，以及组织已发邀请的分页、刷新和确认撤销。成员及访客不请求管理列表，owner/admin 权限只来自相同 issuer/subject 的有效成员记录；actor 不在初始页时继续读取既有分页，服务端保留最终授权。

组织、账号身份或 generation 切换会关闭弹窗并清除草稿。新提交只创建一个 operation UUID，复用 native journal；pending/unknown 阻止新写入，恢复仅发送 operationId、backendEpoch 和 generation。拒绝后的组织状态必须先刷新，再由用户重新确认；不会自动替换 policyRevision 重试。刷新会找回后续分页上的所选组织和操作目标。浏览器不保存 token、凭据或 mutation payload。

桌面修改位于 `zircon_hub/web/src/account/` 的 `AccountPanel.tsx`、`MemberAdministration.tsx`、`IssuedInvitations.tsx`、`controller.ts`、`protocol.ts` 和 `copy.ts`。直接回归为 `zircon_hub/web/tests/account_admin.test.mjs`；本批保留原有 `account_state.test.mjs`。

## 验收边界

浏览器测试运行真实 React 组件、controller、protocol 和 invoke 调用路径，Tauri IPC、账号、服务响应及操作日志由 fixture 提供。它覆盖 owner/admin/member/viewer、分页 actor、后续页组织刷新、拒绝后的再次确认、unknown 恢复、身份/组织切换清草稿、已发邀请状态及撤销，并不证明 Windows Credential Manager、原生 journal、真实 HTTP/OIDC、撤销重试或进程重启已通过验收。

四类弹窗在 360/768/1280/1920 宽、900 高及 English/Chinese 下分别截图，共 32 张。检查窗口水平溢出、弹窗控件边界，等待字体与过渡完成后捕获。`account_admin_shell.mjs` 从既有 Rust `ui_text`、`HubTextBundle` 和消息字面值读取测试所需文案，核对全部 UI 字段；导航、Team 副标题和就绪状态也验证语言，英文整页禁止残留中文 UI。该夹具避免只改 `settings.language` 后保留中文 shell DTO，但没有执行 Rust 原生投影，不能充作其运行证据。

当前截图目录为 `C:/Users/HeJiahui/.codex/visualizations/2026/09/06/01a0780b-97a2-7380-9b15-1e90591a6b4b/hub-admin`，本批仅认定 `account-admin-{edit,transfer,invite,revoke}-{360,768,1280,1920}-{English,Chinese}.png`；同目录旧 `account-*` 文件不计入本批。复核样本包括 360 中文撤销、1280 英文邀请；修复夹具后导航和背景状态与所选语言一致。

真实 Windows Tauri + Keycloak + 本地服务的登录、组织授权、成员与邀请操作、断线/退出/重启恢复仍须按服务计划完成。此次未运行 Cargo，不把 browser fixture 通过或既有构建证据提升为 native acceptance；全路由矩阵及其他布局 finding 的状态不由本条证据关闭。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| S2 桌面团队管理 | typed 成员/所有权/邀请工作流与响应式弹窗 | `browser_fixture_validated_native_pending` | 2026-09-07 | 组合回归 `account_admin.test.mjs` 27 项 + `account_state.test.mjs` 29 项，共 56/56，0 skipped；最终 TypeScript 类型修正后 controller/protocol 10/10，`npm run build` 通过（仅现有大 chunk 提示）。本次只修改截图夹具及记录，修正整页语言后布局专项 8/8、0 skipped，重新生成上述 32 张截图；原生验收待服务计划。 |
