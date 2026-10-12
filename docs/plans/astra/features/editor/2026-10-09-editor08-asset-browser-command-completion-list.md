# Editor08：Assets 命令 owner 修复完成列表

日期：2026-10-09。完成状态仅覆盖实际源码接入；测试及性能验收保持开放。

## 已完成

- [x] 确认当前 toolbar canonical action 与旧 typed binding 不一致。
- [x] 沿两个现有生产 owner 对齐 registry 路由和原 EditorAssetEvent handler，保留预览刷新、view visibility 与 layout/presentation。
- [x] 保留 Always、原 contexts/capabilities 和其他 view command；另一 AssetSurface key 的旧 typed 路由不变。
- [x] 修正同 key 原单元测试 payload，保留原 case/key/Click path；原 10 cases 加 1 个 context owner case。
- [x] 加强现有 retained 回归的真实 journal、preview refresh、host effects 和 MRU 检查。
- [x] 完成独立核心 source review 与四文件实际守卫接入；apply/fmt/diff 均 exit 0，控制文件和 index 无本任务改动。
- [x] 写入[优化记录](../../../optimize/zircon_editor/08/2026-10-09-asset-browser-command-owner.md)和真实 hash/接入凭据。

## 尚未完成

- [ ] 下一批三包 native compile/typecheck 与上述实际测试执行。
- [ ] 原 asset/open-view/canonical registry/retained routing 相关回归通过，且每个指定 case 实际运行。
- [ ] 按当前 UI caller 源码接入必要剩余修复，保留其他任务改动；旧候选计数不作为当前 coverage 证明。
- [ ] 实际模块按需挂载、菜单 geometry/selection/focus 等功能门槛通过。
- [ ] Editor08/Editor386 原 Release P95 门槛和完整 native 产品性能标准达标。
- [ ] 对匹配 request 的终态凭据、实际 test names/producer markers、raw matrix、source hashes 和物理输出进行回填；独立本地证据不等同 Jenkins/里程碑 acceptance。

实际接入：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-asset-browser-command-owner-v2/actual-canonical-apply.json`，SHA-256 `3d2b651bce4596f6f8388474a39bdaf44dd6f45ae7aa9959223aad0994c03c45`。本次没有启动 native job、轮询编译、修改 tooling 或创建源码副本。
