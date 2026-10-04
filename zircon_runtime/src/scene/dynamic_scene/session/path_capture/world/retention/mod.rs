//! World 路径保留捕获沿用内存预检：预览只投影捕获与裁剪结果，提交成功后才原子写回。global 覆盖全档案；tag 仅裁剪对应标签桶。
mod global;
mod tag;
