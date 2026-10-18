//! 输入管理器跨事件持有的计时状态，保存截止时间并由调用方显式推进。
//! 这里只计算等待时长、分类候选与取出到期载荷；事件派发在管理器中完成。
//! 时间戳使用单调时钟微秒，延迟入参使用毫秒，换算及截止加法采用饱和运算。
//! 本模块不读取系统时钟；查询、清除和到期取出均以当前保存的状态为准。

use std::{
    collections::{btree_map::Entry, BTreeMap},
    time::Duration,
};

use zircon_runtime_interface::ui::{
    dispatch::{UiInputTimestamp, UiPointerId, UiPointerSource},
    event_ui::UiNodeId,
    surface::UiPointerButton,
};

const MICROS_PER_MILLI: u64 = 1_000;
const TOOLTIP_INTRO_FRAME_INTERVAL_MS: u64 = 16;
/// 双击候选的默认续期窗口，单位为毫秒。
pub const DEFAULT_DOUBLE_CLICK_TIMEOUT_MS: u64 = 500;
/// 组件未配置提示延迟时，供 Runtime 与 Editor 使用的毫秒缺省值。
pub const DEFAULT_TOOLTIP_DELAY_MS: u64 = 150;
/// 提示框进入动画的默认持续时长，单位为毫秒。
pub const DEFAULT_TOOLTIP_INTRO_DURATION_MS: u64 = 100;

/// 保存节点计时项、一个提示框进入动画和一个双击候选。
///
/// # 使用与生命周期
/// `Default` 创建空状态，克隆会复制当前计时项及其拥有的标识文本。
/// 节点键仅为标识，不保活节点；清理失效目标与派发到期事件由消费方负责。
/// 借出的文本受 `self` 的共享借用期限制，取出到期项则转移文本所有权。
///
/// # 约束与参考
/// 调用方应使用同一单调时间域；本类型不校验时间是否倒退。
/// 保存项不会随时间自动消失，普通查询也不筛除已到期项。
/// [`Self::record_tick`] 只记时间；到期清理由显式清除或取出方法完成。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiInputTimerState {
    last_tick: Option<UiInputTimestamp>,
    typeahead_expirations: BTreeMap<UiNodeId, UiInputTimestamp>,
    submenu_hover_expirations: BTreeMap<UiNodeId, UiSubmenuHoverTimerExpiration>,
    tooltip_expirations: BTreeMap<UiNodeId, UiTooltipTimerExpiration>,
    tooltip_intro: Option<UiTooltipIntroTimer>,
    double_click_candidate: Option<UiDoubleClickTimerCandidate>,
    toast_expirations: BTreeMap<UiNodeId, UiToastTimerExpiration>,
}

/// 子菜单节点当前等待的选项标识及截止时间。
#[derive(Clone, Debug, PartialEq, Eq)]
struct UiSubmenuHoverTimerExpiration {
    deadline: UiInputTimestamp,
    option_id: String,
}

/// 提示框节点当前等待的提示标识及截止时间。
#[derive(Clone, Debug, PartialEq, Eq)]
struct UiTooltipTimerExpiration {
    deadline: UiInputTimestamp,
    tooltip_id: String,
}

/// 单个进入动画的采样区间，起点与截止点均使用微秒时间戳。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiTooltipIntroTimer {
    started_at: UiInputTimestamp,
    deadline: UiInputTimestamp,
}

/// 上次释放留下的全局候选，下一次分类须同时匹配目标与指针身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiDoubleClickTimerCandidate {
    deadline: UiInputTimestamp,
    target: UiNodeId,
    pointer_id: Option<UiPointerId>,
    pointer_source: UiPointerSource,
    button: Option<UiPointerButton>,
    click_count: u8,
}

/// 通知节点当前等待自动隐藏的标识及截止时间。
#[derive(Clone, Debug, PartialEq, Eq)]
struct UiToastTimerExpiration {
    deadline: UiInputTimestamp,
    toast_id: String,
}

impl UiInputTimerState {
    pub(crate) fn cancel_interaction_timers(&mut self) {
        self.submenu_hover_expirations.clear();
        self.tooltip_expirations.clear();
        self.tooltip_intro = None;
        self.double_click_candidate = None;
        // Content typeahead and notification lifetimes retain their own expiration.
    }

    /// 返回最近显式记录的时间戳；尚未记录时为 `None`。
    pub fn last_tick(&self) -> Option<UiInputTimestamp> {
        self.last_tick
    }

