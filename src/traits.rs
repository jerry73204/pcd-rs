//! Traits definitions.

use crate::ValueKind;

/// This trait assocaites Rust primitive types to PCD primitive types.
pub trait Value
where
    Self: Copy,
{
    const KIND: ValueKind;
}

impl Value for u8 {
    const KIND: ValueKind = ValueKind::U8;
}

impl Value for u16 {
    const KIND: ValueKind = ValueKind::U16;
}

impl Value for u32 {
    const KIND: ValueKind = ValueKind::U32;
}

impl Value for i8 {
    const KIND: ValueKind = ValueKind::I8;
}

impl Value for i16 {
    const KIND: ValueKind = ValueKind::I16;
}

impl Value for i32 {
    const KIND: ValueKind = ValueKind::I32;
}

impl Value for f32 {
    const KIND: ValueKind = ValueKind::F32;
}

impl Value for f64 {
    const KIND: ValueKind = ValueKind::F64;
}
