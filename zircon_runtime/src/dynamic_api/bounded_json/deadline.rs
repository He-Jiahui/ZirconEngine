use std::io::{self, Read};
use std::time::{Duration, Instant};

use super::BoundedJsonError;

#[derive(Clone, Copy)]
pub(super) struct ProcessingDeadline {
    started: Instant,
    pub(super) limit: Duration,
}

impl ProcessingDeadline {
    pub(super) fn new(limit_micros: u64) -> Self {
        Self {
            started: Instant::now(),
            limit: Duration::from_micros(limit_micros),
        }
    }

    pub(super) fn exceeded(self) -> bool {
        self.started.elapsed() > self.limit
    }

    pub(super) fn check(self) -> Result<(), BoundedJsonError> {
        if self.exceeded() {
            return Err(BoundedJsonError::ProcessingTime {
                limit_micros: self.limit.as_micros().try_into().unwrap_or(u64::MAX),
            });
        }
        Ok(())
    }
}

pub(super) struct DeadlineReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    deadline: ProcessingDeadline,
}

impl<'a> DeadlineReader<'a> {
    pub(super) fn new(bytes: &'a [u8], deadline: ProcessingDeadline) -> Self {
        Self {
            bytes,
            offset: 0,
            deadline,
        }
    }
}

impl Read for DeadlineReader<'_> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        self.deadline.check().map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded JSON processing deadline exceeded",
            )
        })?;
        if self.offset == self.bytes.len() {
            return Ok(0);
        }
        let count = destination
            .len()
            .min(4 * 1024)
            .min(self.bytes.len() - self.offset);
        destination[..count].copy_from_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}
