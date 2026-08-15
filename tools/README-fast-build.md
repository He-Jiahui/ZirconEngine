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

脚本会通过 Windows 物理路径解析确认根目录位于 D:\cargo-targets、E:\cargo-targets 或
F:\cargo-targets 之一。每次调用还会将下列可写目录收敛到该共享根目录，并在结束时恢复原调用
进程的环境变量：

- CARGO_HOME=shared-root\cargo-home
- SCCACHE_DIR=shared-root\sccache
- TEMP、TMP、TMPDIR=shared-root\profile\temporary

使用 -InstallSccache 时，二进制安装在 shared-root\cargo-home\bin。
使用非默认 shared-root 时，可用 `zr-sccache-status -SharedTargetRoot <shared-root>`
读取同一受管缓存的统计信息。

`check-runtime-domain-features.ps1` 和 `check-runtime-profile-features.ps1` 也使用相同的物理路径
解析和环境隔离。未指定 `-TargetDir` 时，它们分别使用仓库盘符下的
`cargo-targets\zircon-runtime-domain-matrix` 与
`cargo-targets\zircon-runtime-profile-matrix`；显式目标同样必须在 D:、E: 或 F: 的
`cargo-targets` 下。两个脚本会把 Cargo home、sccache 和临时文件放在该目标中，并在完成后恢复
调用方环境。

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
