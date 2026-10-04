# 开发构建

统一入口是 `dev-fast-build.ps1`，支持 `minimal`、`client2d`、`client3d`、`editor`、`dev` 和 `server` 配置。
它将参数交给仓库验证入口；Jenkins 唯一入口模式启用时，需要相应的请求身份。

从仓库根目录运行：

```powershell
# 默认 client3d 检查
.\tools\dev\dev-fast-build.ps1

# Server 检查
.\tools\dev\dev-fast-build.ps1 -Profile server -Action check

# Editor release 构建
.\tools\dev\dev-fast-build.ps1 -Profile editor -Action build -Release

# 加载 zr-client-*、zr-server-*、zr-editor-* 别名
. .\tools\dev\dev-fast-aliases.ps1

# 交互选择 Runtime / Editor 和插件
.\tools\dev\dev-module-interactive.ps1
```

CMD 入口和常用检查、构建快捷命令也放在本目录。
`-InstallSccache` 要求系统已安装 sccache；脚本不会自动安装。

构建目标和编译缓存必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下。
详细参数和产物约定见[构建工具说明](../../docs/tooling/zircon-build-tool.md)。
