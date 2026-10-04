#[cfg(test)]
use std::ops::Index;

const MAX_CACHEABLE_PHASE_COUNT: usize = 3;

/// 单个静态网格最多缓存预深度、阴影、主颜色三个 pass 的命令。
/// 若增加可缓存 pass，须同步扩容这个固定槽位。
pub(super) struct PendingMeshCommandSlots<T> {
    items: [Option<T>; MAX_CACHEABLE_PHASE_COUNT],
    len: usize,
}

impl<T> Default for PendingMeshCommandSlots<T> {
    fn default() -> Self {
        Self {
            items: [None, None, None],
            len: 0,
        }
    }
}

impl<T> PendingMeshCommandSlots<T> {
    pub(super) fn push(&mut self, item: T) {
        assert!(
            self.len < MAX_CACHEABLE_PHASE_COUNT,
            "pending mesh command cache phase capacity exceeded"
        );
        self.items[self.len] = Some(item);
        self.len += 1;
    }

    pub(super) const fn len(&self) -> usize {
        self.len
    }

    #[cfg(test)]
    pub(super) const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
impl<T> Index<usize> for PendingMeshCommandSlots<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.items[index]
            .as_ref()
            .expect("pending mesh command slot must be initialized below its length")
    }
}

impl<T> IntoIterator for PendingMeshCommandSlots<T> {
    type Item = T;
    type IntoIter = std::iter::Flatten<std::array::IntoIter<Option<T>, MAX_CACHEABLE_PHASE_COUNT>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter().flatten()
    }
}
