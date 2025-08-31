//! Types for PCD metadata.

use std::{iter::FromIterator, ops::Index};

/// The struct keep meta data of PCD file.
#[derive(Debug, Clone, PartialEq)]
pub struct PcdMeta {
    pub version: String,
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub num_points: u64,
    pub data: DataKind,
    pub field_defs: Schema,
}

/// Represents VIEWPOINT field in meta data.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewPoint {
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}

impl Default for ViewPoint {
    fn default() -> Self {
        ViewPoint {
            tx: 0.0,
            ty: 0.0,
            tz: 0.0,
            qw: 1.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
        }
    }
}

/// The enum indicates whether the point cloud data is encoded in Ascii, binary, or compressed binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataKind {
    Ascii,
    Binary,
    BinaryCompressed,
}

/// The enum specifies one of signed, unsigned integers, and floating point number type to the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    I,
    U,
    F,
}

/// The enum specifies the exact type for each PCD field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
}

/// Define the properties of a PCD field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDef {
    pub name: String,
    pub kind: ValueKind,
    pub count: u64,
}

/// Define the schema of PCD format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub fields: Vec<FieldDef>,
}

impl Schema {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, FieldDef> {
        self.into_iter()
    }
}

impl Index<usize> for Schema {
    type Output = FieldDef;

    fn index(&self, index: usize) -> &Self::Output {
        self.fields.index(index)
    }
}

impl IntoIterator for Schema {
    type Item = FieldDef;
    type IntoIter = std::vec::IntoIter<FieldDef>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a Schema {
    type Item = &'a FieldDef;
    type IntoIter = std::slice::Iter<'a, FieldDef>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

impl FromIterator<(String, ValueKind, u64)> for Schema {
    fn from_iter<T: IntoIterator<Item = (String, ValueKind, u64)>>(iter: T) -> Self {
        let fields = iter
            .into_iter()
            .map(|(name, kind, count)| FieldDef { name, kind, count })
            .collect();
        Self { fields }
    }
}

impl<'a> FromIterator<(&'a str, ValueKind, u64)> for Schema {
    fn from_iter<T: IntoIterator<Item = (&'a str, ValueKind, u64)>>(iter: T) -> Self {
        iter.into_iter()
            .map(|(name, kind, count)| (name.to_string(), kind, count))
            .collect()
    }
}

impl FromIterator<FieldDef> for Schema {
    fn from_iter<T: IntoIterator<Item = FieldDef>>(iter: T) -> Self {
        Self {
            fields: iter.into_iter().collect(),
        }
    }
}

impl<'a> FromIterator<&'a FieldDef> for Schema {
    fn from_iter<T: IntoIterator<Item = &'a FieldDef>>(iter: T) -> Self {
        Self {
            fields: iter.into_iter().map(|field| field.to_owned()).collect(),
        }
    }
}
