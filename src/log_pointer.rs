use std::convert::From;
use std::ops::Range;

pub struct LogPointer {
    pub log_file_id: u64,
    pub start_position: u64,
    pub len: u64
}

impl From<(u64, Range<u64>)> for LogPointer {
    fn from((id, range): (u64, Range<u64>)) -> Self {
        Self {
            log_file_id: id,
            start_position: range.start,
            len: range.end - range.start
        }
    }
}