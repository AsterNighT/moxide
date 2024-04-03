use crate::process::Process;
use color_eyre::eyre::Result;

use super::scan_type::ScanType;
pub struct Writer {}

impl Writer {
    pub fn new() -> Self {
        Self {}
    }
    pub fn write(&self, target: &Process, address: usize, value: &ScanType) -> Result<usize> {
        target.write_memory(address, &value.to_ne_bytes())
    }
}
