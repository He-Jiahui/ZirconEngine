---
related_code:
  - zircon_hub/src/engines/mod.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/engines/validation.rs
  - zircon_hub/src/build/command.rs
  - zircon_hub/src/build/runner.rs
implementation_files:
  - zircon_hub/src/engines
  - zircon_hub/src/build
plan_sources:
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
tests:
  - zircon_hub/src/build/command.rs
  - zircon_hub/src/build/runner.rs
doc_type: api-reference
---

# 引擎安装、源码校验与构建配置

Hub 的引擎模型围绕 `SourceEngineInstall` 展开：源码目录是输入，`output_dir/ZirconEngine` 是 staged 输出，构建记录是可展示的事实。引擎注册表不拥有 Cargo 构建逻辑；它只负责选择、去重、绑定和展示历史。

## 公共入口

```rust
pub fn validate_source_engine(path: impl AsRef<Path>) -> SourceEngineValidation;
pub fn source_engine_id(source_dir: &Path) -> String;
pub fn source_engine_display_name(source_dir: &Path) -> String;
pub fn same_source_engine_path(left: &Path, right: &Path) -> bool;
pub fn active_source_engine<'a>(engines: &'a [SourceEngineInstall], id: &str)
    -> Option<&'a SourceEngineInstall>;
pub fn upsert_source_engine(engines: &mut Vec<SourceEngineInstall>, engine: SourceEngineInstall);
pub fn ensure_active_source_engine(engines: &mut Vec<SourceEngineInstall>, id: &str)
    -> Result<(), HubError>;
```

`source_engine_id` 是注册键，不应使用显示名替代。`same_source_engine_path` 处理规范化路径相等；在 Windows 上尤其不要直接比较 `PathBuf` 的字符串表示。`upsert_source_engine` 对同一 ID 更新已有记录，调用者应在更新前保留当前项目绑定。

## `SourceEngineInstall` 与历史

```rust
pub struct SourceEngineInstall {
    pub id: String,
    pub display_name: String,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub last_build_unix_ms: Option<u64>,
    pub build_history: Vec<SourceBuildRecord>,
}

pub struct SourceBuildRecord {
    pub finished_unix_ms: u64,
    pub status: String,
    pub profile: String,
    pub jobs: Option<u16>,
    pub output_dir: PathBuf,
    pub detail: HubMessage,
    pub log_excerpt: HubMessage,
    pub command_line: Vec<String>,
}

impl SourceEngineInstall {
    pub fn staged_engine_dir(&self) -> PathBuf;
    pub fn record_build(&mut self, record: SourceBuildRecord);
}
```

`staged_engine_dir()` 永远在 `output_dir` 下追加 `ZirconEngine`。`record_build` 只在 `status == "success"` 时更新 `last_build_unix_ms`，并将历史限制为最近 8 条。失败构建仍必须保留 `detail`、`log_excerpt` 与完整命令行，便于 UI 重试和审计。

## 校验结果

`SourceEngineValidation` 提供 `summary()` 和 `recovery_hint()`。调用方应同时使用二者，而不是把枚举 Debug 文本作为用户消息。典型结果包括路径不存在、不是源码根、缺少构建脚本和可用源码根。

```text
source_dir
  -> validate_source_engine
  -> BuildCommandOptions
  -> BuildCommand::for_editor_runtime
  -> run_build_command
  -> BuildExecutionReport
  -> SourceBuildRecord
```

## 构建命令

```rust
pub struct BuildCommandOptions {
    pub python_path: PathBuf,
    pub cargo_path: PathBuf,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub profile: BuildProfile,
    pub jobs: Option<u16>,
}

impl BuildCommand {
    pub fn for_editor_runtime(options: &BuildCommandOptions) -> Self;
    pub fn new(program: PathBuf, args: Vec<String>, cwd: PathBuf,
               capture_dir: PathBuf) -> Self;
    pub fn with_source_output(self, output_dir: impl AsRef<Path>) -> Self;
    pub fn command_line(&self) -> Vec<String>;
}
```

`for_editor_runtime` 生成 `tools/zircon_build.py --targets editor,runtime --out ...` 的参数形状，并注入 profile、Cargo 路径和可选 jobs。不要把 `command_line()` 当作 shell 字符串执行；它返回参数数组，必须交给 `std::process::Command` 的 `.args()`。

## 执行报告

```rust
pub fn run_build_command(
    command: &BuildCommand,
    cancellation: &TaskCancellationToken,
) -> Result<BuildExecutionReport, HubError>;

impl BuildExecutionReport {
    pub fn process_exited_successfully(&self) -> bool;
    pub fn summary_line(&self) -> String;
    pub fn recovery_hint(&self) -> String;
    pub fn log_excerpt(&self) -> String;
}
```

执行器创建 capture 目录、捕获 stdout/stderr、检查取消令牌并等待子进程结束。启动前已取消时不生成进程；运行中取消时应终止并将状态标记为 cancelled。报告摘要适合 UI，`log_excerpt` 适合展开面板，完整日志应存放在受控 capture 目录。

## profile 选择

| profile | 用途 | 关注点 |
| --- | --- | --- |
| `debug` | 日常迭代 | 编译快，禁止作为发布性能结论 |
| `release` | 性能和分发 | 记录 toolchain、target 和 receipt |
| 自定义 `BuildProfile` | Hub 配置投影 | 必须通过设置层校验 |

jobs 为 `None` 时交给构建脚本默认并行度；设置数值时应限制为正数且记录在 `SourceBuildRecord.jobs`。

## 与 Unreal/Godot 的对照

Unreal Launcher 将已安装引擎和项目绑定为可选择版本；Hub 用 `SourceEngineInstall` 表达源码引擎，因此可以显示“源码 checkout + staged 输出”的双路径。Godot 编辑器管理器通常直接打开项目并在后台导入；Hub 把构建、导入和启动拆成可取消任务，失败后可按结构化消息恢复。

## 常见错误与恢复

1. 源码校验失败：修正 checkout 或选择正确仓库根，不要写入 output。
2. Python/Cargo 不存在：检查 `BuildCommandOptions` 路径，重新生成命令。
3. 输出目录冲突：选择新的 staged 根，避免删除已有产物。
4. 进程被取消：保留 capture 与 cancelled 记录，下一次从干净任务开始。
5. 构建失败：优先阅读 `summary_line` 和 `recovery_hint`，再展开日志。

## 最佳实践

- 把源码 ID、显示名和输出路径分开保存。
- 构建命令使用参数数组，不经 shell 拼接。
- 成功和失败都写入历史，只有成功更新 last-build。
- 发布前固定 profile、target、jobs、Cargo/Python 版本。
- 使用 coordinator 规定的目标目录，不在源码树直接生成临时 target。

## 相关页面

- [Hub 总览](../zircon-hub.md)
- [Session Coordinator](../session-coordinator.md)
- [项目创建与历史](projects-history.md)
- [构建 runner 测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/src/build/runner.rs)
