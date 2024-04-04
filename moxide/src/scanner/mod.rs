mod patterns;
mod scan_type;
mod scanner;
mod writer;

use crate::process::Process;
pub use patterns::*;
pub use scan_type::*;
pub use scanner::{ScanConfig, ScanResult, Scanner};
pub use writer::BasicWriter;

use scanner::BasicScanResult;

pub struct BasicScanner<T: ScannableCandidate> {
    result: Option<ListScanResult<T>>,
}

pub struct BasicScanResultGroup<T: ScannableCandidate> {
    results: Vec<BasicScanResult<T>>,
    start_address: usize,
    length: usize,
}

impl<T: ScannableCandidate> BasicScanResultGroup<T> {
    fn new(results: Vec<BasicScanResult<T>>, start_address: usize, length: usize) -> Self {
        Self {
            results,
            start_address,
            length,
        }
    }
}

pub struct ListScanResult<T: ScannableCandidate>(Vec<BasicScanResultGroup<T>>);

impl<T: ScannableCandidate> ScanResult<T> for ListScanResult<T> {
    fn to_list(&self) -> Vec<BasicScanResult<T>> {
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

impl<T: ScannableCandidate> From<Vec<BasicScanResultGroup<T>>> for ListScanResult<T> {
    fn from(value: Vec<BasicScanResultGroup<T>>) -> Self {
        Self(value)
    }
}

impl<T: ScannableCandidate> Scanner<T> for BasicScanner<T> {
    type Result = ListScanResult<T>;
    fn new_scan<Pattern: ScanPattern<T>>(
        &mut self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
    ) -> usize {
        let size = T::width();
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
                            if pattern.matches(&T::from_raw_bytes(window), &T::default()) {
                                Some(BasicScanResult::new(
                                    region.BaseAddress as usize + offset,
                                    T::from_raw_bytes(window),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<BasicScanResult<T>>>();
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
        let count = result.count();
        self.result = Some(result);
        count
    }
    fn next_scan<Pattern: ScanPattern<T>>(
        &mut self,
        target: &Process,
        config: &ScanConfig,
        pattern: &Pattern,
    ) -> usize {
        let size = T::width();
        match &mut self.result {
            Some(result) => {
                result.0.retain_mut(|group| {
                    let data = target
                        .read_memory(group.start_address, group.length)
                        .expect("Failed to read memory");
                    group.results.retain_mut(|result| {
                        let prev_value = result.value.clone();
                        result.value = T::from_raw_bytes(
                            data[(result.address - group.start_address)
                                ..(result.address - group.start_address + size)]
                                .try_into()
                                .expect("Failed to convert slice to array"),
                        );
                        pattern.matches(&result.value, &prev_value)
                    });
                    if group.results.is_empty() {
                        false
                    } else {
                        true
                    }
                });
                result.count()
            }
            None => 0,
        }
    }
    fn get_result(&self) -> Option<&Self::Result> {
        self.result.as_ref()
    }
}
