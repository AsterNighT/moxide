use crate::process::Process;
use color_eyre::eyre::Result;

use super::Scannable;

pub struct BasicWriter;
impl BasicWriter {
    pub fn new() -> Self {
        BasicWriter {}
    }
    pub fn write<T: Scannable>(&self, target: &Process, address: usize, value: &T) -> Result<usize> {
        target.write_memory(address, &value.mem_view())
    }
}
