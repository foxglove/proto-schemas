use crate::log_sink::LogSink;
use crate::{Channel, FoxgloveError, Metadata};
use parking_lot::Mutex;
use std::sync::Arc;

pub struct MockSink;

impl LogSink for MockSink {
    fn log(
        &self,
        _channel: &Arc<Channel>,
        _msg: &[u8],
        _metadata: &Metadata,
    ) -> Result<(), FoxgloveError> {
        Ok(())
    }
}

pub struct LogCall {
    pub channel: Arc<Channel>,
    pub msg: Vec<u8>,
    pub metadata: Metadata,
}

pub struct RecordingSink {
    pub recorded: Mutex<Vec<LogCall>>,
}

impl RecordingSink {
    pub fn new() -> Self {
        Self {
            recorded: Mutex::new(Vec::new()),
        }
    }
}

impl LogSink for RecordingSink {
    fn log(
        &self,
        channel: &Arc<Channel>,
        msg: &[u8],
        metadata: &Metadata,
    ) -> Result<(), FoxgloveError> {
        let mut recorded = self.recorded.lock();
        recorded.push(LogCall {
            channel: channel.clone(),
            msg: msg.to_vec(),
            metadata: *metadata,
        });
        Ok(())
    }
}

pub struct ErrorSink;

#[derive(Debug, thiserror::Error)]
struct StrError(&'static str);

impl std::fmt::Display for StrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl LogSink for ErrorSink {
    fn log(
        &self,
        _channel: &Arc<Channel>,
        _msg: &[u8],
        _metadata: &Metadata,
    ) -> Result<(), FoxgloveError> {
        Err(FoxgloveError::Unspecified(Box::new(StrError(
            "ErrorSink always fails",
        ))))
    }
}
