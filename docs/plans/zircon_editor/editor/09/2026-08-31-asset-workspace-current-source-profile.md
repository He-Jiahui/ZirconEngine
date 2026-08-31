# Editor09 asset workspace current-source profile gate

## 当前结论

状态：`current-source reviewed / Unreal-grounded / profiler attempted / dynamic baseline unavailable / production optimization not authorized`。

本记录覆盖当前 `AssetWorkspaceState` 稳定帧投影，不复用历史 Editor09 blob，也不把受管构建失败当作性能数据。当前源码已具备 immutable catalog `Arc`、可见资产 chunk generation、UUID/locator index 和 catalog/resource delta patch；剩余热点候选集中在 `build_snapshot` 每次重建 folder presentation，以及双 surface 通过深克隆派生。由于动态基线尚未取得，本轮只补身份耗尽硬切与剖析夹具，不改变生产缓存算法。

当前精确源码基线：

- `zircon_editor/src/ui/workbench/project/asset_workspace_state.rs`: `4BDBCFBA21BA7B0FB439DEF8ED768CCB85A7D9581FA2930336EBD3477A371A59`
- `zircon_editor/src/ui/workbench/project/asset_workspace_state/performance_tests.rs`: `99CEFBA91FD4C2972344A4FD599FC46CE6157C0C8AA7A22EF537DA11706ACB9A`

## 整体算法重审

### 已经正确的边界

- catalog/resource authority 由共享 immutable generation 承载，workspace 不再复制一份可变 catalog authority。
- visible asset projection 按 `{catalog projection generation, selected folder, search query, kind filter}` 缓存；稳定输入返回同一 `AssetWorkspaceItemGeneration`，changed chunk 可共享。
- catalog/resource delta 有 UUID/locator direct index，可对当前 visible generation 做局部 patch；该方向保留。
- projection generation 现在用 `checked_add`，`u64::MAX` 后终止而不是回绕复用旧 identity。此项是正确性硬切，不计性能收益。

### 仍需用动态数据裁决的结构性热点

`build_snapshot` 即使 visible asset generation 命中，仍执行以下工作：

1. `build_folder_tree` 扫描全部 folder，构建临时 parent map，并逐 sibling group 排序；总量为 `O(F + sum(sort(children)))`，最坏为 `O(F log F)`。
2. visible folder 再扫描全部 folder，并克隆命中行的 locator/name；为 `O(F)`。
3. project/search/selection 字符串重新克隆。
4. `build_asset_surface_snapshots` 先构建 Activity，再通过 `activity.clone()` 派生 Explorer。visible assets 是共享 generation，但 folder tree、visible folders、selection 与多个字符串仍按内容复制；稳定双 surface 仍不是 `O(1)`。

因此当前候选瓶颈不是某个 lowercase 或单次 HashMap 调用，而是 source generation 与 presentation generation 尚未完全分层。只有发布一次 immutable folder/surface presentation generation，稳定帧才能从 `O(F)`/`O(F log F)` 收敛到共享句柄复制；但该结构不得在动态剖析前直接落地。

## Unreal 对照

本轮以 `dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp` 当前源码为主要参照：

- `SetBackendFilter` 先比较 backend filter，只有语义变化才请求 slow full refresh（约 1804-1823）。
- slow backend full refresh 与 quick frontend refresh 是两个显式请求通道（约 2132-2140）。
- `HandleItemDataUpdated` 对 Added/Modified/Moved/Removed 做 item-local update/remove，随后只在过滤结果需要时 refresh list，并记录本批数量、总 available items 与耗时（约 6785-6925）。

Zircon 应收敛到同一整体思路：catalog source generation、compiled query/filter、folder/item presentation、visible paint 分层；delta 走局部更新，只有 source/filter authority 变化才进入完整重建。不能用 UI caller 永久缓存第二份 catalog，也不能在 retained tick 继续重建完整 owned DTO。

## 性能工具与动态证据

### Windows Performance Recorder

尝试在 Windows 当前会话启动 CPU + Power 系统采集，WPR 返回 `0xc5585011`：当前非管理员进程无法启用 system performance profiling policy。没有提升权限，也没有在 C 盘写入 trace；失败后未生成可用于结论的 ETL。

### 受管 release 规模夹具

新增 ignored release profile `stable_asset_workspace_snapshot_scale_profile`，覆盖 `1 / 1,000 / 10,000` assets 与 folders、32 次稳定 Activity snapshot 和 Activity+Explorer 双 surface，输出 `EDITOR09_ASSET_WORKSPACE_STABLE_SNAPSHOT_PROFILE_V1` 标记。调用统一受管命令：

