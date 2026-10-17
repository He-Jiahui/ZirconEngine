//! 共享一次 canonical 写入的输出计费、资源上限与有界 sink 请求。
//! 暂存内容复制时沿用原计费；暂存工作量另行累计，出错后的状态不作为可恢复事务。

use std::io::Write;
use std::rc::Rc;

use super::super::super::write_error::CanonicalTextWriteError;
use super::super::canonical_spool::{
    canonical_spool_resource_limit, SpoolAttempt, MAX_CANONICAL_SPOOL_WORK_BYTES,
};

/// 单次交给 sink 的最大切片及磁盘暂存读取缓冲大小，单位为字节；不限制总输出。
pub(in crate::serialization::text) const COPY_BUFFER_BYTES: usize = 64 * 1024;
/// 子值格式深度的包含式上限，根深度为 0；由容器进入子值时校验。
pub(in crate::serialization) const MAX_CANONICAL_NESTING_DEPTH: usize = 128;
/// 每个对象允许保留的不同规范键数量上限；重复键替换不新增条目。
pub(in crate::serialization) const MAX_CANONICAL_OBJECT_ENTRIES: usize = 16_384;

/// 一次顶层写入共享的计费状态，不等于已成功落到外部 sink 的字节数。
/// 键及暂存值会提前预留；只有整次写入成功时，输出计费才对应含换行的完整输出长度。
///
/// 暂存工作量独立累计，重复键退款和暂存复制均不把已完成工作从该累计量中扣除。
pub(super) struct OutputBudget {
    /// 已接受的最终输出字节计费，包含尚在暂存中的值和预留键；失败后不承诺等于 sink 长度。
    pub(super) bytes: usize,
    /// 此次输出上限；来自固定文档上限或调用者显式提供的字节预算。
    max_bytes: usize,
    /// 本次写向暂存的累计字节量，包含向外层暂存再次复制的内容。
    spool_work_bytes: usize,
    /// 与本次各值暂存共享的尝试所有权；最后持有者析构时尝试清理所属目录。
    spool_attempt: Rc<SpoolAttempt>,
}

impl OutputBudget {
    /// 为一个顶层调用初始化零计费和独立尝试；此时不创建暂存目录或文件。
    pub(super) fn new(max_bytes: usize) -> Self {
        Self {
            bytes: 0,
            max_bytes,
            spool_work_bytes: 0,
            spool_attempt: Rc::new(SpoolAttempt::new()),
        }
    }

    /// 在写入前预留 `additional` 个最终输出字节。
    ///
    /// 成功才推进计费；相加溢出以 `found = usize::MAX` 报告，超上限返回 `OutputTooLarge`。
    /// 此方法不访问 sink，后续 I/O 失败也不自动退回本次预留。
    pub(super) fn reserve(&mut self, additional: usize) -> Result<(), CanonicalTextWriteError> {
        let found =
            self.bytes
                .checked_add(additional)
                .ok_or(CanonicalTextWriteError::OutputTooLarge {
                    max: self.max_bytes,
                    found: usize::MAX,
                })?;
        if found > self.max_bytes {
            return Err(CanonicalTextWriteError::OutputTooLarge {
                max: self.max_bytes,
                found,
            });
        }
        self.bytes = found;
        Ok(())
    }

    /// 退回被替换旧值的输出计费并饱和到 0；不退还累计暂存工作量或已分配文件数。
    pub(super) fn release(&mut self, bytes: usize) {
        self.bytes = self.bytes.saturating_sub(bytes);
    }

    /// 返回本次最终输出上限，供键转换器检查转义后的 JSON 键长度。
    pub(super) fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// 累计本次写向暂存的 `additional` 字节；溢出按最大值检测，失败不推进该累计量。
    /// 预算独立于最终输出，嵌套复制和后来被替换的值所做工作仍占用此额度。
    pub(super) fn reserve_spool_work(
        &mut self,
        additional: usize,
    ) -> Result<(), CanonicalTextWriteError> {
        let found = self
            .spool_work_bytes
            .checked_add(additional)
            .unwrap_or(usize::MAX);
        ensure_resource_limit(
            "canonical spool work bytes",
            found,
            MAX_CANONICAL_SPOOL_WORK_BYTES,
        )?;
        self.spool_work_bytes = found;
        Ok(())
    }

    /// 只检查目标子值深度，超过包含式上限才拒绝，不修改当前输出计费。
    pub(super) fn ensure_nesting_depth(&self, found: usize) -> Result<(), CanonicalTextWriteError> {
        ensure_resource_limit(
            "canonical nesting depth",
            found,
            MAX_CANONICAL_NESTING_DEPTH,
        )
    }

