---
related_code:
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/gltf_importer/runtime/src/plugin.rs
  - zircon_plugins/gltf_importer/dist/src/lib.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - tools/plugin_structure_audits/capability.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/engine-architecture/plugin-optional-feature-bundles.md
  - docs/engine-architecture/native-plugin-boundary.md
reference_engines:
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/UnrealEngine/Engine/Plugins/Bridge/Bridge.uplugin
---

# 04 · 插件 DX 与 SDK 工具链

## 1. 目标

把"写一个 Zircon 插件"的体验从"复制 gltf_importer 改 11 个文件、同一个 ID 抄三遍"收敛到：

- 一条脚手架命令生成完整插件骨架，元数据**单源声明、零重复**；
- 校验前移：manifest/符号/能力错误在 `cargo build`/CI 时以带修复提示的错误报出，而不是运行期加载失败；
- 失败可诊断：加载链每个失败点（发现/dlsym/ABI/manifest/能力协商/注册）都有类型化错误与结构化报告；
- 热重载语义实装（native save/restore harness），与 VM 插件终态路线（全系统重构方案）勾稽不冲突。

## 2. 现状与差距（证据）

- **三重声明**：`id = "gltf_importer"`（plugin.toml）、`PLUGIN_ID: &str`（runtime/src/capability.rs）、`b"gltf_importer\0"`（dist/src/lib.rs）；能力表、模块名、dist crate 名、入口符号名均声明两遍。改名要动 3+ 处且编译器不查。
- **接入面广**：新插件需 touch `zircon_plugins/Cargo.toml`、`first_party_runtime_catalog` 的 Cargo.toml + lib.rs（match 分支 + STATIC_PLUGIN_MANIFESTS）、`zircon_app/Cargo.toml` feature，共 ~11 文件。
- **校验后置**：plugin.toml 严格校验在 `tools/plugin_structure_audits/*.py`（回归脚本），非编译期；TOML 解析错误无行号定位，能力引用悬空无交叉检查。
- **诊断贫弱**：dlsym 失败不带期望符号名；abi_version 不匹配仅日志；能力协商失败只给 granted 集不给 missing 集（`zircon_app/src/entry/export_bootstrap.rs`）。
- **热重载空转**：native descriptor 的 `save_state/restore_state/unload` 均为 None；链接态插件不可重载。
- **文档缺口**：无"从零到可加载"教程；SDK 示例只有 editor 例子。

## 3. 设计决策

### 3.1 元数据单源：Rust 声明生成一切

单源放在插件 runtime crate 的一个声明宏/派生宏输入里（`zircon_plugin_sdk`）：

```rust
zircon_plugin_sdk::declare_plugin! {
    id: "my_feature",
    display_name: "My Feature",
    category: runtime,
    targets: [client_runtime, editor_host],
    capabilities: ["runtime.plugin.my_feature"],
    maturity: experimental,
    ...
}
```

由宏展开生成：ID/能力/模块名常量、`RuntimePluginDescriptor` builder、注册入口、dist 需要的 `\0` 结尾字节串与符号名。`plugin.toml` 改为**生成物**（`cargo zircon plugin sync-manifest` 或 build script 输出 + CI 一致性守卫），静态审计脚本继续消费 plugin.toml 但新增"与 Rust 单源一致"检查。方向固定为 Rust→TOML（编译器保住单源），不做 TOML→Rust 的反向 include 拼接。

### 3.2 目录接入自动化

`first_party_runtime_catalog` 的 match 分支 + STATIC_PLUGIN_MANIFESTS + 双 Cargo.toml feature 属于纯机械样板：脚手架负责生成，新增 `cargo zircon plugin check` 校验目录与 catalog 的一致性（漏注册、feature 名漂移、members 缺项），入 CI。

### 3.3 工具链形态

`tools/` 下新增 `cargo-zircon`（Rust 二进制，workspace 内 tool crate）：
- `cargo zircon plugin new <id> [--kind importer|system|editor] [--native]`：模板生成全骨架 + catalog/workspace 接线；
- `cargo zircon plugin check`：manifest 单源一致性、符号导出（对 dist 产物做 `dlsym` 探测）、能力交叉引用、catalog 勾稽；
- `cargo zircon plugin validate <path>`：对第三方插件包做加载前静态验证。
既有 Python 审计脚本保留为回归层，新工具是开发者面前的第一道门。

### 3.4 诊断与错误报告

