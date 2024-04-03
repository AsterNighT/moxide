use crate::process::Process;
use color_eyre::eyre::{eyre, Result};
use winapi::um::winnt;

use super::scan_type::ScanType;

pub trait ScanPattern {
    fn matches(&self, value: &ScanType, prev_value: &ScanType) -> Result<bool>;
    fn type_reference(&self) -> &ScanType;
}

/// In some cases the contained value is rather meaningless, like for the Unknown pattern.
/// The value is only used for the type reference.
pub enum BasicScanPattern {
    Exact(ScanType),
    GreaterOrEqualThan(ScanType),
    LessOrEqualThan(ScanType),
    Between(ScanType, ScanType),
    Increased(ScanType),
    IncreasedBy(ScanType),
    IncreasedAtLeast(ScanType),
    IncreasedAtMost(ScanType),
    Decreased(ScanType),
    DecreasedBy(ScanType),
    DecreasedAtLeast(ScanType),
    DecreasedAtMost(ScanType),
    Unchanged(ScanType),
    Changed(ScanType),
    Unknown(ScanType),
}

impl Default for BasicScanPattern {
    fn default() -> Self {
        BasicScanPattern::Unknown(Default::default())
    }
}

impl ScanPattern for BasicScanPattern {
    fn matches(&self, value: &ScanType, prev_value: &ScanType) -> Result<bool> {
        match self {
            BasicScanPattern::Exact(expected) => Ok(value == expected),
            BasicScanPattern::GreaterOrEqualThan(expected) => Ok(value >= expected),
            BasicScanPattern::LessOrEqualThan(expected) => Ok(value <= expected),
            BasicScanPattern::Between(min, max) => Ok(value >= min && value <= max),
            BasicScanPattern::Increased(_) => Ok(value > prev_value),
            BasicScanPattern::IncreasedBy(diff) => Ok(*value
                == (*prev_value + *diff)
                    .map_err(|_| eyre!("Cannot add two value with different types"))?),
            BasicScanPattern::IncreasedAtLeast(diff) => Ok(*value
                >= (*prev_value + *diff)
                    .map_err(|_| eyre!("Cannot add two value with different types"))?),
            BasicScanPattern::IncreasedAtMost(diff) => Ok(*value
                <= (*prev_value + *diff)
                    .map_err(|_| eyre!("Cannot add two value with different types"))?),
            BasicScanPattern::Decreased(_) => Ok(value < prev_value),
            BasicScanPattern::DecreasedBy(diff) => Ok(*value
                == (*prev_value - *diff)
                    .map_err(|_| eyre!("Cannot add two value with different types"))?),
            BasicScanPattern::DecreasedAtLeast(diff) => Ok(*value
                <= (*prev_value - *diff)
                    .map_err(|_| eyre!("Cannot add two value with different types"))?),
            BasicScanPattern::DecreasedAtMost(diff) => Ok(*value
                >= (*prev_value - *diff)
                    .map_err(|_| eyre!("Cannot add two value with different types"))?),
            BasicScanPattern::Unchanged(_) => Ok(value == prev_value),
            BasicScanPattern::Changed(_) => Ok(value != prev_value),
            BasicScanPattern::Unknown(_) => Ok(true),
        }
    }
    fn type_reference(&self) -> &ScanType {
        match self {
            BasicScanPattern::Exact(value)
            | BasicScanPattern::GreaterOrEqualThan(value)
            | BasicScanPattern::LessOrEqualThan(value)
            | BasicScanPattern::Between(value, _)
            | BasicScanPattern::Increased(value)
            | BasicScanPattern::IncreasedBy(value)
            | BasicScanPattern::IncreasedAtLeast(value)
            | BasicScanPattern::IncreasedAtMost(value)
            | BasicScanPattern::Decreased(value)
            | BasicScanPattern::DecreasedBy(value)
            | BasicScanPattern::DecreasedAtLeast(value)
            | BasicScanPattern::DecreasedAtMost(value)
            | BasicScanPattern::Unchanged(value)
            | BasicScanPattern::Changed(value)
            | BasicScanPattern::Unknown(value) => value,
        }
    }
}

pub struct ScanConfig {
    memory_region_mask: u32,
    pub alignment: usize,
    pub width: usize,
}

impl ScanConfig {
    pub fn new(memory_region_mask: u32, alignment: usize, width: usize) -> Self {
        Self {
            memory_region_mask,
            alignment,
            width,
        }
    }
    pub fn match_memory_region(&self, protect: u32) -> bool {
        (protect & self.memory_region_mask) != 0
    }
}

pub trait ScanResult {
    fn to_list(&self) -> Vec<BasicScanResult>;
    fn count(&self) -> usize;
    fn merge(&mut self, target: &mut Self);

    fn new() -> Self;
}

#[derive(Clone, Debug)]
pub struct BasicScanResult {
    pub address: usize,
    pub value: ScanType,
}

impl BasicScanResult {
    pub fn new(address: usize, value: ScanType) -> Self {
        BasicScanResult { address, value }
    }

    pub fn refresh(&mut self, target: &Process) {
        if let Ok(bytes) = target.read_memory(self.address, self.value.width()) {
            self.value = ScanType::from_ne_bytes(&bytes,&self.value);
        }
    }
}

pub trait Scanner {
    type Result: ScanResult;
    fn new_scan<Pattern: ScanPattern>(
        &self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
    ) -> Self::Result;
    fn next_scan<Pattern: ScanPattern>(
        &self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
        former_result: &mut Self::Result,
    );
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            memory_region_mask: winnt::PAGE_EXECUTE_READWRITE
                | winnt::PAGE_EXECUTE_WRITECOPY
                | winnt::PAGE_READWRITE
                | winnt::PAGE_WRITECOPY,
            alignment: 4,
            width: 4,
        }
    }
}
