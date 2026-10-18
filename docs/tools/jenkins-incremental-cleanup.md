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

## 共享依赖缓存的物理空间回收

增量目录清空后，`build/debug/deps` 内的完整依赖、元数据和调试符号仍可能占用数十 GiB。增量缓存或 CAS 阶段回收 0 B 不能证明它们已经处理。

`cargo_cache_compaction.py` 为现有定时 job 提供独立维护阶段。当前精确范围为 `E:/cargo-targets/zircon-local/win32/codex-default/build/debug/deps`；E 盘不足 80 GiB 时，对已确认的 Cargo `.rlib/.rmeta/.pdb/.o` 执行 [NTFS 原生文件压缩](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_set_compression)。文件保留原位，可继续复用；不删除源码、依赖、调试符号或构建输出，也不设置目录的压缩继承属性。

Jenkins 核验当前原生 owner 和不可变驱动后，取得既有 `.cargo-lock` 的非阻塞独占锁。每项使用禁止并发写入和删除的原始文件句柄，核对 SHA256、文件身份、长度和最后写入时间，压缩后再次验证；新文件、未知命名、硬链接、加密、稀疏、别名、活动写入和已压缩项均保留。当前策略单次最多 512 项、32 GiB 逻辑输入、15 分钟；在下一项开始前检查预算，达到卷剩余 80 GiB 即停止。未处理项由后续定时流程接续，不靠杀死正在处理文件的进程控制预算。

回执位于 `E:/cargo-targets/zircon-local/reports/jenkins-scheduled-cleanup/compaction/compact-<构建号>/`。原生 `FileStandardInfo.AllocationSize` 前后差额记录为物理分配回收量；逻辑文件大小和卷余量变化分别记录。首次 #300 使用 `FileCompressionInfo`，对未压缩输入只能作为物理回收下界，不能冒充精确分配计量。每项意图、完成、原生进程终止与 EOF 证据保留。未确认意图不能计为成功，也不能以整盘清理完成描述单个范围。构建列表单独显示 `NTFS physical`，不再用 CAS 的 0 B 覆盖实际压缩回收量。

专用 `install` 入口固定代码与策略哈希，保留原有增量、CAS、治理阶段及 `H/30` 串行触发器；标准 PowerShell 重装入口也保留已安装的压缩阶段。固定的修复工作树和策略文件在迁移、重新核验与安装前不得删除。当前策略只处理 E 盘该范围；D/F 的活动构建、未知归属和未完成工作树继续独立保留。
