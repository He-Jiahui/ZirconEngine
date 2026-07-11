# M3 受控操作与权限提升

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | M3.0 计划模块提交前缀 | `completed` | 2026-07-11 | 提交服务从 Session 注册的编号计划父目录派生 `【module】`，自动补全无前缀主题，拒绝与计划目录不匹配的前缀；关闭校验器独立复核，技能固定 `【{plan-folder}】<Conventional Commit>` 格式，已补单元与 PowerShell 验收用例，统一在 M3-T 执行。 |
| M3 | M3.1 schema v16 与强类型 Action Catalog | `completed` | 2026-07-11 | 新增 `action_requests`、`action_approvals`、`web_elevation_grants`及不可变审批触发器；v16 在 SQLite 边界闭合 Action kind，并为已应用早期 v15 的数据库补兼容触发器；risk/status/parameters 同为闭合枚举或数据类，未提供通用 shell/Git/Cargo/SQL/path 入口，M4/M5 红色动作仅登记且 `enabled=false`。 |
| M3 | M3.2 一次性提升与 CSRF | `completed` | 2026-07-11 | CLI/托盘通过 runtime 凭据签发 actor/role/daemon/可选 Session 绑定的短期 grant；浏览器只能消费，不能自签发；提升时旋转 CSRF，后续 mutation 必须同时持有 `SameSite=Strict` HttpOnly cookie 与 `X-CSRF-Token`，实例、actor、Session、过期、重放和降级均实施拒绝。 |
| M3 | M3.3 Preview/Fingerprint/Confirm | `completed` | 2026-07-11 | 预览持久化 Action ID、影响范围、警告、短语 hash 与 120 秒过期；指纹覆盖 HEAD、index diff、baseline、Session、lease、Failure Markdown/graph、Patch、Validation Copy、Cargo、plan hash、服务实例及服务派生目标文件；Confirm 的复算与副作用共用服务级 mutation gate，不一致写入 `state_changed` 且零副作用。 |
| M3 | M3.4 黄色动作执行器 | `completed` | 2026-07-11 | 按数据类直接调用 Session、Lease、Patch、Validation Copy、Failure 与 Workflow Store；执行严格使用预览锁定的 Patch/Validation/Failure 资源集；validation 仅接受服务端模板，并在 gate 内登记子进程后异步完成，保留安全取消能力，不转发浏览器 argv/path。 |
| M3 | M3.5 确认 UI 与操作历史 | `completed` | 2026-07-11 | 新增“受控操作”页、Session 页局部入口、权限提升表单、风险/影响摘要、原因与确认短语对话框、本页审计历史；红色动作可见但禁用；`action_state_changed` 自动获取新的只读预览并展示新增/移除影响与指纹差异，但绝不自动重试 mutation；React 使用文本渲染影响项以防 HTML 注入。 |
| M3 | M3.6 权限、生命周期与拒绝码文档 | `completed` | 2026-07-11 | 更新 Workflow Control Center 与 Local Session Coordinator 机器可读头、schema 16 现状、CLI 提升命令、信任边界、Action API、原子 Preview/Confirm/Execute 流程、稳定拒绝码、不重试策略与 `【plan-folder】` 自动提交规范。 |
| M3 | M3-T 里程碑验收 | `completed` | 2026-07-11 | 多轮独立审查持续暴露并清除了 mutation gate、异步完成、资源集过宽、资源 impact、数据库枚举及 Failure Markdown 文件级 TOCTOU；最终结构为动作专属最小资源集、后台完成同 gate、同一批 bytes 校验并从不可变快照解析。协调器全量 183 项、最终相关回归 63 项、Web 30 项及类型/构建/dist、两条 HTTP 烟雾门、技能 quick-validate、关闭校验器 27 项均通过；最终独立复审 0 Critical / 0 Important。 |

## 验收结论

- M3 已完成并通过验收；红色动作继续保持可见但禁用，服务排空仅允许预览。
- 独立只读代码审查最终为 0 Critical / 0 Important，达到里程碑提交门槛。
