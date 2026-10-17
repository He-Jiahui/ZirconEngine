//! 为快照抽取和多个 Surface 的合并过程提供共享预算；计时从实例创建开始。
//! 借用值的 JSON 只写入计数器，不保存编码缓冲；分步观察之后仍校验完整输出。
//! 计数器跨序列化写入保存字符串/转义状态，层级只累计字符串之外的容器。

use std::io::{self, Write};
use web_time::{Duration, Instant};

use serde::Serialize;
use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

/// 快照预算检查的失败原因；JSON 分支保留序列化器返回的错误文本。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AccessibilitySnapshotBudgetError {
    /// 累计 JSON 编码字节数超过上限，两个字段均以字节为单位。
    EncodedBytes { observed: usize, limit: usize },
    /// 调用方观察的保留项目数超过上限。
    Items { observed: usize, limit: usize },
    /// 外层偏移与当前 JSON 容器深度之和超过上限。
    NestingDepth { observed: usize, limit: usize },
    /// 自预算创建起的耗时超限；上限字段以微秒为单位。
    ProcessingTime { limit_micros: u64 },
    /// 序列化失败的拥有型错误文本。
    Json(String),
}

/// 为外部快照构建累计项目、编码字节、嵌套层级及时间预算；观察值后仍须验证最终完整载荷。
/// unbounded 仅供内部即时查询，调用外部 payload 边界须使用带限制的实例。
///
/// # 生命周期
/// 观察调用共享累计计数，失败后不回滚此前写入的计数。实例不保留被测值的引用，
/// 也不保存 JSON 缓冲；调用方可让一个实例覆盖同一外部输出中的多个局部快照。
pub(crate) struct AccessibilityBuildBudget {
    limit: Option<ZrRuntimePayloadLimitV1>,
    observed_items: usize,
    observed_encoded_bytes: usize,
    started: Instant,
}

impl AccessibilityBuildBudget {
    /// 使用调用方选定的限制，从零计数并开始计时。
    pub(crate) fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self {
            limit: Some(limit),
            observed_items: 0,
            observed_encoded_bytes: 0,
            started: Instant::now(),
        }
    }

    /// 创建跳过 JSON 测量与预算拒绝的内部查询实例。
    pub(super) fn unbounded() -> Self {
        Self {
            limit: None,
            observed_items: 0,
            observed_encoded_bytes: 0,
            started: Instant::now(),
        }
    }

    /// 将本次保留的节点、动作、关系项或诊断数累加到项目预算。
    /// 超限时累计值已经更新；无上限实例直接成功。
    pub(super) fn observe_items(
        &mut self,
        count: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        let Some(limit) = self.limit else {
            return Ok(());
        };
        self.observed_items = self.observed_items.saturating_add(count);
        if self.observed_items > limit.max_items {
            return Err(AccessibilitySnapshotBudgetError::Items {
                observed: self.observed_items,
                limit: limit.max_items,
            });
        }
        self.check_deadline()
    }

    /// 在保留值前，以当前累计字节为基数测量其 JSON，再计入成功的编码字节数。
    ///
    /// # 参数
    /// `value` 仅在调用内借用；`nesting_offset` 是该值外层已存在的 JSON 容器数。
    ///
    /// # 返回
    /// 字节、层级、耗时越界或序列化失败时返回错误；失败测量不提交本次字节增量。
    pub(crate) fn observe_value(
        &mut self,
        value: &(impl Serialize + ?Sized),
        nesting_offset: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        let encoded_bytes =
            self.measure_serialized(value, nesting_offset, self.observed_encoded_bytes)?;
        self.observed_encoded_bytes = self.observed_encoded_bytes.saturating_add(encoded_bytes);
        Ok(())
    }

    /// 以当前累计量预检借用值而不累计字节，供复制名称等文本前检查大小与层级。
    /// `nesting_offset` 与观察值时含义相同；实际保留后仍需观察相应输出。
    pub(crate) fn preflight_value(
        &self,
        value: &(impl Serialize + ?Sized),
        nesting_offset: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        self.measure_serialized(value, nesting_offset, self.observed_encoded_bytes)
            .map(|_| ())
    }

    /// 按旧值抵扣后的基数测量新值，两次测量均成功后才更新累计字节。
    ///
    /// # 参数与约束
    /// `previous` 和 `replacement` 表示同一已观察字段的替换前后值，仅在调用内借用。
    /// `nesting_offset` 表示字段外层容器数。本函数只调整计数，字段写入由调用方完成。
    pub(crate) fn observe_replacement<T: Serialize + ?Sized>(
        &mut self,
        previous: &T,
        replacement: &T,
        nesting_offset: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        let previous_bytes = self.measure_serialized(previous, nesting_offset, 0)?;
        let retained_without_previous = self.observed_encoded_bytes.saturating_sub(previous_bytes);
        let replacement_bytes =
            self.measure_serialized(replacement, nesting_offset, retained_without_previous)?;
        self.observed_encoded_bytes = retained_without_previous.saturating_add(replacement_bytes);
        Ok(())
    }

    /// 从零字节、零层级测量完整输出，核对最终 JSON 形状；不重置累计计数。
    /// 项目数仍由此前的观察调用累计，完整测量检查字节、层级、耗时及序列化错误。
    pub(crate) fn validate_payload(
        &self,
        value: &(impl Serialize + ?Sized),
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        self.measure_serialized(value, 0, 0).map(|_| ())
    }

    /// 按实例创建时刻检查累计耗时，限制字段使用微秒。
    pub(super) fn check_deadline(&self) -> Result<(), AccessibilitySnapshotBudgetError> {
        let Some(limit) = self.limit else {
            return Ok(());
        };
        if self.started.elapsed() > Duration::from_micros(limit.max_processing_time_micros) {
            return Err(AccessibilitySnapshotBudgetError::ProcessingTime {
                limit_micros: limit.max_processing_time_micros,
            });
        }
        Ok(())
    }

    fn measure_serialized(
        &self,
        value: &(impl Serialize + ?Sized),
        nesting_offset: usize,
        base_count: usize,
    ) -> Result<usize, AccessibilitySnapshotBudgetError> {
        let Some(limit) = self.limit else {
            return Ok(0);
        };
        self.check_deadline()?;
        if nesting_offset > limit.max_nesting_depth {
            return Err(AccessibilitySnapshotBudgetError::NestingDepth {
                observed: nesting_offset,
                limit: limit.max_nesting_depth,
            });
        }
        let mut writer =
            AccessibilityCountingWriter::new(limit, self.started, nesting_offset, base_count);
        let result = serde_json::to_writer(&mut writer, value);
        writer.finish(result)
    }
}

