use super::visibility_batch_key::VisibilityBatchKey;

/// 指向扁平可见实例数组的批次绘制区间；offset/count 必须与同帧数组共同传递。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibilityDrawCommand {
    pub key: VisibilityBatchKey,
    pub visible_instance_offset: u32,
    pub visible_instance_count: u32,
}
