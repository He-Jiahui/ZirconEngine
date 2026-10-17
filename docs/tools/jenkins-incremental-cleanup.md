# Jenkins 共享增量缓存清理

`zircon-scheduled-cleanup` 保留原来的 `H/30 * * * *` 定时器、固定 Windows 节点、CAS 清理和历史归属巡检。新增缓存阶段先运行，处理此前 CAS inventory 没有登记、因此始终回收 0 字节的共享 Rust 增量缓存。

本阶段只允许以下实际目录：

```text
E:/cargo-targets/zircon-local/win32/codex-default/build/debug/incremental
D:/cargo-targets/zircon-local/win32/2e35cdd9399f5e89889e/build/debug/incremental
D:/cargo-targets/zircon-local/win32/engine-audit-editor102-testbuild-20261007-r4/build/debug/incremental
```

E 盘可用空间少于 80 GiB、D 盘少于 40 GiB 时开始选择，每个命名空间每次最多处理 32 GiB 逻辑大小。第二个 D 盘目录开始前重新检查空间，达到阈值就保留缓存。候选必须是可识别的 rustc 会话目录，至少十分钟未更新，并且能取得该会话 `.lock` 文件的原生非阻塞独占锁。清理期间持续持有锁；活动编译器的共享或独占锁均会阻止删除。年龄只是附加条件。

仅删除选中会话中的已识别 `.bin`、`.o`、`.obj`、`.rmeta`、`.bc` 文件。逐项检查实际文件身份，并通过已打开句柄删除；遇到源码、未知内容、路径别名或变化则保留。硬链接只移除该缓存路径下的名称，其他名称继续保留。源码、Git、依赖下载、依赖构建目录和 96 条待归属记录保持原归属。

文件内容无须重新读取或计算哈希。回执分别记录逻辑删除字节、最后一个链接被移除时的分配字节，以及删除前后的卷剩余空间。并发编译会影响卷空间变化，不能把逻辑大小冒充实际回收量。

实现为 `tools/jenkins-management/incremental_cache_cleanup.py` 和同目录 `incremental-cache-cleanup.groovy`。标准定时任务安装器会包含此阶段；对现有任务也可使用以下公开安装入口，它保留原任务配置和其他阶段：

```powershell
python -B tools/jenkins-management/incremental_cache_cleanup.py install
```

删除命令仅在 Jenkins 的 `zircon-scheduled-cleanup` 与固定节点环境运行；没有本地替代删除入口。`DRY_RUN=true` 只选取和记录。实际运行的清理回执与逐项删除记录位于：

```text
E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/incremental/incremental-<构建号>/
```

两个 D 盘目录的回执分别添加 `-legacy-d-integration`、`-legacy-d-editor` 后缀。三个选择键为固定白名单，命令不接受任意清理路径。

`zircon-management-tests` 的 `checkout-commit-cache-*` 回归覆盖活动锁、硬链接、未知内容保护、空间预算、文件变化、预演和安装保留行为。这里的回归只验证维护工具；业务提交继续使用各自的产品测试证据。