/// 序列化期间的无存储字节接收器；保存容器深度、字符串状态和预算失败原因。
/// 每次测量独立创建并在完成时消费，`base_count` 只参与总字节上限比较。
struct AccessibilityCountingWriter {
    limit: ZrRuntimePayloadLimitV1,
    started: Instant,
    base_count: usize,
    count: usize,
    depth: usize,
    in_string: bool,
    escaped: bool,
    failure: Option<AccessibilitySnapshotBudgetError>,
}

impl AccessibilityCountingWriter {
    fn new(
        limit: ZrRuntimePayloadLimitV1,
        started: Instant,
        nesting_offset: usize,
        base_count: usize,
    ) -> Self {
        Self {
            limit,
            started,
            base_count,
            count: 0,
            depth: nesting_offset,
            in_string: false,
            escaped: false,
            failure: None,
        }
    }

    /// 优先返回计数器保存的预算错误，再处理序列化错误与末次耗时检查。
    fn finish(
        mut self,
        result: Result<(), serde_json::Error>,
    ) -> Result<usize, AccessibilitySnapshotBudgetError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| AccessibilitySnapshotBudgetError::Json(error.to_string()))?;
        if self.started.elapsed() > Duration::from_micros(self.limit.max_processing_time_micros) {
            return Err(AccessibilitySnapshotBudgetError::ProcessingTime {
                limit_micros: self.limit.max_processing_time_micros,
            });
        }
        Ok(self.count)
    }
}

impl Write for AccessibilityCountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.started.elapsed() > Duration::from_micros(self.limit.max_processing_time_micros) {
            self.failure = Some(AccessibilitySnapshotBudgetError::ProcessingTime {
                limit_micros: self.limit.max_processing_time_micros,
            });
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "accessibility snapshot build deadline exceeded",
            ));
        }
        self.count = self.count.saturating_add(bytes.len());
        let observed = self.base_count.saturating_add(self.count);
        if observed > self.limit.max_encoded_bytes {
            self.failure = Some(AccessibilitySnapshotBudgetError::EncodedBytes {
                observed,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other(
                "accessibility snapshot byte budget exceeded",
            ));
        }
        // JSON 可分多次写入；跨块保留引号与转义状态，避免把字符串内的括号计作容器。
        for byte in bytes {
            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.in_string = false;
                }
                continue;
            }
            match *byte {
                b'"' => self.in_string = true,
                b'{' | b'[' => {
                    self.depth = self.depth.saturating_add(1);
                    if self.depth > self.limit.max_nesting_depth {
                        self.failure = Some(AccessibilitySnapshotBudgetError::NestingDepth {
                            observed: self.depth,
                            limit: self.limit.max_nesting_depth,
                        });
                        return Err(io::Error::other(
                            "accessibility snapshot nesting budget exceeded",
                        ));
                    }
                }
                b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/budget.rs"]
mod tests;