    /// 检查当前对象准备保留的不同键数量，不把其他对象的字段累加到此值。
    pub(super) fn ensure_object_entries(
        &self,
        found: usize,
    ) -> Result<(), CanonicalTextWriteError> {
        ensure_resource_limit(
            "canonical object entries",
            found,
            MAX_CANONICAL_OBJECT_ENTRIES,
        )
    }

    /// 克隆本次尝试的共享所有权，供新建暂存使用；不开始新的独立尝试。
    pub(super) fn spool_attempt(&self) -> Rc<SpoolAttempt> {
        Rc::clone(&self.spool_attempt)
    }
}

/// 同时独占借用 sink 和共享计费状态的输出器；不拥有或关闭调用方的 sink。
/// 暂存模式只额外记录工作量，嵌套输出器仍使用同一份预算。
pub(in crate::serialization::text) struct CountingWriter<'sink, 'budget, W: Write + ?Sized> {
    /// 当前接收端的独占借用，可为最终接收端，也可为暂存。
    sink: &'sink mut W,
    /// 本次写入预算的独占重借用；嵌套层完成后交还父层。
    pub(super) budget: &'budget mut OutputBudget,
    /// 是否把写入本 sink 的每批字节另计入暂存工作量。
    counts_spool_work: bool,
}

impl<'sink, 'budget, W: Write + ?Sized> CountingWriter<'sink, 'budget, W> {
    /// 借用最终 sink 与已有预算，不记录暂存工作量，也不分配缓冲或刷新。
    pub(super) fn new(sink: &'sink mut W, budget: &'budget mut OutputBudget) -> Self {
        Self {
            sink,
            budget,
            counts_spool_work: false,
        }
    }

    /// 借用暂存 sink 与同次预算，使写入及预计费复制都累计暂存工作量。
    pub(super) fn new_spool(sink: &'sink mut W, budget: &'budget mut OutputBudget) -> Self {
        Self {
            sink,
            budget,
            counts_spool_work: true,
        }
    }

    /// 先预留输出字节，再按模式预留暂存工作量，最后有界写入。
    /// 后一步失败不回退前一步计费；I/O 错误可能已写入部分字节。
    pub(super) fn write_counted(&mut self, bytes: &[u8]) -> Result<(), CanonicalTextWriteError> {
        self.budget.reserve(bytes.len())?;
        if self.counts_spool_work {
            self.budget.reserve_spool_work(bytes.len())?;
        }
        self.write_all_bounded(bytes)
    }

    /// 写入已在键准备或值序列化阶段计费的内容，不再次增加最终输出预算。
    /// 调用方须保证内容已计费；复制到另一暂存时仍累计工作量，失败不回滚。
    pub(in crate::serialization::text) fn write_preaccounted(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), CanonicalTextWriteError> {
        if self.counts_spool_work {
            self.budget.reserve_spool_work(bytes.len())?;
        }
        self.write_all_bounded(bytes)
    }

    /// 按固定大小切片调用 sink 的完整写入协议，允许底层短写并保留底层错误。
    /// 暂存文件数错误恢复为资源错误，其余写失败归入 I/O；此处不另外计费。
    fn write_all_bounded(&mut self, bytes: &[u8]) -> Result<(), CanonicalTextWriteError> {
        for chunk in bytes.chunks(COPY_BUFFER_BYTES) {
            self.sink.write_all(chunk).map_err(|source| {
                if let Some((resource, max, found)) = canonical_spool_resource_limit(&source) {
                    CanonicalTextWriteError::ResourceLimitExceeded {
                        resource,
                        max,
                        found,
                    }
                } else {
                    CanonicalTextWriteError::Io {
                        operation: "write canonical text",
                        source,
                    }
                }
            })?;
        }
        Ok(())
    }

    /// 请求刷新当前 sink 并保留 I/O 错误；成功不等价于文件同步、持久化或原子发布。
    pub(super) fn flush(&mut self) -> Result<(), CanonicalTextWriteError> {
        self.sink
            .flush()
            .map_err(|source| CanonicalTextWriteError::Io {
                operation: "flush canonical text",
                source,
            })
    }
}

/// 对 `found` 与 `max` 作包含式上限检查；超出时携带资源名、上限和观测值返回类型化错误。
fn ensure_resource_limit(
    resource: &'static str,
    found: usize,
    max: usize,
) -> Result<(), CanonicalTextWriteError> {
    if found > max {
        return Err(CanonicalTextWriteError::ResourceLimitExceeded {
            resource,
            max,
            found,
        });
    }
    Ok(())
}
