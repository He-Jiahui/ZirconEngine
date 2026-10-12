# Editor01 · Particle Library 完成列表

日期：2026-10-09。对应[优化记录](../../../optimize/zircon_editor/01/2026-10-09-particle-library-lazy-mount.md)。

## 已完成源码工作

- [x] 公开 VFX 入口沿真实 callback/Controller 按需加载并挂载实际 Particle Library 文档。
- [x] 同一 Surface 的 pane、paint、hit、15 authored local actions 与 child 清退/失败保留合同接入。
- [x] 增加四项实际 Rust 回归源码，迁移一个已有负例并保留 139 项原逐事件断言。
- [x] 独审接受后实际一次接入六文件，386 守卫/六 LF 后像/380 未受影响依赖及 apply/fmt/diff 通过，共享 index/control 未变。
- [x] 保留当前 foreign viewport overflow 与既有焦点 owner，不声称历史资产 geometry 等值。
- [x] 合入同一 15-arm 草案 v6，UI 保守数量下界 136，原 flags/Release producers/性能标准保持。

实际回执 SHA-256：`3d55232d36965b03cdfbe3db541ab3b2d509a5ad7888094aecbfe73d55f0cc0e`；草案 v6：`2fa9d2145ed07044256d52cd5e9ddfa350677a5aaae1e659df3afc50566b20fe`。以上为源码完成项。

## 待实际功能与验收

- [ ] 同批 native 成功执行四新增、一迁移及全部原相关用例，核实际来源、精确名称和非零计数。
- [ ] 真实主窗口呈现、paint/hit/typed input、IME/focus/capture/retirement 与失败原子性。
- [ ] 六 Tools 菜单/43 扩展、Generated 单一真实 owner、21 catalog preview 与 legacy 闭环。
- [ ] 所有原组件 offender-empty 强门，动态消费按真实 physical/source/publication/input 证据证明。
- [ ] 真实 Editor exe + Runtime dll 产品/F0 和原完整 UI CPU/RSS/分配/分位数/像素/帧标准达标。

local preview/journal 准入、fmt 或哈希不能关闭产品与性能验收。