加载链错误统一为 `PluginLoadError` 类型树（thiserror），每个变体携带：阶段、期望值/实际值、修复提示（例如 `MissingSymbol { expected: "zircon_native_plugin_descriptor_v3", path, hint: "dist crate 是否使用 native_dist_runtime_plugin_v3! 宏？" }`）。能力协商失败输出 `missing_required` / `denied` 明细。`NativePluginEntryReportV3.diagnostics` 字段成为强制填充项。

### 3.5 热重载

对齐 Fyrox 三步语义（prepare_to_reload→reload→on_loaded）挂到计划 02 的统一生命周期：native 插件实装 save_state/restore_state 函数表与 live-host 替换流程（宿主句柄稳定、实例可替换——与全系统重构方案的 VM 插件语义同一句柄模型，VM 执行器落地后本机制平移复用）。首批以 `native_dynamic_fixture` 与一个 importer 插件为验收样本。

## 4. 里程碑

### M1 元数据单源与 SDK 宏

实现切片：`declare_plugin!` 宏 + manifest 生成/同步机制；`gltf_importer` 与 `native_dynamic_fixture` 先迁为样板；其余 first-party 插件批量迁移（硬切换，同批删除手写常量文件）。

测试阶段（policy §3 最小批次）：
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- focused 过滤词批：`cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked declare_plugin manifest plugin_id`（宏展开/manifest 生成/迁移插件变更面）；插件工作区全量 test 留给波次收口（policy §4）
- 一致性守卫：生成的 plugin.toml 与库内快照 diff 为空；Python 审计脚本全绿。
- 验收证据：任一插件全文 `grep <id>` 只出现在单源声明与生成物；文档更新 `zircon_plugins/README.md`。

### M2 cargo-zircon 脚手架与检查器

实现切片：`cargo zircon plugin new/check/validate` 三命令；模板覆盖 importer/system/editor 三形态；`check` 接入 CI。

测试阶段（policy §3 最小批次）：
- 端到端：`cargo zircon plugin new demo_probe --kind system` → 生成插件包级 build + focused `cargo test --manifest-path zircon_plugins/Cargo.toml -p demo_probe --locked` 通过 → 删除 demo_probe；插件工作区全量 build/test 留给波次收口（policy §4）；
- `cargo test -p cargo-zircon --locked`（模板快照测试）；
- 验收证据：新插件从命令到可加载 ≤3 步（new → 填实现 → build）；`docs/` 新增 walkthrough 教程（“新插件五分钟指南”）。

### M3 加载诊断增强

实现切片：`PluginLoadError` 类型树替换加载链的字符串错误；能力协商 missing/denied 明细；export_bootstrap 与 native_plugin_loader 的报告结构化。

测试阶段（policy §3 最小批次）：
- focused 过滤词批：`cargo test -p zircon_runtime --lib --locked plugin_load native_plugin capability`（新增故障注入单测：坏符号/坏 ABI/坏 manifest/缺能力四类各有断言错误码与提示文案）；全量 lib 回归留给波次收口（policy §4）；
- `cargo test -p zircon_app --locked export_bootstrap plugin`；
- 验收证据：四类故障的错误输出快照入测试。

### M4 native 热重载 harness

实现切片：save_state/restore_state/unload 实装（fixture + 一个 importer）；live-host 替换流程与文件监视开发模式（仅 dev profile 开启）；与计划 02 生命周期钩子对接。

测试阶段（policy §3 最小批次）：
- focused 过滤词批：`cargo test -p zircon_runtime --lib --locked plugin_load hot_reload save_state restore_state`（重载状态迁移契约测试）；全量 lib 回归留给波次收口（policy §4）；
- 手工验收脚本：修改 fixture 源码 → 重编 dist → 运行中的 editor-host 完成替换且句柄不变；
- 验收证据：契约测试 + 替换日志；文档更新 `docs/engine-architecture/native-plugin-boundary.md` 勾稽。

## 5. 风险与回退

- **宏复杂度**：declare_plugin! 保持"数据声明"性质，禁止在宏里藏行为逻辑（generated-code boundary 规则）；宏出错信息需专门打磨（trybuild 测试）。
- **与 VM 插件终态的关系**：本计划不加深 native 主路径的公开面（native-plugin-boundary 债务方向不变），热重载机制设计为句柄模型可平移到 VM 插件。
- **第三方破坏**：SDK 对外形态变化集中在 M1 一次性发生，`sdk_api_version` 升 minor 并在 README 记录迁移表。
