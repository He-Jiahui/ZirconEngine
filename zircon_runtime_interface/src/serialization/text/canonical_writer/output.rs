use std::io::Write;
use std::rc::Rc;

use super::super::super::write_error::CanonicalTextWriteError;
use super::super::canonical_spool::{
    canonical_spool_resource_limit, SpoolAttempt, MAX_CANONICAL_SPOOL_WORK_BYTES,
};

pub(in crate::serialization::text) const COPY_BUFFER_BYTES: usize = 64 * 1024;
pub(in crate::serialization) const MAX_CANONICAL_NESTING_DEPTH: usize = 128;
pub(in crate::serialization) const MAX_CANONICAL_OBJECT_ENTRIES: usize = 16_384;

pub(super) struct OutputBudget {
    pub(super) bytes: usize,
    max_bytes: usize,
    spool_work_bytes: usize,
    spool_attempt: Rc<SpoolAttempt>,
}

impl OutputBudget {
    pub(super) fn new(max_bytes: usize) -> Self {
        Self {
            bytes: 0,
            max_bytes,
            spool_work_bytes: 0,
            spool_attempt: Rc::new(SpoolAttempt::new()),
        }
    }

    pub(super) fn reserve(&mut self, additional: usize) -> Result<(), CanonicalTextWriteError> {
        let found =
            self.bytes
                .checked_add(additional)
                .ok_or(CanonicalTextWriteError::OutputTooLarge {
                    max: self.max_bytes,
                    found: usize::MAX,
                })?;
        if found > self.max_bytes {
            return Err(CanonicalTextWriteError::OutputTooLarge {
                max: self.max_bytes,
                found,
            });
        }
        self.bytes = found;
        Ok(())
    }

    pub(super) fn release(&mut self, bytes: usize) {
        self.bytes = self.bytes.saturating_sub(bytes);
    }

    pub(super) fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub(super) fn reserve_spool_work(
        &mut self,
        additional: usize,
    ) -> Result<(), CanonicalTextWriteError> {
        let found = self
            .spool_work_bytes
            .checked_add(additional)
            .unwrap_or(usize::MAX);
        ensure_resource_limit(
            "canonical spool work bytes",
            found,
            MAX_CANONICAL_SPOOL_WORK_BYTES,
        )?;
        self.spool_work_bytes = found;
        Ok(())
    }

    pub(super) fn ensure_nesting_depth(&self, found: usize) -> Result<(), CanonicalTextWriteError> {
        ensure_resource_limit(
            "canonical nesting depth",
            found,
            MAX_CANONICAL_NESTING_DEPTH,
        )
    }

    pub(super) fn ensure_object_entries(
        &self,
        found: usize,
    ) -> Result<(), CanonicalTextWriteError> {
        ensure_resource_limit(
            "canonical object entries",
            found,
            MAX_CANONICAL_OBJECT_ENTRIES,
        )
    }

    pub(super) fn spool_attempt(&self) -> Rc<SpoolAttempt> {
        Rc::clone(&self.spool_attempt)
    }
}

pub(in crate::serialization::text) struct CountingWriter<'sink, 'budget, W: Write + ?Sized> {
    sink: &'sink mut W,
    pub(super) budget: &'budget mut OutputBudget,
    counts_spool_work: bool,
}

impl<'sink, 'budget, W: Write + ?Sized> CountingWriter<'sink, 'budget, W> {
    pub(super) fn new(sink: &'sink mut W, budget: &'budget mut OutputBudget) -> Self {
        Self {
            sink,
            budget,
            counts_spool_work: false,
        }
    }

    pub(super) fn new_spool(sink: &'sink mut W, budget: &'budget mut OutputBudget) -> Self {
        Self {
            sink,
            budget,
            counts_spool_work: true,
        }
    }

    pub(super) fn write_counted(&mut self, bytes: &[u8]) -> Result<(), CanonicalTextWriteError> {
        self.budget.reserve(bytes.len())?;
        if self.counts_spool_work {
            self.budget.reserve_spool_work(bytes.len())?;
        }
        self.write_all_bounded(bytes)
    }

    pub(in crate::serialization::text) fn write_preaccounted(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), CanonicalTextWriteError> {
        if self.counts_spool_work {
            self.budget.reserve_spool_work(bytes.len())?;
        }
        self.write_all_bounded(bytes)
    }

    fn write_all_bounded(&mut self, bytes: &[u8]) -> Result<(), CanonicalTextWriteError> {
        for chunk in bytes.chunks(COPY_BUFFER_BYTES) {
            self.sink.write_all(chunk).map_err(|source| {
                if let Some((resource, max, found)) = canonical_spool_resource_limit(&source) {
                    CanonicalTextWriteError::ResourceLimitExceeded {
                        resource,
                        max,
                        found,
                    }
                } else {
                    CanonicalTextWriteError::Io {
                        operation: "write canonical text",
                        source,
                    }
                }
            })?;
        }
        Ok(())
    }

    pub(super) fn flush(&mut self) -> Result<(), CanonicalTextWriteError> {
        self.sink
            .flush()
            .map_err(|source| CanonicalTextWriteError::Io {
                operation: "flush canonical text",
                source,
            })
    }
}

fn ensure_resource_limit(
    resource: &'static str,
    found: usize,
    max: usize,
) -> Result<(), CanonicalTextWriteError> {
    if found > max {
        return Err(CanonicalTextWriteError::ResourceLimitExceeded {
            resource,
            max,
            found,
        });
    }
    Ok(())
}
