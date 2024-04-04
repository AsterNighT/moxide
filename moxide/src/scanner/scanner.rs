use crate::process::Process;
use color_eyre::eyre::{eyre, Result};
use winapi::um::winnt;

use super::{patterns::ScanPattern, ScannableCandidate};

pub struct ScanConfig {
    memory_region_mask: u32,
    pub alignment: usize,
}

impl ScanConfig {
    pub fn new(memory_region_mask: u32, alignment: usize) -> Self {
        Self {
            memory_region_mask,
            alignment,
        }
    }
    pub fn match_memory_region(&self, protect: u32) -> bool {
        (protect & self.memory_region_mask) != 0
    }
}

pub trait ScanResult<T:ScannableCandidate> {
    fn to_list(&self) -> Vec<BasicScanResult<T>>;
    fn count(&self) -> usize;
    fn merge(&mut self, target: &mut Self);

    fn new() -> Self;
}

#[derive(Clone, Debug)]
pub struct BasicScanResult<T:ScannableCandidate> {
    pub address: usize,
    pub value: T,
}

impl<T:ScannableCandidate> BasicScanResult<T> {
    pub fn new(address: usize, value: T) -> Self {
        BasicScanResult { address, value }
    }

    pub fn refresh(&mut self, target: &Process) {
        if let Ok(bytes) = target.read_memory(self.address, T::width()) {
            self.value = T::from_raw_bytes(&bytes);
        }
    }
}

pub trait Scanner<T:ScannableCandidate> {
    type Result: ScanResult<T>;
    fn new_scan<Pattern: ScanPattern<T>>(
        &mut self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
    ) -> usize;
    fn next_scan<Pattern: ScanPattern<T>>(
        &mut self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
    ) -> usize;
    fn get_result(&self) -> Option<&Self::Result>;
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            memory_region_mask: winnt::PAGE_EXECUTE_READWRITE
                | winnt::PAGE_EXECUTE_WRITECOPY
                | winnt::PAGE_READWRITE
                | winnt::PAGE_WRITECOPY,
            alignment: 4,
        }
    }
}
