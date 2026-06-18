use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderVariantMissReport {
    pub request_count: usize,
    pub memory_hit_count: usize,
    pub disk_hit_count: usize,
    pub compile_miss_count: usize,
    pub disk_write_count: usize,
    pub disk_error_count: usize,
}

impl ShaderVariantMissReport {
    pub const fn with_request(mut self) -> Self {
        self.request_count += 1;
        self
    }

    pub const fn with_memory_hit(mut self) -> Self {
        self.request_count += 1;
        self.memory_hit_count += 1;
        self
    }

    pub const fn with_disk_hit(mut self) -> Self {
        self.disk_hit_count += 1;
        self
    }

    pub const fn with_compile_miss(mut self) -> Self {
        self.compile_miss_count += 1;
        self
    }

    pub const fn with_disk_write(mut self) -> Self {
        self.disk_write_count += 1;
        self
    }

    pub const fn with_disk_error(mut self) -> Self {
        self.disk_error_count += 1;
        self
    }

    pub fn accumulate(&mut self, other: Self) {
        self.request_count += other.request_count;
        self.memory_hit_count += other.memory_hit_count;
        self.disk_hit_count += other.disk_hit_count;
        self.compile_miss_count += other.compile_miss_count;
        self.disk_write_count += other.disk_write_count;
        self.disk_error_count += other.disk_error_count;
    }
}
