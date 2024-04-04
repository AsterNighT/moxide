use std::marker::PhantomData;

use crate::scanner::scan_type::{EqScannable, OrdScannable, ScannableCandidate};

use super::NumericScannable;

pub trait ScanPattern<T:ScannableCandidate> {
    fn matches(&self, value: &T, prev_value: &T) -> bool;
}

pub struct Exact<T:EqScannable>(T);
impl<T:EqScannable> ScanPattern<T> for Exact<T> {
    fn matches(&self, value: &T, _prev_value: &T) -> bool {
        value == &self.0
    }
}
pub struct GreaterOrEqualThan<T:OrdScannable>(T);
impl<T:OrdScannable> ScanPattern<T> for GreaterOrEqualThan<T> {
    fn matches(&self, value: &T, _prev_value: &T) -> bool {
        value >= &self.0
    }
}
pub struct LessOrEqualThan<T:OrdScannable>(T);
impl<T:OrdScannable> ScanPattern<T> for LessOrEqualThan<T> {
    fn matches(&self, value: &T, _prev_value: &T) -> bool {
        value <= &self.0
    }
}
pub struct Between<T:OrdScannable>(T, T);
impl<T:OrdScannable> ScanPattern<T> for Between<T> {
    fn matches(&self, value: &T, _prev_value: &T) -> bool {
        value >= &self.0 && value <= &self.1
    }
}
pub struct Increased<T:OrdScannable>(PhantomData<T>);
impl<T:OrdScannable> ScanPattern<T> for Increased<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        value > prev_value
    }
}
pub struct IncreasedBy<T:NumericScannable>(T);
impl<T:NumericScannable> ScanPattern<T> for IncreasedBy<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        *value == *prev_value + self.0
    }
}
pub struct IncreasedAtLeast<T:NumericScannable>(T);
impl<T:NumericScannable> ScanPattern<T> for IncreasedAtLeast<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        *value >= *prev_value + self.0
    }
}
pub struct IncreasedAtMost<T:NumericScannable>(T);
impl<T:NumericScannable> ScanPattern<T> for IncreasedAtMost<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        *value <= *prev_value + self.0
    }
}
pub struct Decreased<T:OrdScannable>(PhantomData<T>);
impl<T:OrdScannable> ScanPattern<T> for Decreased<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        value < prev_value
    }
}
pub struct DecreasedBy<T:NumericScannable>(T);
impl<T:NumericScannable> ScanPattern<T> for DecreasedBy<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        *value == *prev_value - self.0
    }
}
pub struct DecreasedAtLeast<T:NumericScannable>(T);
impl<T:NumericScannable> ScanPattern<T> for DecreasedAtLeast<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        *value <= *prev_value - self.0
    }
}
pub struct DecreasedAtMost<T:NumericScannable>(T);
impl<T:NumericScannable> ScanPattern<T> for DecreasedAtMost<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        *value >= *prev_value - self.0
    }
}
pub struct Unchanged<T:EqScannable>(PhantomData<T>);
impl<T:EqScannable> ScanPattern<T> for Unchanged<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        value == prev_value
    }
}
pub struct Changed<T:EqScannable>(PhantomData<T>);
impl<T:EqScannable> ScanPattern<T> for Changed<T> {
    fn matches(&self, value: &T, prev_value: &T) -> bool {
        value != prev_value
    }
}
pub struct Unknown<T:ScannableCandidate>(PhantomData<T>);
impl<T:ScannableCandidate> ScanPattern<T> for Unknown<T> {
    fn matches(&self, _value: &T, _prev_value: &T) -> bool {
        true
    }
}

