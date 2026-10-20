# 开发构建

日常验证优先使用 `dev-module-build.ps1`。默认只检查 `zircon_runtime` 的 `core-min`，
关闭默认 features；通过 `-Package`、`-Features` 和测试目标选择本次改动需要的模块。
测试必须指定 `-LibTests` 或一个 `-TestTarget`；入口不会追加 `--workspace`、
`--all-features` 或 `--all-targets`。请求沿用现有 Jenkins 身份和受管验证入口。

`dev-fast-build.ps1` 保留 `minimal`、`client2d`、`client3d`、`editor`、`dev` 和 `server`
组合，用于相应的集成验证；扩大范围时显式选择配置。

从仓库根目录运行：

```powershell
# 默认最小核心检查
.\tools\dev\dev-module-build.ps1

# 只检查数学模块
.\tools\dev\dev-module-build.ps1 -Package zr_math

# 只构建并运行数学库测试
.\tools\dev\dev-module-build.ps1 -Package zr_math -Action test -LibTests

# 检查确实需要图形和文本的 Runtime 改动
.\tools\dev\dev-module-build.ps1 -Features core-min,graphics,text

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

Jenkins 的共享构建池按仓库、工具链、目标平台、已登记环境和依赖布局确定；
包、features、dev/test 选择保存在请求及 Cargo 指纹中，不再据此复制整份公共依赖缓存。
同一池只允许一个 Jenkins writer，原生 Cargo 锁继续生效。多 worktree 通过稳定的
Jenkins 源码准备目录顺序构建，保持未变源码的时间戳；编译中的输入不得被另一任务替换。
交付产品继续沿用项目现有的独立产品身份与归档流程；共享缓存中的 EXE/PDB 可以被
下一次构建替换，旧回执中的路径不能当作永久交付物，后续任务必须核验实际输入和字节。
旧池和待对账请求原样保留；新目录策略随管理 capsule 更新生效，不搬移活动池。

主工作区、插件工作区和 WOC 独立工作区的日常 dev/test 均采用 `debug=0`、
`incremental=false`，包括 Runtime、Editor 和 App，不再为大模块默认保留增量缓存。
这是磁盘优先的取舍：修改这些模块后的重编译可能变慢，已匹配的 Cargo 依赖产物仍可复用。
完整符号使用显式的 profiling 或 shipping-symbols 诊断/交付配置。普通请求统一
关闭增量和 dev/test 符号；特殊诊断需另交需求、空间预算与保留条件，不自动启用。
不得通过环境变量、包级 profile 覆盖或新 target 目录静默恢复高占用默认值。

每次验证只选受影响的 package、必要 features 和目标，关闭默认 features；禁止全量构建。
复用兼容的 Jenkins 构建池，不为每个会话复制一套 target。关闭增量不会自动删除旧缓存；
旧缓存仍须由 Jenkins 核对归属、活动进程和实际回执后清理。最终交付包、符号、
共享编译产物及增量缓存分别计量，不把配置改变宣称为已释放磁盘。


复用静态库需要 Cargo 指纹匹配工具链、features、profile、编译参数及依赖。
PDB 必须与对应二进制一起保留，不能跨不匹配的构建手动拼接。
最小引擎最终交付包、符号包、共享静态库和增量缓存分别计量；现有 Windows runtime
shipping 产品配置的交付包上限是 200,000,000 字节，构建缓存另行统计。
