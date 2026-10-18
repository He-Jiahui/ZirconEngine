# Jenkins 增量缓存清理

`zircon-scheduled-cleanup` 保留 `H/30 * * * *` 定时器、固定 Windows 节点、CAS 清理和历史归属巡检。2026-10-18 审查确认：旧阶段只列出三个目录，D 盘 40 GiB 阈值也未触发；大部分实际增量缓存位于其他 D/E/F 盘命名空间，因此多次 SUCCESS 仍回收 0 字节。

精确白名单在 `tools/jenkins-management/incremental_cache_cleanup.py` 的 `CACHE_POLICIES`。本次增加 D 盘 dependency-fix、windows，E 盘 wasm、Miri/跨平台缓存和 F 盘 penpot-workbench 的已审查 `incremental` 子目录。不会把整个产品目录、源码副本、Cargo 依赖下载或任意输入路径变成删除范围。所有目录均物理位于盘符根 `D/E/F:/cargo-targets`。

每个命名空间开始前重新检查卷空间；不足 80 GiB 时选择，每次最多 32 GiB 逻辑大小。候选必须是可识别的 rustc 会话，至少十分钟未更新，并取得对应 `.lock` 的原生非阻塞独占锁。清理期间持续持锁；活动编译器的共享或独占锁均保护该会话。年龄只是附加条件。

只删除选中会话中的已识别 `.bin`、`.o`、`.obj`、`.rmeta`、`.bc` 文件。每项检查实际文件身份，通过原生句柄删除；源码、未知内容、路径别名、变化、活动锁均保留。硬链接只移除缓存下的名称。保留主程序、deps、下载缓存、Git、工作树和历史回执；工作树仍需其独立的归属、交付和原操作终止证据。

安装器将所选实现、辅助模块、专用 Python 和运行代次核验入口的 SHA256 固定到 job。源码可来自已提交的 Jenkins 修复工作树；该路径在迁移到 main 的实现前必须保留。阶段先验证真实 controller/agent/host 身份与当前驱动代次，再运行有界清理。变更已固定的实现必须重新审查并安装，不会静默执行改动。

```powershell
python -B tools/jenkins-management/incremental_cache_cleanup.py install
```

命令只在 Jenkins 的 `zircon-scheduled-cleanup` 与固定节点中删除；没有本地替代入口。`DRY_RUN=true` 只选择和记录。回执位于 `E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/incremental/incremental-<构建号>[-<选择键>]/`，原生代次核验结果和各目录回执归档到同次构建。

回执分别记录逻辑删除字节、最后一个硬链接移除时的分配字节和卷空间变化，并明确列出空目录、保护会话和范围是否完成。并发写入会影响卷空间变化；逻辑大小不能当作实际回收量。单个范围成功不表示整盘已清理，未覆盖或未知归属继续待处理。

`zircon-management-tests` 的缓存回归覆盖活动锁、硬链接、未知内容、预算、文件变化、预演、D/E/F 范围、空目录、保留会话和安装保留行为。维护工具回归不代表产品验收。

## Standard installation

`install-jenkins-scheduled-cleanup.ps1` calls the cleanup module's read-only `render` command. It verifies the returned entry path, module and template hashes, resolved placeholders and nonempty scope before submitting job XML. Direct incremental-stage updates and full standard installations therefore consume the same renderer. The renderer neither connects to Jenkins nor deletes files; execution still requires the scheduled job and its native runtime binding. Standard reinstallation retains its existing governance reattachment and readback gates.

The retained registered-CAS stage stores protected and unknown digests in one case-insensitive hash set before scanning objects. This preserves its protection and byte/object budgets while avoiding one full protection-list scan per object. The source selector is exercised in Jenkins against a private fixture with 30,001 protected entries, unknown content and bounded selection; it never deletes fixture or production objects.
