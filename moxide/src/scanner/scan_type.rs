use color_eyre::eyre::{eyre, Result};
use derive_more::{Add, Display, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, Sub, Display)]
pub enum ScanType {
    #[display(fmt = "{_0}")]
    I32(i32),
    I16(i16),
    F32(f32),
    F64(f64),
}

impl ScanType {
    pub fn width(&self) -> usize {
        match self {
            ScanType::I32(_) => 4,
            ScanType::I16(_) => 2,
            ScanType::F32(_) => 4,
            ScanType::F64(_) => 8,
        }
    }
    pub fn from_ne_bytes(bytes: &[u8], type_reference: &Self) -> Self {
        match type_reference {
            ScanType::I32(_) => ScanType::I32(i32::from_ne_bytes(bytes.try_into().unwrap())),
            ScanType::I16(_) => ScanType::I16(i16::from_ne_bytes(bytes.try_into().unwrap())),
            ScanType::F32(_) => ScanType::F32(f32::from_ne_bytes(bytes.try_into().unwrap())),
            ScanType::F64(_) => ScanType::F64(f64::from_ne_bytes(bytes.try_into().unwrap())),
            _ => panic!("Invalid byte length"),
        }
    }
    pub fn to_ne_bytes(&self) -> Vec<u8> {
        match self {
            ScanType::I32(value) => value.to_ne_bytes().to_vec(),
            ScanType::I16(value) => value.to_ne_bytes().to_vec(),
            ScanType::F32(value) => value.to_ne_bytes().to_vec(),
            ScanType::F64(value) => value.to_ne_bytes().to_vec(),
            _ => panic!("Invalid type"),
        }
    }
    pub fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();
        let pos = value.find(|c: char| c == '_');
        let (value, data_type) = if let Some(pos) = pos {
            let (value, data_type) = value.split_at(pos);
            let data_type = data_type[1..].trim();
            (value, data_type)
        } else {
            (value, "i32")
        };
        let value = if value.is_empty() { "0" } else { value };
        match data_type {
            "i32" => Ok(ScanType::I32(value.parse::<i32>()?)),
            "i16" => Ok(ScanType::I16(value.parse::<i16>()?)),
            "f32" => Ok(ScanType::F32(value.parse::<f32>()?)),
            "f64" => Ok(ScanType::F64(value.parse::<f64>()?)),
            _ => Err(eyre!("Invalid type")),
        }
    }
}

/// Sometimes we need a dummy previous value
/// to compare against, this is the default value
impl Default for ScanType {
    fn default() -> Self {
        ScanType::I32(0)
    }
}

#[cfg(test)]
mod test {
    use crate::scanner::ScanType;


    #[test]
    fn should_display_scan_type() {
        let scan_type = ScanType::I32(10);
        assert_eq!(format!("{}", scan_type), "10");
    }
}