    /// 查询下一次可见计时工作的相对等待时长。
    ///
    /// # 参数与返回
    /// `now` 为当前单调微秒时间戳；没有相关状态时返回 `None`，
    /// 已到期时返回零时长。提示框进入动画按采样间隔或截止点唤醒。
    ///
    /// # 约束
    /// 只查询键入检索、子菜单、提示框、进入动画和通知；双击候选不触发帧唤醒。
    /// 本方法不取出到期项，也不更新 [`Self::last_tick`]。
    pub fn next_frame_visible_delay(&self, now: UiInputTimestamp) -> Option<Duration> {
        let earliest_deadline_micros = self
            .typeahead_expirations
            .values()
            .map(|deadline| deadline.monotonic_micros)
            .chain(
                self.submenu_hover_expirations
                    .values()
                    .map(|expiration| expiration.deadline.monotonic_micros),
            )
            .chain(
                self.tooltip_expirations
                    .values()
                    .map(|expiration| expiration.deadline.monotonic_micros),
            )
            .chain(
                self.tooltip_intro
                    .map(|intro| intro.next_sample_micros(now)),
            )
            .chain(
                self.toast_expirations
                    .values()
                    .map(|expiration| expiration.deadline.monotonic_micros),
            )
            .min()?;

        Some(Duration::from_micros(
            earliest_deadline_micros.saturating_sub(now.monotonic_micros),
        ))
    }

    /// 记录调用方提供的单调微秒时间戳，覆盖旧值。
    /// 不检查时间先后，也不顺带清除或取出其他计时状态。
    pub fn record_tick(&mut self, now: UiInputTimestamp) {
        self.last_tick = Some(now);
    }

