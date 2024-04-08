use crate::scanner::{BasicScanner,ScanPattern};
use strum_macros::EnumString;
// Is there a better way?
macro_rules! type_to_enums {
    ($($t:ident),*) => {
        pub enum TypedScanner{
            $(
                $t(BasicScanner<$t>),
            )*
        }
        pub enum TypedPattern{
            $(
                $t(Box<dyn ScanPattern<$t>>),
            )*
        }
        #[derive(Debug, PartialEq, EnumString)]
        pub enum SupportedType{
            $(
                $t,
            )*
        }
    };
}

type_to_enums!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

macro_rules! match_types {
    ($variable:ident, $target $($type:ident), *) => {
        match $variable {
            ($type )*
        }
    };
    ($pattern:ty, $arg0:ident, $arg1:ident) => {
        
    };
}

macro_rules! type_to_pattern {
    ($arg0:ident, $pattern:ty, $($type:ident), *) => {
        match $arg0 {
            ($type(value)=>$pattern::$type(value))*
        }
    };
    ($arg0:ident, $arg1:ident, $pattern:ty, $($type:ident), *) => {
        match $arg0 {
            ($type(value)=>$pattern::$type(value))*
        }
    };
}

macro_rules! type_to_pattern_without_data {
    ($pattern:ty, $arg0:ident) => {
        
    };
}