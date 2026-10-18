# `zircon_hub` 独立启动器设计

## Summary
- 新增顶层 workspace 包 `zircon_hub`，作为默认启动入口，使用真实 Slint 构建独立桌面 Hub；Slint 只存在于 Hub，不恢复 `zircon_editor` 旧 Slint 路径。
- Hub 启动后保持打开，负责 Projects、Installs、Settings 三页；选择创建/打开项目后，以独立子进程启动 `zircon_editor`。
- Hub 管理最近项目、源码引擎、构建设置，并与现有 Editor 配置做双向同步，方便直接启动 Editor 时仍能看到最近项目。

## Key Changes / Interfaces
- Workspace:
  - 添加 `zircon_hub` workspace member 和 `zircon_hub` 二进制。
  - 打包工具新增 `hub` target；默认打包入口为 `zircon_hub.exe`，同时继续产出 `zircon_editor.exe`、`zircon_runtime.dll`。
- Hub 模块结构:
  - `app/`: Slint 初始化、主窗口、页面路由、事件绑定。
  - `state/`: Hub 快照、选中导航、任务状态、搜索过滤。
  - `projects/`: 最近项目、项目校验、创建/打开请求、与 Editor recent 同步。
  - `engines/`: Source engine 注册、源码目录校验、构建产物定位、构建状态。
  - `build/`: 调用 `tools/zircon_build.py --targets editor,runtime --out <output> --mode <debug|release>`，捕获日志和退出码。
  - `settings/`: Cargo/Rustup/Python 路径、默认源码产物目录、默认项目目录、语言、构建 profile/jobs。
  - `process/`: 启动 Editor、打开文件夹、子进程生命周期。
- Editor 启动参数:
  - 新增 GUI 启动请求：`--project <path>` 打开已有项目。
  - 新增 GUI 创建请求：`--create-project --project-name <name> --location <dir> --template renderable-empty`。
  - 有上述参数时绕过自动恢复 last project；无参数直接启动 Editor 时保留当前 fallback 行为。
- Hub 配置:
  - Hub 自有配置放在用户配置目录下，例如 Windows `%LOCALAPPDATA%\ZirconHub\config.toml`。
  - Editor recent 同步读写现有 `%LOCALAPPDATA%\ZirconEngine\config.json` 的 `editor.startup.session`。
  - recent 冲突按 `last_opened_unix_ms` 合并，路径去重，最多保留 8 条。
- Slint UI:
  - `ui/app.slint` 主壳：左侧窄导航栏、顶部标题/操作区、右侧页面内容。
  - `ui/projects.slint`：最近项目列表、搜索、新建/打开按钮。
  - `ui/installs.slint`：源码引擎列表、Locate Source、Build、Open Output、Launch Editor。
  - `ui/settings.slint`：工具链路径、产物目录、默认项目目录、语言、构建参数。
  - 视觉按 Unity/Tuanjie Hub 风格：暗色、左侧导航、8px 内圆角以内、列表型信息密度，不做 marketing hero。

## Launch Flow
```text
zircon_hub
  -> 读取 Hub 配置 + 合并 Editor recent
  -> 用户创建/打开项目
  -> 如选择源码引擎且无可用产物，先构建 staged ZirconEngine
  -> 启动 <engine-output>/ZirconEngine/zircon_editor.exe --project <path>
     或 zircon_editor.exe --create-project ...
  -> Hub 保持打开并记录任务状态
```

## Test Plan
- Hub 单元测试:
  - 配置读写、默认路径生成、recent 合并去重、源码引擎目录校验。
  - build command 生成符合 `tools/zircon_build.py` 参数约定。
  - Editor launch args 对 Windows 路径、空格路径、相对路径处理正确。
- Editor 集成测试:
  - `zircon_editor --project <valid>` 进入项目模式。
  - `--create-project ...` 创建 renderable empty 后进入项目模式。
  - 无 Hub 参数时当前直接启动行为不回归。
- 工具验证:
  - `cargo check -p zircon_hub --locked`
  - `cargo check -p zircon_app --features target-editor-host --no-default-features --locked`
  - `python tools/zircon_build.py --targets hub,editor,runtime --out <tmp> --mode debug --dry-run`
- 手动验收:
  - Hub 默认启动；Projects/Installs/Settings 三页可切换。
  - 添加源码引擎、构建、启动 Editor。
  - 创建项目后 Editor 打开项目，Hub 仍保持打开。
  - 关闭再开 Hub 后 recent 和 settings 保持。

## Assumptions
- `zircon_hub` 是新增顶层包的例外，符合用户指定；它不参与 `zircon_runtime` 生命周期，不注册 engine module。
- 真实 Slint 依赖只加在 `zircon_hub`，不恢复 editor 旧 `slint_host`、`.slint` 业务 UI 或兼容 shim。
- v1 只实装 Projects、Installs、Settings；后续页面通过导航注册表扩展。
- 源码引擎 v1 只支持 source install；构建使用现有 `tools/zircon_build.py` staged 产物路径。
- 当前工作区有大量未提交改动；实现时不得回退无关改动。
