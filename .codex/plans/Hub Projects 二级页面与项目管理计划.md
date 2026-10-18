# Hub Projects 二级页面与项目管理计划

## Summary
把 Projects 首页从“内联表单 + 紧凑表格”调整为“Dashboard + 二级页面”结构。Dashboard 保持参考图中的卡片与快捷入口；`New Project`、`View All Projects`、列表模式、项目详情都进入 Projects 内部二级页面，不占用全局导航。项目版本绑定 Hub 已注册的 Source Engine；Pinned、项目绑定版本、详情操作作为 Hub 自有项目元数据持久化。

## Key Changes

- **Projects 页面路由**
  - 新增 Projects 内部页面状态：`dashboard | new-project | project-browser | project-detail`。
  - `New Project` 打开 `new-project` 二级页；关闭/返回回到 dashboard。
  - `View All Projects` 打开 `project-browser`，不再复用 `Show More` 或展开卡片逻辑。
  - 顶部列表模式按钮也打开 `project-browser`，作为宽松项目选择面板。

- **New Project 二级页**
  - 主内容区放项目核心设置：项目名、项目地址、版本/Source Engine 选择、创建按钮、校验状态。
  - 右侧放模板选择栏；模板不做大面积主视觉。
  - 模板列表展示多项，但第一版只启用当前真实可创建的 `Renderable Empty`；2D/3D/Sample 等显示为不可用状态。
  - 创建项目仍走现有 Editor 启动参数 `--create-project --template renderable-empty`，选择不可用模板时禁用创建。

- **Project Browser / Project List**
  - 宽松列表行替代紧凑表格：更高行距，显示项目图标/封面、名称、路径、版本、缺失状态、Pin 状态、详情按钮。
  - 列表数据优先显示 Pinned Projects；如果没有 pinned 项目，则显示 Recent Projects。
  - 搜索、筛选、排序保留在二级页顶部；Dashboard 的 `Show More` 只控制卡片展示数量，不影响 project-browser。
  - 点击行选择项目；详情按钮进入 `project-detail`。

- **Project Detail 二级页**
  - 展示项目名称、路径、存在/缺失状态、最近打开时间、当前绑定 Source Engine。
  - 操作：打开、更换引擎版本、收藏/取消收藏、从 Hub 移除、彻底删除。
  - “从 Hub 移除”只移除 Hub recent/pin/engine 元数据，不删除磁盘文件。
  - “彻底删除”需要二次确认；Windows 优先通过系统回收站删除，失败则不删除并显示错误。非 Windows 第一版显示不可用状态。

## Interfaces / Data
- `HubConfig` 增加 Hub 自有 `project_metadata`，按规范化项目路径记录：
  - `pinned: bool`
  - `engine_id: Option<String>`
  - `last_selected_template: Option<String>`
- 不把这些 Hub 元数据写入 Editor recent JSON；Editor recent 继续只同步项目 recent 信息。
- `RecentProjectRowData` / 项目 UI 数据扩展为统一 Project item projection：包含 `pinned`、`engine-id`、`engine-label`、`missing`、`selected`、`can-open`、`can-delete`。
- Runtime 新增 callbacks：打开 Projects 子页、选择模板、选择项目 engine、pin/unpin、remove-from-hub、request-delete、confirm-delete。

## Test Plan
- `cargo fmt -p zircon_hub --check`
- `cargo check -p zircon_hub --locked --offline --jobs 1`
- `cargo test -p zircon_hub --locked --offline --jobs 1`
- `cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1`
- Unit tests:
  - pinned 项目优先；无 pinned 时回退 recent。
  - project metadata 持久化且不污染 Editor recent JSON。
  - New Project 只允许 enabled template 创建。
  - project detail 的 remove-from-hub 不删除磁盘。
  - recycle-bin 删除命令/Windows 路径构造正确；不在测试中真实删除用户目录。
- Visual validation:
  - 截图 dashboard、new-project、project-browser、project-detail。
  - 对照用户截图确认：New Project 主设置居中突出，模板在右侧；Project Browser 宽松；Recent `View All Projects` 与 `Show More` 不再表现为同一逻辑。

## Assumptions
- “版本”指 Hub 已注册的 Source Engine 记录，不是单独的字符串版本表。
- 第一版模板展示多项，但只启用真实可创建的 `Renderable Empty`。
- “彻底删除”默认走二次确认 + 回收站，不做无保护递归硬删。
- 这些页面仍属于 Projects 内部二级页面，不新增全局导航项。
