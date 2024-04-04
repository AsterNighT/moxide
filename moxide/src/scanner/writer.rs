use crate::process::Process;
use color_eyre::eyre::Result;

use crate::scanner::RawBytes;

pub trait Writer {
    fn write<T: RawBytes>(&self, target: &Process, address: usize, value: &T) -> Result<usize> {
        target.write_memory(address, &value.to_raw_bytes())
    }
}

pub struct BasicWriter;
impl Writer for BasicWriter {}
