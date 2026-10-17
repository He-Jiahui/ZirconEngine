//! 为具备文档键的纯文本 viewport 布局保留解析结果。
//!
//! 16 项与 32 MiB 只限制本 owner 的估计驻留，返回解析结果可经克隆独立存活；
//! 外部克隆、字体、artifact 和帧内去重均不计入这个预算。

use std::mem::size_of;

use crate::core::framework::text::TextLayoutError;
use crate::text::{RichTextFormat, SharedTextLayoutSession, TextDocumentKey};

use super::super::rich_text::{parse_source_text_with_provider, UiParsedText};

pub(super) const RETAINED_PLAIN_DOCUMENT_CAPACITY: usize = 16;
pub(super) const RETAINED_PLAIN_DOCUMENT_MAX_BYTES: usize = 32 * 1024 * 1024;

/// owner 的观测快照：条目数/估计字节为当前值，其余为饱和历史计数。
/// clear 只刷新当前大小，不清零历史计数；字节数不是共享资源实际驻留总量。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RetainedPlainTextDocumentCacheReport {
    pub(super) entry_count: usize,
    pub(super) estimated_bytes: usize,
    pub(super) hit_count: u64,
    pub(super) miss_count: u64,
    pub(super) stale_source_alias_count: u64,
    pub(super) source_exact_compare_count: u64,
    pub(super) evicted_count: u64,
    pub(super) oversized_bypass_count: u64,
}

/// 持有键、共享解析结果、估计字节和饱和访问序号；淘汰不撤销外部克隆。
#[derive(Clone, Debug, PartialEq)]
struct RetainedPlainTextDocument {
    key: TextDocumentKey,
    parsed: UiParsedText,
    estimated_bytes: usize,
    last_access: u64,
}

/// 按文档键缩小查找，再以完整 Plain 原文相等确认命中。
///
/// 同键异文先移除旧条目并重新解析，成功时返回 Ok，不把它当身份错误。
/// 单项估计超上限时仍返回解析结果，只绕过驻留；访问序号饱和后可能并列。
/// Bounded parsed-document owner for viewport layouts.
///
/// The document key narrows lookup, while exact Plain source equality remains the correctness
/// authority until the surface owns a pointer-stable source snapshot beside its revision.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct RetainedPlainTextDocumentCache {
    documents: Vec<RetainedPlainTextDocument>,
    estimated_bytes: usize,
    next_access: u64,
    report: RetainedPlainTextDocumentCacheReport,
}

impl Default for RetainedPlainTextDocumentCache {
    fn default() -> Self {
        Self {
            documents: Vec::new(),
            estimated_bytes: 0,
            next_access: 0,
            report: RetainedPlainTextDocumentCacheReport::default(),
        }
    }
}

impl RetainedPlainTextDocumentCache {
    /// 清空本 owner 持有的条目并更新当前大小，保留历史统计和访问序号。
    pub(super) fn clear(&mut self) {
        self.documents.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    #[cfg(test)]
    /// 返回统计值副本，条目数和估计字节以当前驻留条目为准。
    pub(super) fn report(&self) -> RetainedPlainTextDocumentCacheReport {
        let mut report = self.report;
        report.entry_count = self.documents.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    /// 取得纯文本解析结果，并按估计容量决定是否驻留。
    ///
    /// # 参数与返回
    /// `key` 是调用方文档身份/修订，`source` 为本次完整原文，`provider` 借用会话解析器。
    /// 返回可共享持有的解析结果；同键不同原文重新解析，解析失败返回原错误。
    ///
    /// # 约束与生命周期
    /// 16 项及估计字节仅约束此 owner 持有的条目，外部克隆不随淘汰失效。
    pub(super) fn resolve(
        &mut self,
        key: TextDocumentKey,
        source: &str,
        provider: &SharedTextLayoutSession,
    ) -> Result<UiParsedText, TextLayoutError> {
        let access = self.next_access();
        if let Some(index) = self
            .documents
            .iter()
            .position(|document| document.key == key)
        {
            self.report.source_exact_compare_count =
                self.report.source_exact_compare_count.saturating_add(1);
            if self.documents[index].parsed.text() == source {
                self.report.hit_count = self.report.hit_count.saturating_add(1);
                self.documents[index].last_access = access;
                return Ok(self.documents[index].parsed.clone());
            }

            let stale = self.documents.swap_remove(index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(stale.estimated_bytes);
            self.report.stale_source_alias_count =
                self.report.stale_source_alias_count.saturating_add(1);
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let parsed = parse_source_text_with_provider(source, RichTextFormat::Plain, provider)?;
        let estimated_bytes = parsed
            .estimated_bytes()
            .saturating_add(size_of::<RetainedPlainTextDocument>());
        if estimated_bytes > RETAINED_PLAIN_DOCUMENT_MAX_BYTES {
            self.report.oversized_bypass_count =
                self.report.oversized_bypass_count.saturating_add(1);
            self.refresh_report_size();
            return Ok(parsed);
        }

        self.evict_for(estimated_bytes);
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.documents.push(RetainedPlainTextDocument {
            key,
            parsed: parsed.clone(),
            estimated_bytes,
            last_access: access,
        });
        self.refresh_report_size();
        Ok(parsed)
    }

    /// 插入前按最小访问序号淘汰至可准入；序号饱和并列时不承诺严格 LRU。
    fn evict_for(&mut self, incoming_bytes: usize) {
        while self.documents.len() >= RETAINED_PLAIN_DOCUMENT_CAPACITY
            || self.estimated_bytes.saturating_add(incoming_bytes)
                > RETAINED_PLAIN_DOCUMENT_MAX_BYTES
        {
            let Some((oldest_index, _)) = self
                .documents
                .iter()
                .enumerate()
                .min_by_key(|(_, document)| document.last_access)
            else {
                break;
            };
            let removed = self.documents.swap_remove(oldest_index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(removed.estimated_bytes);
            self.report.evicted_count = self.report.evicted_count.saturating_add(1);
        }
    }

    /// 返回饱和递增的访问序号；达到上限后允许后续访问取相同值。
    fn next_access(&mut self) -> u64 {
        self.next_access = self.next_access.saturating_add(1);
        self.next_access
    }

    fn refresh_report_size(&mut self) {
        self.report.entry_count = self.documents.len();
        self.report.estimated_bytes = self.estimated_bytes;
    }
}
