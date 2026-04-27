# 开发快编译配置

本目录提供一套可直接用的 Rust 开发快编译脚本：

- `dev-fast-build.ps1`：统一入口（sccache + 共享 target + feature profile）。
- `dev-fast-aliases.ps1`：常用命令别名集合。
- `dev-module-interactive.ps1`：交互选择 target + 模块，并通过环境变量动态加载模块。
- `dev-module-interactive.cmd`：交互脚本的 cmd 包装器。

## 1) 一次性加载别名

```powershell
. .\scripts\dev-fast-aliases.ps1
```

## 2) 直接使用统一脚本

```powershell
# client 最快 check（默认）
.\scripts\dev-fast-build.ps1

# server profile check
.\scripts\dev-fast-build.ps1 -Profile server -Action check

# editor profile build（release）
.\scripts\dev-fast-build.ps1 -Profile editor -Action build -Release
```

## 3) 三套 profile

- `client` -> `target-client`
- `server` -> `target-server`
- `editor` -> `target-editor-host`

脚本默认使用：

- `--no-default-features`
- `--features <profile 对应特性>`
- `--locked`（可用 `-NoLocked` 关闭）

## 4) sccache

```powershell
# 自动安装并启用
.\scripts\dev-fast-build.ps1 -InstallSccache

# 查看缓存统计（需先 dot-source 别名脚本）
zr-sccache-status
```

脚本会在存在 `sccache` 时自动设置：

- `RUSTC_WRAPPER=sccache`

## 5) 共享 target

脚本会自动设置：

- `CARGO_TARGET_DIR=<仓库盘符>\cargo-targets\zircon-shared\<profile>`

例如仓库在 `E:` 时默认是：

- `E:\cargo-targets\zircon-shared\client`
- `E:\cargo-targets\zircon-shared\server`
- `E:\cargo-targets\zircon-shared\editor`

可通过 `-SharedTargetRoot` 自定义根目录。

## 6) 常用别名

- `zr-client-check/build/test/run`
- `zr-server-check/build/test`
- `zr-editor-check/build/test/run`

示例：

```powershell
zr-client-check
zr-server-test -Package zircon_runtime
zr-editor-run
```

## 7) 交互选择模块并动态加载

```powershell
.\scripts\dev-module-interactive.ps1
```

或在 cmd 中：

```cmd
scripts\dev-module-interactive.cmd
```

脚本会：

- 交互选择 `runtime` 或 `editor`。
- 交互选择可选模块（physics/sound/animation/net/navigation/particles/texture/vg/gi）。
- 自动组合编译 feature（避免全量）。
- 自动设置运行时环境变量：
  - `ZIRCON_TARGET_MODE`
  - `ZIRCON_PLUGIN_MANIFEST`

这样 editor/runtime 在启动时会按你选择的模块清单动态加载可用模块。
