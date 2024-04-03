mod scan_type;
mod scanner;
mod writer;
use crate::process::Process;
pub use crate::scanner::scan_type::ScanType;

pub use self::scanner::{BasicScanPattern, BasicScanResult};
pub use self::scanner::{ScanConfig, ScanPattern, ScanResult, Scanner};
pub use self::writer::Writer;

pub struct BasicScanner {}

pub struct BasicScanResultGroup {
    results: Vec<BasicScanResult>,
    start_address: usize,
    length: usize,
}

impl BasicScanResultGroup {
    fn new(results: Vec<BasicScanResult>, start_address: usize, length: usize) -> Self {
        Self {
            results,
            start_address,
            length,
        }
    }
}

pub struct ListScanResult(Vec<BasicScanResultGroup>);

impl ScanResult for ListScanResult {
    fn to_list(&self) -> Vec<BasicScanResult> {
        self.0
            .iter()
            .flat_map(|group| group.results.clone())
            .collect()
    }
    fn count(&self) -> usize {
        self.0.iter().map(|group| group.results.len()).sum()
    }
    fn merge(&mut self, target: &mut Self) {
        self.0.append(&mut target.0);
    }
    fn new() -> Self {
        ListScanResult(vec![])
    }
}

impl From<Vec<BasicScanResultGroup>> for ListScanResult {
    fn from(value: Vec<BasicScanResultGroup>) -> Self {
        Self(value)
    }
}

impl Scanner for BasicScanner {
    type Result = ListScanResult;
    fn new_scan<Pattern: ScanPattern>(
        &self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
    ) -> Self::Result {
        let size = config.width;
        let mut result = Self::Result::new();
        let regions = target
            .memory_regions()
            .into_iter()
            .filter(|p| (config.match_memory_region(p.Protect)))
            .collect::<Vec<_>>();

        tracing::debug!("Scanning {} memory regions", regions.len());
        tracing::debug!("Scanning at a {} bytes align", config.alignment);

        regions.into_iter().for_each(|region| {
            match target.read_memory(region.BaseAddress as _, region.RegionSize) {
                Ok(memory) => {
                    let partial_result = memory
                        .windows(size)
                        .enumerate()
                        .step_by(config.alignment)
                        .filter_map(|(offset, window)| {
                            if pattern
                                .matches(
                                    &ScanType::from_ne_bytes(window, pattern.type_reference()),
                                    &Default::default(),
                                )
                                .ok()?
                            {
                                Some(BasicScanResult::new(
                                    region.BaseAddress as usize + offset,
                                    ScanType::from_ne_bytes(window, pattern.type_reference()),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<BasicScanResult>>();
                    let result_group = BasicScanResultGroup::new(
                        partial_result,
                        region.BaseAddress as usize,
                        region.RegionSize,
                    );
                    result.merge(&mut vec![result_group].into());
                }
                Err(err) => tracing::debug!(
                    "Failed to read {} bytes at {:?}: {}",
                    region.RegionSize,
                    region.BaseAddress,
                    err,
                ),
            }
        });
        result
    }
    fn next_scan<Pattern: ScanPattern>(
        &self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
        former_result: &mut Self::Result,
    ) {
        let size = config.width;
        former_result.0.retain_mut(|group| {
            let data = target
                .read_memory(group.start_address, group.length)
                .expect("Failed to read memory");
            group.results.retain_mut(|result| {
                let prev_value = result.value.clone();
                result.value = ScanType::from_ne_bytes(
                    data[(result.address - group.start_address)
                        ..(result.address - group.start_address + size)]
                        .try_into()
                        .expect("Failed to convert slice to array"),
                    pattern.type_reference(),
                );
                pattern.matches(&result.value, &prev_value).is_ok_and(|r| r)
            });
            if group.results.is_empty() {
                false
            } else {
                true
            }
        });
    }
}
