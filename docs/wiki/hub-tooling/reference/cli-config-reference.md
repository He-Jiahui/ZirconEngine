---
related_code:
  - tools/cargo-zircon/Cargo.toml
  - tools/cargo-zircon/src/main.rs
  - zircon_hub/hub.toml
  - zircon_hub/src/settings
implementation_files:
  - tools/cargo-zircon/src
  - zircon_hub/src/settings
plan_sources:
  - docs/plans/zircon_hub/04-settings-draft-and-source-engine.md
tests:
  - zircon_hub/src/settings
  - tools/cargo-zircon/src/main.rs
doc_type: configuration-reference
---

# Hub 与 `cargo-zircon` 配置参考

配置文件决定项目根、引擎注册、构建 profile、服务端点和账户 journal 的默认位置；命令行参数决定一次动作的覆盖值。配置不是安全边界，所有路径和 URL 仍需在执行入口重新校验。

## Hub 配置分层

```text
hub.toml / settings file
  -> settings::paths
  -> runtime state
  -> project/engine scope
  -> command options
  -> process or service
```

`zircon_hub/hub.toml` 是开发部署示例，不应直接复制到生产环境。settings 模块负责配置路径、HubConfig 和 BuildProfile 的读取/投影；变更后应重启 Hub 以清空旧快照和任务状态。

## 构建覆盖

Hub 构造 `BuildCommandOptions` 时应显式给出 python、cargo、source、output、profile 和 jobs。CLI 不接受任意 shell 片段；每个值都是独立参数或受控路径。

```powershell
cargo zircon plugin check --root E:\Projects\Game --artifact-root E:\Builds\artifacts
cargo zircon validation-batch E:\Projects\Game\tools\zircon-validation-runtime74-batch.ps1
```

推荐把 CI 的 root、artifact-root、profile、target 和 toolchain 写入流水线配置并打印脱敏摘要，而不是依赖当前工作目录。

## 路径解析规则

| 配置 | 约束 |
| --- | --- |
| project root | 必须包含 `zircon-project.toml` |
| source engine | 必须通过 `validate_source_engine` |
| output root | 不得嵌入项目根，建议位于独立构建盘 |
| artifact root | receipt verify 的受控根目录 |
| journal path | 绝对路径、有文件名、禁止 `..` |
| service URL | HTTPS；loopback HTTP 需显式允许 |

## TOML 示例

```toml
[build]
profile = "debug"
jobs = 8
output_root = "E:/ZirconBuilds"

[hub]
recent_projects_limit = 32
```

字段名和可用 profile 以当前 `HubConfig`/`BuildProfile` 源码为准。示例表达配置形状，不代表所有版本都接受额外字段；未知字段应让解析器报错并提示迁移。

## 环境和平台

Windows Hub 使用 `LOCALAPPDATA` 作为默认账户操作存储；`cargo-zircon validation-batch` 在 Windows 调用 `pwsh.exe`，并将 Cargo target 投影到受管目录。Linux/macOS 下涉及 Windows picker、credential store 和 PowerShell 的入口必须通过平台 guard 或由对应工具替代，不能假设同一 executable 存在。

## 迁移策略

1. 读取配置到临时结构。
2. 校验版本、路径、URL 和未知字段。
3. 生成新的规范化设置。
4. 写入新文件并保留旧文件备份。
5. 重启 Hub，重新读取 snapshot。

配置迁移不能把旧的 secret 复制到新日志。账户凭据由 CredentialStore 独立迁移，HubConfig 只保存非 secret 引用。

## 与 Unreal/Godot 的对照

Unreal Project Settings 与 AutomationTool 参数常把平台、配置和输出组合在一起；Godot export presets 则把项目导出选项保存为文件。Zircon 通过 HubConfig、BuildCommandOptions 和 receipt request 分开表达运行时设置、一次性动作和发布证明，便于审计但要求调用者明确映射。

## 诊断

配置错误应报告字段名、允许范围和恢复 hint，不报告 refresh token、私钥或完整认证 URL。CLI 参数错误返回 usage 和非零码；文件不存在与字段非法应保持可区分，以便 CI 分类。

## 检查清单

- [ ] CI 不依赖隐式当前目录。
- [ ] output/artifact root 位于受控目录。
- [ ] profile、jobs、toolchain 记录可复现。
- [ ] Windows-only 入口有平台 guard。
- [ ] 配置未知字段和超长文件被拒绝。
- [ ] 日志不包含 secret。

## 相关页面

- [引擎安装与构建](engine-builds.md)
- [账户配置](account-service.md)
- [`cargo-zircon` CLI](cargo-zircon-cli.md)
