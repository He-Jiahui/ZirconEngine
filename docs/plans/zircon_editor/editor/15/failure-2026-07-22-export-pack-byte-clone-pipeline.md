---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: export-pack-byte-clone-pipeline
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/bin/zircon_export_pack/manifest.rs
  - zircon_runtime/src/bin/zircon_export_pack/run.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/asset/pack/writer/optimization_tests.rs
  - zircon_editor/src/core/export
tests:
  - python -B -m unittest tools.tests.test_editor15_export_pack_borrowed_writer_contract -v
  - cargo test -p zircon_runtime --bin zircon_export_pack --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib pack --locked --jobs 1 -- --nocapture --test-threads=1
---

# Editor15：export pack全量bytes复制与streaming边界交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Runtime bin 40/40逐Rust文件性能审查，PERF-MVP-449
- 修复责任计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 交接原因：Editor15拥有CookAssets→Pack→Report阶段、resume与三入口一致性；最低根因是当前bin writer接收全量owned bytes而非可复用staged chunk artifact。
- 生命周期键：`export-pack-byte-clone-pipeline`

## 失败现象与复现证据

manifest pack input串行读取全部included sources进内存，随后clone完整`Vec<ZrPackInputAsset>`给writer；determinism double-run再次`to_vec`，delta把target bytes复制给reader、复制delta bytes验证并apply到base重建完整pack后逐字节比较。大项目峰值RSS随source+pack+delta多份正文增长。原included path还对assets线性find形成O(A²)，本轮已改first-wins HashMap，但没有触及bytes owner。

2026-07-22底层`asset/pack` 17/17补证：writer仍同时拥有全部input bytes和最终pack Vec；delta apply先物化全部target `ZrPackInputAsset`再由writer复制进rebuilt pack；installer同时整读base+delta+rebuilt，promotion在rename前整读并hash全staged pack，copy fallback又整读installed复验。PERF-MVP-513已把有序lookup切为二分、unique chunk只hash一次且不复制、删除全path/全target row clone，但没有降低整包owner峰值。

## 最低共享层根因

CookAssets输出没有content-addressed immutable chunk/stream contract，Pack工具被迫重新打开、拥有和复制每个asset；determinism与delta也只能按整包bytes复算。Editor15的阶段resume若不保存该artifact，warm export仍无法跳过I/O。

## 架构修复验收

- Cook发布按content hash寻址的staged chunks与manifest，Pack借用/stream读取并复用已有hash；writer in-flight bytes有显式上限。
- zrpack directory/chunk table增量构建，dedup不要求全部payload同时驻留；atomic publish保留。
- determinism先比较canonical manifest/chunk hashes，可选逐chunk复核；不复制全部inputs或生成第二份整包常驻内存。
- delta直接比较base/target chunk tables、复用unchanged chunks并流式写/验证；不通过apply后整包bytes常驻比较。
- reader/installer先读header+manifest并按需映射/stream chunk；初始化每unique chunk hash≤1且不复制，promotion rename成功路径不得整包读入RSS。
- resume的unchanged Cook/Pack source reads=0；1/1k/100k assets与1MiB/1GiB级pack记录clone bytes、peak RSS、queue与wall time，并通过zrpack/delta byte parity和export E2E。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止用更大内存预算、关闭determinism或delta验证掩盖复制。
- 禁止把全部bytes搬到一个后台线程后仍无界驻留；worker queue和chunk buffer必须有界。
- 禁止改变zrpack/delta格式或首项duplicate语义而没有版本/迁移与golden验证。

## 修复结果与回传

### 2026-08-27 借用式 writer 边界切片

- RED：新增 `tools/tests/test_editor15_export_pack_borrowed_writer_contract.py`，初次运行 3/3 失败，分别固定 payload DTO 仍可 `Clone`、writer 仅接受 owned asset、首写与 determinism 复写会复制完整 input vector。
- GREEN：`ZrPackWriter::write` 以单一泛型 `Borrow<ZrPackInputAsset>` 入口同时接受 owned 与 borrowed input；bin 首写和 determinism 复写都传 `pack_assets.iter()`，不再复制 `Vec<ZrPackInputAsset>` 或其中 payload。`ZrPackInputAsset` 与 `ExportPackInputs` 删除 `Clone` derive，把回归转为编译期不可表达。
- lower-layer Rust 回归复用同一组 payload 连续写两次，要求 report byte parity 且输入 payload 地址不变。此前已存在于 worktree 的 unstable sort、容量预分配及其 benchmark 保持不变，仅把源码契约断言更新为泛型借用后的同义排序表达式。
- 静态门：`python -B -m unittest tools.tests.test_editor15_export_pack_borrowed_writer_contract -v` 为 3/3；四个 touched Rust 文件通过 `rustfmt +1.94.1 --edition 2021 --check`；Python test 通过 `py_compile`；scoped diff whitespace gate 通过。
- 受管产品门：执行 `validate-matrix.ps1 -Package zircon_runtime -SkipBuild -Bin zircon_export_pack -VerboseOutput` 后没有创建新的 coordinator Cargo job，未取得产品编译/测试证据。此前同一窗口的只读 artifact audit（request `1a2ef083c416447b8108a56dc620a491`）仍报告外部未受管目录 `D:\\ZirconBuilds\\tooling15-wave143-runtime-20260827-080526`；本切片不清理、不重试该外部 artifact。

Open state: `部分推进，仍待修复`。本切片只消除 writer 边界和 determinism 复写的 input payload 深复制；source loading、最终 pack `Vec<u8>`、有界 chunk streaming、delta/reader/installer 全包 owner、resume 与 1/1k/100k 性能门均未完成，因此不声明 failure fixed。
