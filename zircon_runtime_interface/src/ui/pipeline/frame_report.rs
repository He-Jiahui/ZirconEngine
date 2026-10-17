//! 对调用方提供的阶段行作汇总和诊断查询；不采集耗时或验证实际执行。

use serde::{Deserialize, Serialize};

use super::{UiPipelineStage, UiPipelineStageCounters, UiPipelineStageReport};

#[cfg(test)]
#[path = "frame_report/tests/performance_tests.rs"]
mod performance_tests;

/// 一个帧标识下的阶段观测及派生汇总，拥有所有阶段行。
///
/// # 约束与生命周期
/// [`Self::from_stage_reports`] 会重算汇总；直接改动公开字段或由 Serde 读取后不会自动重新计算。
/// 需要基于阶段行更新汇总时，应显式调用 [`Self::recompute_totals`]。
/// 本类型不自动检查阶段唯一性或顺序；借出的阶段引用受报告借用期限制。
///
/// # 参考
/// [`UiPipelineStageReport`]、[`UiPipelineStage::ordered`]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPipelineFrameReport {
    pub frame_index: u64,
    pub stages: Vec<UiPipelineStageReport>,
    pub total_elapsed_micros: u64,
    pub totals: UiPipelineStageCounters,
}

impl UiPipelineFrameReport {
    /// 接收阶段向量并计算汇总，保留调用方传入的顺序与重复项。
    ///
    /// # 参数
    /// - `frame_index`：调用方指定的帧标识，本函数不递增或验证它。
    /// - `stages`：移入报告的阶段行；跳过项也参与耗时和计数累加。
    ///
    /// # 返回与约束
    /// 拥有阶段向量和新汇总的报告。累加采用普通 `u64` 加法，
    /// 不做饱和处理或返回数值错误；输入总和需要处于可表示范围。
    pub fn from_stage_reports(frame_index: u64, stages: Vec<UiPipelineStageReport>) -> Self {
        let mut report = Self {
            frame_index,
            stages,
            ..Self::default()
        };
        report.recompute_totals();
        report
    }

    /// 根据当前阶段行覆盖耗时总和与计数汇总，公开字段变更后由调用方主动调用。
    ///
    /// 所有行都参与普通整数累加；不会补齐、重排或执行阶段，也不改动阶段行。
    pub fn recompute_totals(&mut self) {
        let mut totals = UiPipelineStageCounters::default();
        let mut total_elapsed_micros = 0u64;

        for stage in &self.stages {
            total_elapsed_micros += stage.elapsed_micros;
            totals.add_assign(stage.counters);
        }

        self.total_elapsed_micros = total_elapsed_micros;
        self.totals = totals;
    }

    /// 按阶段标识借用一项报告。
    ///
    /// # 参数与返回
    /// `stage` 为目标标识；未找到时返回 `None`，成功引用随 `self` 的共享借用有效。
    /// 优先检查该阶段的规范下标，再回退为从头扫描；允许稀疏、乱序或归档行。
    /// 若有重复阶段，规范位置命中可优先于更早的重复行；调用方需自行定义去重策略。
    pub fn stage_report(&self, stage: UiPipelineStage) -> Option<&UiPipelineStageReport> {
        if let Some(index) = stage.runtime_index() {
            if let Some(report) = self
                .stages
                .get(index)
                .filter(|report| report.stage == stage)
            {
                return Some(report);
            }
        }
        self.stage_report_linear(stage)
    }

    fn stage_report_linear(&self, stage: UiPipelineStage) -> Option<&UiPipelineStageReport> {
        self.stages.iter().find(|report| report.stage == stage)
    }

    /// 统计未标为跳过的行数，不验证其阶段是否属于当前顺序，也不消除重复行。
    pub fn completed_stage_count(&self) -> usize {
        self.stages.iter().filter(|report| !report.skipped).count()
    }

    /// 按当前规范顺序返回缺失的阶段标识。
    ///
    /// 有任意对应行即视为存在，跳过行也算；归档名称不补足当前阶段。
    /// 返回新向量，不检查重复行、耗时或实际执行情况。
    pub fn missing_required_stages(&self) -> Vec<UiPipelineStage> {
        let mut present = [false; UiPipelineStage::ORDER.len()];
        for report in &self.stages {
            if let Some(index) = report.stage.runtime_index() {
                present[index] = true;
            }
        }
        UiPipelineStage::ordered()
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, stage)| (!present[index]).then_some(stage))
            .collect()
    }

    #[cfg(test)]
    fn missing_required_stages_linear(&self) -> Vec<UiPipelineStage> {
        UiPipelineStage::ordered()
            .iter()
            .copied()
            .filter(|stage| self.stage_report_linear(*stage).is_none())
            .collect()
    }

    /// 核对行数及阶段标识次序是否与当前规范表完全一致。
    /// 此判断不读取跳过标志、耗时或计数，不能证明各阶段实际执行过。
    pub fn is_complete_ordered(&self) -> bool {
        self.stages.len() == UiPipelineStage::ordered().len()
            && self
                .stages
                .iter()
                .map(|report| report.stage)
                .eq(UiPipelineStage::ordered().iter().copied())
    }

    /// 检查汇总字段是否满足重复指针移动的诊断条件。
    ///
    /// # 参数与返回
    /// `expected_moves` 是移动计数下限；还要求模板重载、全量布局及布局节点计数为零。
    /// 本函数只读取现有汇总，不重算它，也不证明 Runtime 走过某条执行路径。
    pub const fn repeated_pointer_move_fast_path_holds(&self, expected_moves: u64) -> bool {
        self.totals.pointer_move_count >= expected_moves
            && self.totals.template_reload_count == 0
            && self.totals.full_layout_count == 0
            && self.totals.layout_node_count == 0
    }
}