    /// 按 `target` 查询保存的键入检索截止时间；没有该键时返回 `None`。
    pub fn typeahead_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.typeahead_expirations.get(&target).copied()
    }

    /// 按 `target` 查询保存的子菜单悬停截止时间；不判断当前是否已到期。
    pub fn submenu_hover_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.submenu_hover_expirations
            .get(&target)
            .map(|expiration| expiration.deadline)
    }

    /// 借用 `target` 对应的子菜单选项标识；引用有效期不超过本次状态借用。
    pub fn submenu_hover_option_id(&self, target: UiNodeId) -> Option<&str> {
        self.submenu_hover_expirations
            .get(&target)
            .map(|expiration| expiration.option_id.as_str())
    }

    /// 按 `target` 查询保存的提示框截止时间；不判断当前是否已到期。
    pub fn tooltip_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.tooltip_expirations
            .get(&target)
            .map(|expiration| expiration.deadline)
    }

    /// 借用 `target` 对应的提示标识；引用有效期不超过本次状态借用。
    pub fn tooltip_id(&self, target: UiNodeId) -> Option<&str> {
        self.tooltip_expirations
            .get(&target)
            .map(|expiration| expiration.tooltip_id.as_str())
    }

    /// 按 `now` 的单调微秒时间查询进入动画进度，没有活动动画时为 `None`。
    ///
    /// 进度截取到起止区间，返回值在零到一之间；零长度区间按完成处理。
    /// 查询不清除动画，移除时机见 [`Self::expire_tooltip_intro`]。
    pub fn tooltip_intro_progress(&self, now: UiInputTimestamp) -> Option<f32> {
        self.tooltip_intro.map(|intro| intro.progress(now))
    }

    /// 按 `target` 查询保存的通知截止时间；不判断当前是否已到期。
    pub fn toast_expiration(&self, target: UiNodeId) -> Option<UiInputTimestamp> {
        self.toast_expirations
            .get(&target)
            .map(|expiration| expiration.deadline)
    }

    /// 借用 `target` 对应的通知标识；引用有效期不超过本次状态借用。
    pub fn toast_id(&self, target: UiNodeId) -> Option<&str> {
        self.toast_expirations
            .get(&target)
            .map(|expiration| expiration.toast_id.as_str())
    }

    /// 返回仍保存的双击候选截止时间，不根据当前时钟过滤。
    pub fn double_click_expiration(&self) -> Option<UiInputTimestamp> {
        self.double_click_candidate
            .as_ref()
            .map(|candidate| candidate.deadline)
    }

    /// 返回仍保存的双击候选目标，没有候选时为 `None`。
    pub fn double_click_target(&self) -> Option<UiNodeId> {
        self.double_click_candidate
            .as_ref()
            .map(|candidate| candidate.target)
    }

    /// 返回仍保存的双击候选点击次数，没有候选时为 `None`。
    pub fn double_click_count(&self) -> Option<u8> {
        self.double_click_candidate
            .as_ref()
            .map(|candidate| candidate.click_count)
    }

    /// 为 `target` 设置或替换键入检索截止时间。
    ///
    /// # 参数与约束
    /// `started_at` 是单调微秒时间戳，`timeout_ms` 是毫秒时长。
    /// 换算及截止加法均饱和到可表示上限；零时长将截止点设为起点。
    /// 只保存节点键与截止时间，不验证目标是否仍在树中。
    pub fn arm_typeahead_expiration(
        &mut self,
        target: UiNodeId,
        started_at: UiInputTimestamp,
        timeout_ms: u64,
    ) {
        let timeout_micros = timeout_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline = UiInputTimestamp::from_micros(
            started_at.monotonic_micros.saturating_add(timeout_micros),
        );
        self.typeahead_expirations.insert(target, deadline);
    }

    /// 为 `target` 设置或替换子菜单悬停候选，并取得 `option_id` 的所有权。
    ///
    /// `started_at` 为单调微秒时间，`delay_ms` 为毫秒延迟；时间换算及加法饱和。
    /// 标识只作为保存的载荷，不校验选项是否存在；同节点旧候选被覆盖。
    pub fn arm_submenu_hover_expiration(
        &mut self,
        target: UiNodeId,
        option_id: impl Into<String>,
        started_at: UiInputTimestamp,
        delay_ms: u64,
    ) {
        let delay_micros = delay_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline =
            UiInputTimestamp::from_micros(started_at.monotonic_micros.saturating_add(delay_micros));
        self.submenu_hover_expirations.insert(
            target,
            UiSubmenuHoverTimerExpiration {
                deadline,
                option_id: option_id.into(),
            },
        );
    }

    /// 为 `target` 设置或替换提示框候选，并取得 `tooltip_id` 的所有权。
    ///
    /// `started_at` 为单调微秒时间，`delay_ms` 为毫秒延迟；时间换算及加法饱和。
    /// 此处只保存候选；是否显示以及是否启动进入动画由消费方决定。
    pub fn arm_tooltip_expiration(
        &mut self,
        target: UiNodeId,
        tooltip_id: impl Into<String>,
        started_at: UiInputTimestamp,
        delay_ms: u64,
    ) {
        let delay_micros = delay_ms.saturating_mul(MICROS_PER_MILLI);
        let deadline =
            UiInputTimestamp::from_micros(started_at.monotonic_micros.saturating_add(delay_micros));
        self.tooltip_expirations.insert(
            target,
            UiTooltipTimerExpiration {
                deadline,
                tooltip_id: tooltip_id.into(),
            },
        );
    }

    /// 移除 `target` 的子菜单悬停候选；不存在该键时保持原状。
    pub fn clear_submenu_hover_expiration(&mut self, target: UiNodeId) {
        self.submenu_hover_expirations.remove(&target);
    }

    /// 移除 `target` 的提示框候选，不改动独立的进入动画状态。
    pub fn clear_tooltip_expiration(&mut self, target: UiNodeId) {
        self.tooltip_expirations.remove(&target);
    }

    /// 移除全部提示框候选，不改动独立的进入动画状态或其他计时项。
    pub fn clear_tooltip_expirations(&mut self) {
        self.tooltip_expirations.clear();
    }

    /// 从 `started_at` 的单调微秒时间重新开始唯一的提示框进入动画。
    ///
    /// 使用 [`DEFAULT_TOOLTIP_INTRO_DURATION_MS`]，截止加法饱和；覆盖旧动画。
    /// 此方法不检查提示框是否可见，调用方负责选择启动时机。
    pub fn arm_tooltip_intro(&mut self, started_at: UiInputTimestamp) {
        let duration_micros = DEFAULT_TOOLTIP_INTRO_DURATION_MS.saturating_mul(MICROS_PER_MILLI);
        self.tooltip_intro = Some(UiTooltipIntroTimer {
            started_at,
            deadline: UiInputTimestamp::from_micros(
                started_at.monotonic_micros.saturating_add(duration_micros),
            ),
        });
    }

    /// 清除进入动画，后续进度查询将返回 `None`。
    pub fn clear_tooltip_intro(&mut self) {
        self.tooltip_intro = None;
    }

    /// 在保存的动画截止时间小于等于 `now` 时清除它；未到期则保留。
    pub fn expire_tooltip_intro(&mut self, now: UiInputTimestamp) {
        if self
            .tooltip_intro
            .is_some_and(|intro| intro.deadline <= now)
        {
            self.clear_tooltip_intro();
        }
    }

    /// 为一次释放查询点击次数，不修改当前候选。
    ///
    /// # 参数与返回
    /// `target`、`pointer_id`、`pointer_source` 与 `button` 必须同时匹配保存的身份。
    /// `now` 为单调微秒时间，候选截止点等于它时仍可匹配；缺失或不匹配返回一。
    /// 匹配时把已存次数饱和加一，最多为 `u8::MAX`。
    ///
    /// # 时机
    /// 只查询仍保存的候选；若已调用到期清除，同一时刻也会按首次释放处理。
    pub fn double_click_count_for_release(
        &self,
        target: UiNodeId,
        pointer_id: Option<UiPointerId>,
        pointer_source: UiPointerSource,
        button: Option<UiPointerButton>,
        now: UiInputTimestamp,
    ) -> u8 {
        self.double_click_candidate
            .filter(|candidate| {
                candidate.deadline >= now
                    && candidate.target == target
                    && candidate.pointer_id == pointer_id
                    && candidate.pointer_source == pointer_source
                    && candidate.button == button
            })
            .map(|candidate| candidate.click_count.saturating_add(1).max(1))
            .unwrap_or(1)
    }

    /// 用一次释放的目标、指针身份和次数替换唯一的双击候选。
    ///
    /// `started_at` 为单调微秒时间；用 [`DEFAULT_DOUBLE_CLICK_TIMEOUT_MS`] 计算饱和截止点。
    /// `click_count` 至少保存为一；候选仅供后续分类，不纳入可见帧等待查询。
    pub fn arm_double_click_candidate(
        &mut self,
        target: UiNodeId,
        pointer_id: Option<UiPointerId>,
        pointer_source: UiPointerSource,
        button: Option<UiPointerButton>,
        click_count: u8,
        started_at: UiInputTimestamp,
    ) {
        let timeout_micros = DEFAULT_DOUBLE_CLICK_TIMEOUT_MS.saturating_mul(MICROS_PER_MILLI);
        let deadline = UiInputTimestamp::from_micros(
            started_at.monotonic_micros.saturating_add(timeout_micros),
        );
        self.double_click_candidate = Some(UiDoubleClickTimerCandidate {
            deadline,
            target,
            pointer_id,
            pointer_source,
            button,
            click_count: click_count.max(1),
        });
    }

    /// 丢弃当前双击候选，之后的释放查询将按无候选处理。
    pub fn clear_double_click_candidate(&mut self) {
        self.double_click_candidate = None;
    }

    /// 在候选截止时间小于等于 `now` 时清除它；其他情况保持原状。
    pub fn expire_double_click_candidate(&mut self, now: UiInputTimestamp) {
        if self
            .double_click_candidate
            .is_some_and(|candidate| candidate.deadline <= now)
        {
            self.clear_double_click_candidate();
        }
    }

    /// 为 `target` 设置或替换通知截止时间，并取得 `toast_id` 的所有权。
    ///
    /// `started_at` 为单调微秒时间，`timeout_ms` 为毫秒时长；时间换算及加法饱和。
    /// 保存载荷不代表通知仍可见，到期后的表面状态判断交给消费方。
    pub fn arm_toast_expiration(
        &mut self,
        target: UiNodeId,
        toast_id: impl Into<String>,
        started_at: UiInputTimestamp,
        timeout_ms: u64,
    ) {
        let deadline = toast_deadline(started_at, timeout_ms);
        self.toast_expirations.insert(
            target,
            UiToastTimerExpiration {
                deadline,
                toast_id: toast_id.into(),
            },
        );
    }

    /// 从借用标识更新通知候选，供组件队列与表面状态同步使用。
    ///
    /// 时间参数同 [`Self::arm_toast_expiration`]。已有同一标识时保留原字符串存储，
    /// 仍更新截止点；标识不同或没有旧项时才复制 `toast_id`。
    pub(crate) fn arm_toast_expiration_ref(
        &mut self,
        target: UiNodeId,
        toast_id: &str,
        started_at: UiInputTimestamp,
        timeout_ms: u64,
    ) {
        let deadline = toast_deadline(started_at, timeout_ms);
        match self.toast_expirations.entry(target) {
            Entry::Occupied(mut entry) => {
                let expiration = entry.get_mut();
                expiration.deadline = deadline;
                if expiration.toast_id.as_str() != toast_id {
                    expiration.toast_id = toast_id.to_owned();
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(UiToastTimerExpiration {
                    deadline,
                    toast_id: toast_id.to_owned(),
                });
            }
        }
    }

    /// 移除 `target` 的通知候选；不改动表面上的通知内容。
    pub fn clear_toast_expiration(&mut self, target: UiNodeId) {
        self.toast_expirations.remove(&target);
    }

    /// 移除并返回截止时间小于等于 `now` 的键入检索节点。
    ///
    /// # 参数与返回
    /// `now` 为单调微秒时间。返回拥有节点键的新向量，顺序为节点键序，
    /// 不按截止先后排序；未到期项仍留在状态中。
    ///
    /// # 分配与消费
    /// 首次找到到期项时才按当前表长预留输出容量，没有到期项时返回零容量空向量。
    /// 结果只是已移出的状态，事件派发仍由调用方执行。
    pub fn drain_expired_typeahead(&mut self, now: UiInputTimestamp) -> Vec<UiNodeId> {
        let mut expired = Vec::new();
        let capacity = self.typeahead_expirations.len();
        self.typeahead_expirations.retain(|target, deadline| {
            if *deadline <= now {
                if expired.is_empty() {
                    expired.reserve(capacity);
                }
                expired.push(*target);
                false
            } else {
                true
            }
        });
        expired
    }

    /// 移除到期子菜单项并返回节点键及选项标识的所有权。
    /// 到期边界、节点键顺序和懒分配规则同 [`Self::drain_expired_typeahead`]；载荷不克隆。
    pub fn drain_expired_submenu_hover(
        &mut self,
        now: UiInputTimestamp,
    ) -> Vec<(UiNodeId, String)> {
        let mut expired = Vec::new();
        let capacity = self.submenu_hover_expirations.len();
        self.submenu_hover_expirations.retain(|target, expiration| {
            if expiration.deadline <= now {
                if expired.is_empty() {
                    expired.reserve(capacity);
                }
                expired.push((*target, std::mem::take(&mut expiration.option_id)));
                false
            } else {
                true
            }
        });
        expired
    }

    /// 移除到期提示框项并返回节点键及提示标识的所有权。
    /// 到期边界、节点键顺序和懒分配规则同 [`Self::drain_expired_typeahead`]；载荷不克隆。
    pub fn drain_expired_tooltips(&mut self, now: UiInputTimestamp) -> Vec<(UiNodeId, String)> {
        let mut expired = Vec::new();
        let capacity = self.tooltip_expirations.len();
        self.tooltip_expirations.retain(|target, expiration| {
            if expiration.deadline <= now {
                if expired.is_empty() {
                    expired.reserve(capacity);
                }
                expired.push((*target, std::mem::take(&mut expiration.tooltip_id)));
                false
            } else {
                true
            }
        });
        expired
    }

    /// 移除到期通知项并返回节点键及通知标识的所有权。
    /// 到期边界、节点键顺序和懒分配规则同 [`Self::drain_expired_typeahead`]；载荷不克隆。
    pub fn drain_expired_toasts(&mut self, now: UiInputTimestamp) -> Vec<(UiNodeId, String)> {
        let mut expired = Vec::new();
        let capacity = self.toast_expirations.len();
        self.toast_expirations.retain(|target, expiration| {
            if expiration.deadline <= now {
                if expired.is_empty() {
                    expired.reserve(capacity);
                }
                expired.push((*target, std::mem::take(&mut expiration.toast_id)));
                false
            } else {
                true
            }
        });
        expired
    }
}

