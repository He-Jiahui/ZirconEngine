# Zircon Hub 本地闭环 v1 功能实现设计

## Summary
- 目标：把当前 Tauri + React/MUI 版 `zircon_hub` 做成本地可用的桌面 Hub。
- 默认语言：中文；设置页可切换 `中文 / English`，所有页面标题、按钮、状态、错误恢复、空状态和“敬请期待”统一走本地化文案。
- Cloud 不做远程账号/同步/市场服务；v1 只做本地包输出、设备安装、服务预留状态。
- 未完成或远期功能统一显示“敬请期待”，但必须有明确禁用态、说明和未来能力分类。

## Key Changes
- IPC/API：扩展 `hub_action` 请求为 `{ actionId, targetId?, payload? }`，前端统一通过 `dispatchHubAction()` 发送 typed payload；Rust 侧按 action id 解析明确 payload 类型。
- 状态模型：在 `HubViewModel` 增加本地化后的 UI 文案、可编辑 settings draft、项目创建模板、功能可用性、comingSoon entries、异步任务进度。
- 异步任务：构建、打包、安装、打开编辑器改为后台任务队列，立即返回 running 状态，并通过已有 `hub-state-changed` 事件推送进度、完成、失败和恢复建议。
- 本地化：新增 Rust 侧 `HubTextBundle` 或等价模块，默认 `Chinese`；React 页面不再硬编码业务文案，只消费 DTO 或本地化 key。
- 架构边界：所有 Hub 功能实现留在 `zircon_hub`；不把 Hub 注册进 `zircon_runtime`，不新增 `server` 命名层，不修改运行时/编辑器权威模型，除非只调用既有编辑器启动协议。

## Functional Implementation
- Projects：
  - New Project 表单提交真实调用 `create-project`，payload 包含 `name/location/template/engineId`；只启用 `renderable-empty`，其他模板显示“敬请期待”。
  - Import Project 走文件夹选择，校验 `zircon-project.toml`，写入 recent + metadata，进入项目详情。
  - Project Detail 增加 pin/unpin、remove-from-hub、request-delete、cancel-delete、confirm-delete；删除仅 Windows 回收站，失败保留 Hub 状态。
  - 搜索、筛选、排序、视图模式保持持久化，默认中文标签。
- Editor：
  - Open Editor 使用已选项目；无项目时允许打开空编辑器；缺 staged editor 时给出“先构建编辑器/运行时”的恢复提示。
  - Source Engine 选择、注册、校验、构建历史展示真实化；新增打开 staged output 文件夹动作。
  - 编辑器插件页只读展示 editor-scoped plugins，未支持安装/启停的操作显示“敬请期待”。
- Builds：
  - Build 运行 `tools/zircon_build.py --targets editor,runtime`，后台推送 running/success/error。
  - Package 复制项目到配置的 package output，写 `zircon-package.toml`。
  - Install 先确保 package，再复制到本地 device install 目录。
  - History 展示 command line、log excerpt、output dir、recovery；支持打开 output dir。
- Assets / Plugins / Learn：
  - Assets/Plugins 保持 selected project 优先、source engine 次之的只读目录发现。
  - Learn 增加 open-resource，仅允许打开当前 catalog 中真实存在的文档。
  - 资产导入、插件安装/启停/市场下载标为“敬请期待”。
- Cloud：
  - 改名语义为本地交付中心：Packages、Installs、Reserved Services。
  - Packages/Installs 读取 action history 和输出目录；远程同步、账号、云仓库全部 disabled + “敬请期待”。
- Team：
  - 读取 selected project Git 仓库，缺失时回退 source engine 仓库。
  - 展示 git identity、contributors、recent Hub actions。
  - 邀请成员、权限、远程协作显示“敬请期待”。
- Settings：
  - 所有路径、工具链、build profile、jobs、language 可编辑并保存。
  - Browse buttons 调用 folder picker；保存后注册/更新 Source Engine 并刷新 catalogs。
  - Configuration Health 从真实设置和路径状态计算，不硬编码百分比。

## Test Plan
- Rust：覆盖 action payload 解析、项目创建/导入/删除、settings 保存、Source Engine 注册、后台任务状态、package/install、learn open-resource、comingSoon DTO、本地化文案。
- React/TS：`npm run typecheck` 和 `npm run build`；校验页面只通过 shared action dispatcher 调用后端。
- Hub 合同测试：更新现有 `project_*`、`ui_*`、`tauri_react_shell_contract`，确保 React/MUI cutover、不回退 Slint/旧 app 模块。
- 视觉验证：运行 Tauri Hub，截图 Projects/New/Detail/Editor/Builds/Cloud/Settings/中文默认/错误/运行中/空状态，确认无溢出、遮挡、英文硬编码残留。
- 验收命令：Hub scoped `cargo fmt/check/test`、`npm run typecheck`、`npm run build`、必要时 `npm run tauri:build`。

## Assumptions
- v1 只做本地桌面闭环，不实现远程账号、云同步、插件市场、团队权限服务。
- 中文为首次启动默认语言；用户设置可覆盖并持久化。
- 现有大规模未提交工作视为当前基线，实施时不回滚无关改动。
- Build 可能耗时较长，因此必须改为非阻塞后台任务，不能继续阻塞 Tauri command。