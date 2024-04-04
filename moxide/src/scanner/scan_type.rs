use std::fmt::Display;
use std::ops::{Add, Sub};

pub trait ScannableCandidate: RawBytes + Clone + Display + Default{}

// Allow exact matching
pub trait EqScannable: ScannableCandidate + PartialEq {}

// Allow matching pattern like "Increased" or "Decreased"
pub trait OrdScannable: EqScannable + PartialOrd {}

// Allows matching pattern like "IncreasedAtLeast"
pub trait NumericScannable: OrdScannable + Add<Output = Self> + Sub<Output = Self> {}

// A type that can be directly copied, casted to and from a piece of raw bytes.
// When things go back from rust to c.
pub unsafe trait RawBytes: Sized + Copy {
    fn from_raw_bytes(bytes: &[u8]) -> Self {
        unsafe { bytes.as_ptr().cast::<Self>().read_unaligned() }
    }
    fn to_raw_bytes(&self) -> Vec<u8> {
        let ptr = self as *const Self as *const u8;
        let len = Self::width();
        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
    }
    fn width() -> usize {
        std::mem::size_of::<Self>()
    }
}

unsafe impl<const N: usize> RawBytes for [u8; N] {}

macro_rules! impl_raw_bytes {
    ($($t:ty),*) => {
        $(
            unsafe impl RawBytes for $t {}
        )*
    };
}

impl_raw_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