/// 两种通知设置入口共用毫秒到微秒的饱和截止计算。
fn toast_deadline(started_at: UiInputTimestamp, timeout_ms: u64) -> UiInputTimestamp {
    let timeout_micros = timeout_ms.saturating_mul(MICROS_PER_MILLI);
    UiInputTimestamp::from_micros(started_at.monotonic_micros.saturating_add(timeout_micros))
}

impl UiTooltipIntroTimer {
    /// 下一采样点不晚于动画截止；已到期值由上层换算为零等待。
    fn next_sample_micros(self, now: UiInputTimestamp) -> u64 {
        let frame_interval_micros =
            TOOLTIP_INTRO_FRAME_INTERVAL_MS.saturating_mul(MICROS_PER_MILLI);
        now.monotonic_micros
            .saturating_add(frame_interval_micros)
            .min(self.deadline.monotonic_micros)
    }

    fn progress(self, now: UiInputTimestamp) -> f32 {
        let duration = self
            .deadline
            .monotonic_micros
            .saturating_sub(self.started_at.monotonic_micros);
        // 起点在可表示上限时，饱和截止值可能与起点相同。
        if duration == 0 {
            return 1.0;
        }
        let elapsed = now
            .monotonic_micros
            .saturating_sub(self.started_at.monotonic_micros)
            .min(duration);
        (elapsed as f64 / duration as f64) as f32
    }
}

#[cfg(test)]
#[path = "timers/tests/retain_drain_tests.rs"]
mod retain_drain_tests;
