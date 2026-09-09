---
related_code:
  - zircon_app/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_host/src/lib.rs
  - zircon_hub/src/lib.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
implementation_files:
  - zircon_app/src
  - zircon_runtime/src
  - zircon_editor/src
  - zircon_runtime_interface/src
  - zircon_runtime_host/src
  - zircon_hub/src
  - zircon_plugins/plugin_sdk/src
plan_sources:
  - user: 2026-09-09 API 公开接口覆盖审计
tests:
  - tools/wiki_site.py
doc_type: category-index
---

# Rust API 参考

本分区从 crate root、公开模块和 feature gate 出发，回答“能从哪里调用、谁拥有返回值、失败后怎样恢复、升级时检查什么”。它不是 rustdoc 的替代品：rustdoc 提供精确签名，本 Wiki 解释跨模块语义、生命周期和使用策略。

## 阅读顺序

| 目标 | 首选页面 | 继续阅读 |
| --- | --- | --- |
| 启动 runtime/editor/server | [zircon_app](app.md) | [运行时启动教程](../tutorials/runtime-module-startup.md) |
| 调用核心运行时模块 | [zircon_runtime 模块](runtime-modules.md) | [核心运行时](../core-runtime/index.md) |
| 注册命令、网关或编辑器插件 | [zircon_editor](editor.md) | [编辑器](../editor/index.md) |
| 实现动态宿主或 DLL 边界 | [runtime interface](runtime-interface.md) | [动态 ABI 交接](../mechanisms/ui-render-and-dynamic-abi-handoff.md) |
| 构建插件包 | [plugin SDK](plugin-sdk.md) | [插件](../plugins/index.md) |
| 审核覆盖缺口 | [覆盖矩阵](coverage-matrix.md) | 源码 root 与 rustdoc |

## 公开性的四个层次

| 层次 | 识别方式 | 对调用方的含义 |
| --- | --- | --- |
| crate-root re-export | `pub use ...` 出现在 `lib.rs` | 首选调用路径；重构时最可能被维护 |
| public module | `pub mod name` | 可通过模块路径访问，但不保证每个子项稳定 |
| feature-gated public | `#[cfg(feature = ...)] pub ...` | 编译期存在性取决于 feature 集合 |
| internal public | workspace 内部 crate 的 `pub`，或 `pub(crate)` | 只服务当前实现，不应宣传成第三方承诺 |

`pub` 是 Rust 可见性，不是语义版本保证。特别是 `zircon_runtime_interface` 的 crate 文档明确说明它是内部 lockstep 边界；带版本后缀的布局仍可能由宿主和 runtime 同步升级。

## API 条目应包含什么

每个稳定条目的完整说明至少包含：

1. 完整 Rust 路径以及需要的 Cargo feature。
2. 类型或函数解决的业务问题。
3. 输入、输出、错误类型和错误后的恢复策略。
4. 所有权、借用、句柄释放和跨线程限制。
5. 生命周期阶段，例如 build、ready、tick、finish、cleanup。
6. 最小调用示例以及对应源码/测试。
7. 与 ABI、序列化 schema、插件 manifest 或项目格式的兼容性关系。

## 路径选择

优先使用 crate-root re-export：

```rust
use zircon_editor::{EditorCommandDescriptor, EditorCommandRegistry};
```

只有根未导出或需要明确领域边界时才使用模块路径：

```rust
use zircon_runtime::asset::facade::{AssetLoadState, Handle};
```

不要引用实现文件中的私有模块路径。模块被拆分时，私有路径可以在不迁移调用者的情况下改变。

## Feature 选择

```toml
[dependencies]
zircon_runtime = { path = "../zircon_runtime", default-features = false, features = ["graphics", "ui", "text"] }
```

选择原则：

- headless 产品不启用窗口、graphics 或 editor feature。
- UI 依赖 text shaping 时同时声明 `ui` 与 `text`，不要依赖传递 feature 偶然开启。
- 动态插件需要 runtime、host、interface 和 SDK 的 ABI 版本成组验证。
- 测试若使用 `--all-features`，仍需至少一条最小 feature 构建避免隐藏错误依赖。

## `Result` 与错误分类

API 调用方应按错误是否可恢复分类，而不是把所有错误转换为字符串：

| 类别 | 示例 | 默认策略 |
| --- | --- | --- |
| 输入错误 | 非法路径、超限 payload、错误 schema | 拒绝请求并保留诊断 |
| 暂态错误 | 资源未 ready、设备暂时不可用 | 有界重试或延迟到下一帧 |
| 能力不支持 | feature/ABI slot/backend 缺失 | 禁用功能并报告 capability |
| 生命周期错误 | 重复释放、错误 generation、shutdown 后调用 | 终止该操作并修复调用顺序 |
| 数据损坏 | manifest/hash/长度不一致 | 隔离产物并要求重新导入 |

## 所有权与句柄

Rust 内部 API 通常由类型系统表达所有权；跨 DLL/FFI 边界则必须显式表达：

