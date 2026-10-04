// 图像路径分开源解析、圆角遮罩、缓存和命令生成；内容序列只消费最终像素或回退。
mod cache;
mod command;
mod icon;
mod pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use command::push_avatar_image;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use icon::avatar_icon_pixels;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use pixels::avatar_image_pixels;
