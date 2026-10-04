//! 这些块级算法目前只为测试中的历史与数值契约编译；实时混音由 Kira 接管，调用方不能把它们视为已安装的效果器。
#[path = "dynamics.rs"]
mod dynamics;
#[path = "gain.rs"]
mod gain;
#[path = "history.rs"]
mod history;
#[path = "meter.rs"]
mod meter;
#[path = "modulation.rs"]
mod modulation;
#[path = "reverb.rs"]
mod reverb;
#[path = "shaper.rs"]
mod shaper;
#[path = "stereo.rs"]
mod stereo;

pub(crate) use meter::meter_for;

#[cfg(test)]
#[path = "cases.rs"]
mod tests;
