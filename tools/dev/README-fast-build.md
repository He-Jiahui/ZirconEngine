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

主工作区、插件工作区和 WOC 独立工作区的日常 dev/test 均采用 `debug=0`、
`incremental=false`，包括 Runtime、Editor 和 App，不再为大模块默认保留增量缓存。
这是磁盘优先的取舍：修改这些模块后的重编译可能变慢，已匹配的 Cargo 依赖产物仍可复用。
完整符号使用显式的 profiling 或 shipping-symbols 诊断/交付配置；如需增量诊断，
须在 Jenkins 请求中登记 `CARGO_INCREMENTAL=1` 的原因、输出归属与保留期限。
不得通过环境变量、包级 profile 覆盖或新 target 目录静默恢复高占用默认值。

每次验证只选受影响的 package、必要 features 和目标，关闭默认 features；禁止全量构建。
复用兼容的 Jenkins 构建池，不为每个会话复制一套 target。关闭增量不会自动删除旧缓存；
旧缓存仍须由 Jenkins 核对归属、活动进程和实际回执后清理。最终交付包、符号、
共享编译产物及增量缓存分别计量，不把配置改变宣称为已释放磁盘。
