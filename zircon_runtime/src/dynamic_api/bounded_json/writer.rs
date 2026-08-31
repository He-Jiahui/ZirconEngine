use std::io::{self, Write};

use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

use super::deadline::ProcessingDeadline;
use super::BoundedJsonError;

pub(super) struct BoundedJsonCountingWriter {
    count: usize,
    limit: ZrRuntimePayloadLimitV1,
    deadline: ProcessingDeadline,
    nesting: JsonNestingTracker,
    failure: Option<BoundedJsonError>,
}

impl BoundedJsonCountingWriter {
    pub(super) fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self {
            count: 0,
            limit,
            deadline: ProcessingDeadline::new(limit.max_processing_time_micros),
            nesting: JsonNestingTracker::default(),
            failure: None,
        }
    }

    pub(super) fn finish(
        mut self,
        result: Result<(), serde_json::Error>,
    ) -> Result<(), BoundedJsonError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| BoundedJsonError::Json(error.to_string()))?;
        self.deadline.check()?;
        if self.count == 0 && !self.limit.allow_empty {
            return Err(BoundedJsonError::Empty);
        }
        Ok(())
    }
}

impl Write for BoundedJsonCountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other(
                "bounded JSON counting writer already failed",
            ));
        }
        if let Err(error) = self.deadline.check() {
            self.failure = Some(error);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded JSON processing deadline exceeded",
            ));
        }
        let Some(encoded_len) = self.count.checked_add(bytes.len()) else {
            self.failure = Some(BoundedJsonError::EncodedBytes {
                observed: usize::MAX,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other("bounded JSON byte count overflowed"));
        };
        if encoded_len > self.limit.max_encoded_bytes {
            self.failure = Some(BoundedJsonError::EncodedBytes {
                observed: encoded_len,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other("bounded JSON byte limit exceeded"));
        }
        if let Err(error) = self.nesting.inspect(bytes, self.limit.max_nesting_depth) {
            self.failure = Some(error);
            return Err(io::Error::other("bounded JSON nesting depth exceeded"));
        }
        self.count = encoded_len;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(in crate::dynamic_api) struct BoundedJsonWriter {
    bytes: Vec<u8>,
    limit: ZrRuntimePayloadLimitV1,
    pub(super) deadline: ProcessingDeadline,
    nesting: JsonNestingTracker,
    failure: Option<BoundedJsonError>,
}

impl BoundedJsonWriter {
    pub(in crate::dynamic_api) fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self::with_capacity(limit, 0)
    }

    pub(in crate::dynamic_api) fn with_capacity(
        limit: ZrRuntimePayloadLimitV1,
        capacity: usize,
    ) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity.min(limit.max_encoded_bytes)),
            limit,
            deadline: ProcessingDeadline::new(limit.max_processing_time_micros),
            nesting: JsonNestingTracker::default(),
            failure: None,
        }
    }

    pub(in crate::dynamic_api) fn finish(
        mut self,
        result: Result<(), serde_json::Error>,
    ) -> Result<Vec<u8>, BoundedJsonError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| BoundedJsonError::Json(error.to_string()))?;
        self.deadline.check()?;
        if self.bytes.is_empty() && !self.limit.allow_empty {
            return Err(BoundedJsonError::Empty);
        }
        Ok(self.bytes)
    }

    pub(in crate::dynamic_api) fn finish_io_result(
        mut self,
        result: io::Result<()>,
    ) -> Result<Vec<u8>, BoundedJsonError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| BoundedJsonError::Json(error.to_string()))?;
        self.deadline.check()?;
        if self.bytes.is_empty() && !self.limit.allow_empty {
            return Err(BoundedJsonError::Empty);
        }
        Ok(self.bytes)
    }
}

impl Write for BoundedJsonWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other("bounded JSON writer already failed"));
        }
        if let Err(error) = self.deadline.check() {
            self.failure = Some(error);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "bounded JSON processing deadline exceeded",
            ));
        }
        let Some(encoded_len) = self.bytes.len().checked_add(bytes.len()) else {
            let error = BoundedJsonError::EncodedBytes {
                observed: usize::MAX,
                limit: self.limit.max_encoded_bytes,
            };
            self.failure = Some(error);
            return Err(io::Error::other("bounded JSON byte count overflowed"));
        };
        if encoded_len > self.limit.max_encoded_bytes {
            self.failure = Some(BoundedJsonError::EncodedBytes {
                observed: encoded_len,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other("bounded JSON byte limit exceeded"));
        }
        if let Err(error) = self.nesting.inspect(bytes, self.limit.max_nesting_depth) {
            self.failure = Some(error);
            return Err(io::Error::other("bounded JSON nesting depth exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct JsonNestingTracker {
    pub(super) depth: usize,
    in_string: bool,
    escaped: bool,
}

impl JsonNestingTracker {
    pub(super) fn inspect(&mut self, bytes: &[u8], limit: usize) -> Result<(), BoundedJsonError> {
        for byte in bytes {
            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.in_string = false;
                }
                continue;
            }
            match *byte {
                b'"' => self.in_string = true,
                b'{' | b'[' => {
                    self.depth = self.depth.saturating_add(1);
                    if self.depth > limit {
                        return Err(BoundedJsonError::NestingDepth {
                            observed: self.depth,
                            limit,
                        });
                    }
                }
                b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        Ok(())
    }
}
