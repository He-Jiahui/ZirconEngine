use super::{source_cubemap_mips_from_base, source_cubemap_mips_from_base_with_parallel_executor};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Real;

#[derive(Default)]
struct CountingParallelSliceExecutor(std::sync::atomic::AtomicUsize);

impl ParallelSliceExecutor for CountingParallelSliceExecutor {
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        for chunk in items.chunks_mut(chunk_size.max(1)) {
            task(chunk);
        }
    }
}

#[test]
fn parallel_angular_mipmap_writes_final_storage_and_matches_serial_output() {
    let face_size = 128;
    let mip_count = super::super::source_cubemap_mip_count(face_size);
    let base_texels = (0..super::super::source_cubemap_sample_count(face_size, mip_count))
        .map(|index| {
            let value = (index % 17) as Real / 16.0;
            [value, value * 0.5, 1.0 - value, 1.0]
        })
        .collect::<Vec<_>>();
    let serial = source_cubemap_mips_from_base(&base_texels, face_size, mip_count);

    let executor = CountingParallelSliceExecutor::default();
    let parallel = source_cubemap_mips_from_base_with_parallel_executor(
        &base_texels,
        face_size,
        mip_count,
        &executor,
    );

    assert_eq!(parallel, serial);
    assert!(
        executor.0.load(std::sync::atomic::Ordering::Relaxed) > 0,
        "a 128-face input must retain the angular face-parallel schedule"
    );
}
