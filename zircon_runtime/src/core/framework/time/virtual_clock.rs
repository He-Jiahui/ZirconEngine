use std::time::Duration;

/// Game-time marker. Virtual time may be paused, scaled, and clamped per update.
/// World 游戏时间的可暂停、变速时钟；每次推进时才采样 effective_speed，长帧经 max_delta 截断。
///
/// # 状态与生命周期
///
/// 配置倍率、暂停标志与最近采样的有效倍率分别保存；暂停或改配置不会追改旧样本。
/// 默认上限为 250 毫秒、未暂停，两种倍率均为 `1.0`。本值可独立拷贝，副本不跟随 World 更新。
/// 非零上限和有限非负倍率由内部设置入口维护；World 的策略状态准入与 epoch 更新另行完成。
///
/// # 参考
///
/// [Time](super::Time) 在下一次虚拟推进时采样这些状态，
/// [TimePolicy](super::TimePolicy) 描述可交给所有者验证并应用的配置值。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Virtual {
    /// 变速后单次虚拟增量的非零上限。
    max_delta: Duration,
    /// 下次采样是否强制使用零倍率；独立于配置倍率。
    paused: bool,
    /// 保存的有限非负配置倍率，暂停时也保留；负零有效。
    relative_speed: f64,
    /// 初始为一，此后缓存最近一次推进所采样的倍率。
    effective_speed: f64,
}

impl Default for Virtual {
    fn default() -> Self {
        Self {
            max_delta: Duration::from_millis(250),
            paused: false,
            relative_speed: 1.0,
            effective_speed: 1.0,
        }
    }
}

impl Virtual {
    /// 返回变速后单次虚拟增量的上限副本。
    pub fn max_delta(&self) -> Duration {
        self.max_delta
    }

    /// 保存非零上限；零值会 panic，不修改旧增量、暂停状态或有效倍率缓存。
    pub(crate) fn set_max_delta(&mut self, max_delta: Duration) {
        assert_ne!(max_delta, Duration::ZERO, "max delta must be non-zero");
        self.max_delta = max_delta;
    }

    /// 读取暂停标志；配置倍率为零不会自动设置该标志。
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 置暂停标志，下一次推进采样时使用零倍率；最近有效倍率暂时保持原值。
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// 清除暂停标志；下一次采样恢复配置倍率，不立即更新有效倍率缓存。
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// 返回配置倍率；暂停时也保持该配置，实际最近样本应另读有效倍率。
    pub fn relative_speed_f64(&self) -> f64 {
        self.relative_speed
    }

    /// 保存有限非负的 `speed`，允许正零与负零；非有限或负值会 panic。
    /// 设置不立即更新有效倍率缓存，也不改变独立暂停标志。
    pub(crate) fn set_relative_speed_f64(&mut self, speed: f64) {
        assert!(speed.is_finite(), "relative speed must be finite");
        assert!(
            speed.is_sign_positive() || speed == 0.0,
            "relative speed must be non-negative"
        );
        self.relative_speed = speed;
    }

    /// 返回最近一次推进使用的倍率缓存，初始为 `1.0`；不根据当前暂停标志即时重算。
    pub fn effective_speed_f64(&self) -> f64 {
        self.effective_speed
    }

    /// 在每次虚拟推进前刷新并返回有效倍率：暂停取零，否则取配置值；零增量也需采样。
    pub(crate) fn effective_speed_for_next_delta(&mut self) -> f64 {
        self.effective_speed = if self.paused {
            0.0
        } else {
            self.relative_speed
        };
        self.effective_speed
    }
}