- `ZrRuntimeSessionHandle`、`ZrRuntimeViewportHandle` 等是不透明身份，不代表 Rust 引用。
- `ZrOwnedByteBuffer` 与 allocation ID 必须由分配它的 runtime 释放。
- host 不得把裸指针保留到契约允许的调用窗口之外。
- generation 变化后，旧句柄应视为 stale，即使整数值仍相同。
- callback 不得在未知线程上直接操作仅主线程资源。

## ABI 调用检查顺序

```mermaid
flowchart LR
    Load[加载动态库] --> Version[读取 API/ABI 版本]
    Version --> Shape[校验结构大小与必需 slot]
    Shape --> Session[创建 session]
    Session --> Calls[提交事件/帧/操作]
    Calls --> Drain[读取 owned 输出]
    Drain --> Release[调用 runtime release]
    Release --> Destroy[销毁 session/卸载库]
```

文字等价说明：先验证版本和函数表形状，再创建 session；每次调用后先消费并释放 runtime-owned 输出；所有 session 与 allocation 释放完成后才能卸载动态库。

## 异步操作

submit/poll/harvest API 使用句柄关联阶段：

1. submit 成功仅表示请求被接纳，不表示业务完成。
2. poll 返回 pending 时，调用方应遵守 frame cadence 和超时预算。
3. terminal outcome 需要 harvest 才能取得 owned payload。
4. harvest 后同一 handle 是否仍有效由具体契约决定，不应重复消费。
5. shutdown 前应 drain 或 cancel 未完成操作。

## 生命周期型服务

模块和 manager 的常见阶段：

| 阶段 | 允许行为 | 禁止行为 |
| --- | --- | --- |
| build | 注册依赖、分配私有状态 | 假设其他模块已 ready |
| ready | 解析服务、建立可用资源 | 无界阻塞主线程 |
| tick | 处理帧输入、发布输出 | 持有跨帧临时借用 |
| finish | 停止接收新工作、drain | 创建新的长任务 |
| cleanup | 释放资源、撤销注册 | 访问已清理依赖 |

## 示例可信度标签

- “可复制入口”只使用公开 re-export，并应能在对应 feature 下编译。
- “调用形状示意”说明概念顺序，构造参数可能省略，不能直接复制。
- “ABI 布局示意”只解释字段关系，不能替代 `#[repr(C)]` 源码。

看到示意代码时必须沿 `related_code` 路径核对当前签名。API 页面若没有标注示例层级，应视为需要修正文档。

## 升级审计

升级 workspace 版本时逐项检查：

- crate root 是否新增、删除或移动 re-export。
- feature 默认值及依赖传播是否变化。
- 错误 enum 是否新增必须处理的变体。
- ABI 结构大小、字段顺序、版本常量和 symbol 名是否变化。
- 句柄释放、callback 线程和异步 terminal 状态是否变化。
- 现有教程是否仍只使用公开路径。

## 自动清单命令

```powershell
$roots = @(
  'zircon_app/src',
  'zircon_runtime/src',
  'zircon_editor/src',
  'zircon_runtime_interface/src',
  'zircon_runtime_host/src',
  'zircon_hub/src',
  'zircon_plugins/plugin_sdk/src'
)

rg --no-heading --line-number `
  '^pub use|^pub mod|^pub (trait|struct|enum|type|const|fn)' `
  $roots
```

feature 审计：

```powershell
rg --no-heading --line-number '^\[features\]|^[A-Za-z0-9_-]+\s*=\s*\[' `
  zircon_app/Cargo.toml `
  zircon_runtime/Cargo.toml `
  zircon_editor/Cargo.toml `
  zircon_runtime_interface/Cargo.toml `
  zircon_runtime_host/Cargo.toml `
  zircon_hub/Cargo.toml `
  zircon_plugins/plugin_sdk/Cargo.toml
```

rustdoc 审计：

```powershell
cargo doc -p zircon_app -p zircon_editor -p zircon_runtime_interface `
  -p zircon_runtime_host -p zircon_hub -p zircon_plugin_sdk `
  --all-features --no-deps
```

## 文档验证

```powershell
.wiki-venv\Scripts\python.exe tools/wiki_site.py validate --strict-metadata --json
.wiki-venv\Scripts\python.exe tools/wiki_site.py build --strict-metadata --output site --json
```

验证通过只证明元数据、导航和链接一致，不证明所有 API 语义完整；覆盖矩阵中的 `partial` 必须继续作为开放缺口维护。

## 当前审计结论

- crate-root re-export 已在本分区按 app、editor、runtime-interface 和 plugin SDK 分组。
- `zircon_runtime` 以 public module 为主，当前按模块/feature 列出，字段级覆盖仍不完整。
- runtime host 与 Hub 主要通过公开模块暴露，稳定性低于明确 root re-export。
- 所有 ABI 类型必须结合版本与释放规则阅读，不能因 Rust 路径可见就假设可跨版本兼容。