```powershell
.\tools\validate-matrix.ps1 -Package zircon_editor -LibTests -TestFilter stable_asset_workspace_snapshot_scale_profile -IgnoredTests -CargoProfile release -VerboseOutput
```

本次受管 release build 与随后的 focused test 均以 `exit 101` 结束；包装器总墙钟为 `962,286 ms`。失败发生在进入 `zircon_editor` 前：共享 `zircon_runtime` 当前有 34 个编译错误、128 个 warning，包含未解析 `zr_contracts`、dynamic-scene descriptor helper、scene post-process 类型和 render frame payload 等跨 owner 漂移。该 16 分钟仅是两次失败编译的证据通道成本，不是资产 snapshot 耗时。由于测试二进制未形成，不能给出 workload wall-time、CPU、allocation、功耗或规模曲线，也不能声称瓶颈已消失。

## 下一次测量与裁决门

恢复可编译 current closure 后，按同一 source hash 重新运行 release 夹具，并在可启用 WPR 的 Windows 会话采集 CPU sampling、allocation/working set、context switch 与 Power。至少记录：

- `1 / 1,000 / 10,000` 规模下稳定 Activity 与双 surface 的 p50/p95、每次分配 bytes/count；
- catalog full publish、单 asset delta、单 folder move、search/filter change 四种事件的工作计数与耗时；
- retained frame 的 snapshot build 次数、queue age、CPU frame budget 占比；
- 同一场景空闲 60 秒的 package power/CPU residency，并与编辑器空项目基线及参考引擎同规模 Content Browser 经验值使用同机同配置比较。

算法验收目标先按规模定义，不预填虚假毫秒：稳定输入 `O(1)` shared generation；单项 delta `O(delta)` 或 `O(delta log N)`；只有 source/filter 全量变化允许 `O(N log N)` rebuild；Activity/Explorer 不得深克隆 folder presentation。取得 baseline 后，先写热点占比与 trace 证据，再决定是否引入 immutable folder/surface presentation generation。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 |
| --- | --- | --- |
| 2026-08-31 11:02 +08:00 | `review-complete / profiler-attempted / dynamic-baseline-blocked / optimization-held` | 完成 current `AssetWorkspaceState` 整体算法、snapshot consumer 与 Unreal `SAssetView` 分层/增量更新复核；补 1/1K/10K ignored release profile 和 projection generation exhaustion 回归，生产代码仅将 generation 回绕改为显式终止。WPR 因 `0xc5585011` 权限门未生成 ETL；受管 release build/test 均 `exit 101`，包装器 962,286 ms 全部消耗于共享 `zircon_runtime` 34 errors/128 warnings 的失败编译，故没有 workload 性能或功耗收益声明，production folder/surface generation 优化保持未开始。 |
| 2026-08-31 21:42 +08:00 | `current-topology-ownership-restored / terminal-consumer-review-complete / managed-test-queued` | 复核 Runtime `ResourceEventTryRecvError::SequenceExhausted` 新终态在 current `events/runtime.rs` 中被语义化为一次 terminal reconcile、latch exhaustion、停止后续资源流读取，并由 focused 回归覆盖；未使用旧单文件或旧 preview。exact current hashes：`events.rs` `94218AA423F221546DADFD3E6F5B682807EE4242DE36CDFE0C1C26C54043D21A`、`events/runtime.rs` `66B62BCDF3BFBA31F7D8A0ABD097E32CF6F9EEC09B4205F14506AE4E9686449F`、`events/runtime/capacity_tests.rs` `4F246FC4BE333D05411BF246005AD609BA1365E66C9A0528ADFAA5C9F353CA9B`、`asset_workspace_state.rs` `4BDBCFBA21BA7B0FB439DEF8ED768CCB85A7D9581FA2930336EBD3477A371A59`、`asset_workspace_state/performance_tests.rs` `99CEFBA91FD4C2972344A4FD599FC46CE6157C0C8AA7A22EF537DA11706ACB9A`；5-file ownership matrix为`integration_ready`，lease request `1fa195d027084689b6acf7411109fd37`、attribution `fcdc479820df4389a93d432ec89a60cd`，focused ticket `47387AA72D5841F6B71B0F4EF7C59C56`（manifest `6AC0EF828FBC815E8455F624203CACC72D9EA459521D54309D34ED8B14D73C2B`）当前 queued，尚无动态 GREEN。 |
