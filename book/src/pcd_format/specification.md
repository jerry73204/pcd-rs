# PCD File Format Specification

The Point Cloud Data (PCD) file format is the primary file format for the Point Cloud Library (PCL). This chapter provides a complete specification of the PCD format as implemented in pcd-rs.

## Overview

PCD files consist of:
1. **Header**: ASCII metadata describing the point cloud
2. **Data**: Point data in ASCII, binary, or compressed format

## Format Versions

pcd-rs currently supports:
- **Version 0.7**: The current standard version
- Earlier versions are not yet supported

## File Structure

```
# .PCD v0.7 - Point Cloud Data file format
VERSION 0.7
FIELDS x y z rgb
SIZE 4 4 4 4
TYPE F F F F
COUNT 1 1 1 1
WIDTH 213
HEIGHT 1
VIEWPOINT 0 0 0 1 0 0 0
POINTS 213
DATA ascii
0.93773 0.33763 0 4.2108e+06
0.90805 0.35641 0 4.2108e+06
...
```

## Header Fields

The header contains the following fields in **strict order**:

### 1. VERSION
- **Format**: `VERSION <version>`
- **Example**: `VERSION 0.7`
- **Description**: PCD file format version
- **Required**: Yes

### 2. FIELDS
- **Format**: `FIELDS <field1> <field2> ...`
- **Example**: `FIELDS x y z rgb normal_x normal_y normal_z`
- **Description**: Names of each dimension/field
- **Required**: Yes

### 3. SIZE
- **Format**: `SIZE <size1> <size2> ...`
- **Example**: `SIZE 4 4 4 4`
- **Description**: Size in bytes of each field
- **Required**: Yes
- **Valid values**: 1, 2, 4, 8

### 4. TYPE
- **Format**: `TYPE <type1> <type2> ...`
- **Example**: `TYPE F F F U`
- **Description**: Data type of each field
- **Required**: Yes
- **Valid values**:
  - `I`: Signed integer
  - `U`: Unsigned integer
  - `F`: Floating point

### 5. COUNT
- **Format**: `COUNT <count1> <count2> ...`
- **Example**: `COUNT 1 1 1 1`
- **Description**: Number of elements per field (for arrays)
- **Required**: Yes
- **Default**: 1 (scalar field)

### 6. WIDTH
- **Format**: `WIDTH <width>`
- **Example**: `WIDTH 640`
- **Description**: Width of the point cloud
- **Required**: Yes
- **Notes**:
  - For unorganized clouds: total number of points
  - For organized clouds: width of the image

### 7. HEIGHT
- **Format**: `HEIGHT <height>`
- **Example**: `HEIGHT 480`
- **Description**: Height of the point cloud
- **Required**: Yes
- **Notes**:
  - For unorganized clouds: 1
  - For organized clouds: height of the image

### 8. VIEWPOINT
- **Format**: `VIEWPOINT <tx> <ty> <tz> <qw> <qx> <qy> <qz>`
- **Example**: `VIEWPOINT 0 0 0 1 0 0 0`
- **Description**: Acquisition viewpoint
- **Required**: Yes
- **Components**:
  - `tx, ty, tz`: Translation
  - `qw, qx, qy, qz`: Quaternion (rotation)
- **Default**: `0 0 0 1 0 0 0` (identity transform)

### 9. POINTS
- **Format**: `POINTS <count>`
- **Example**: `POINTS 307200`
- **Description**: Total number of points
- **Required**: Yes
- **Validation**: Must equal WIDTH × HEIGHT

### 10. DATA
- **Format**: `DATA <type>`
- **Example**: `DATA binary`
- **Description**: Data storage format
- **Required**: Yes
- **Valid values**:
  - `ascii`: Human-readable text
  - `binary`: Raw binary data
  - `binary_compressed`: LZF compressed binary

## Data Type Mappings

| TYPE | SIZE | Rust Type | Description |
|------|------|-----------|-------------|
| I | 1 | i8 | Signed 8-bit integer |
| I | 2 | i16 | Signed 16-bit integer |
| I | 4 | i32 | Signed 32-bit integer |
| U | 1 | u8 | Unsigned 8-bit integer |
| U | 2 | u16 | Unsigned 16-bit integer |
| U | 4 | u32 | Unsigned 32-bit integer |
| F | 4 | f32 | 32-bit floating point |
| F | 8 | f64 | 64-bit floating point |

## Special Fields

### RGB/RGBA Fields
RGB color can be stored as:
- Three separate fields: `r g b` (each as U8)
- Single packed field: `rgb` or `rgba` (as F32 or U32)

### Normal Vectors
Typically stored as:
- `normal_x normal_y normal_z` (each as F32)
- Optional: `curvature` (F32)

### Invalid Points
- NaN values indicate invalid/missing data
- Particularly important for organized clouds

## Organized vs Unorganized Clouds

### Unorganized Point Clouds
- HEIGHT = 1
- WIDTH = total number of points
- Points in no particular order
- Example: LiDAR scans

### Organized Point Clouds
- HEIGHT > 1
- WIDTH × HEIGHT = total points
- Preserves 2D structure
- Example: RGB-D camera data
- Invalid points marked with NaN

## Binary Data Encoding

### Binary Format
- Direct memory dump of point array
- System endianness (typically little-endian)
- No padding between points
- Fields packed according to SIZE

### Binary Compressed Format
- LZF compression algorithm
- Format: `<compressed_size><uncompressed_size><compressed_data>`
- Sizes as 32-bit unsigned integers
- Decompression required before parsing

## Example Files

### Minimal ASCII Example
```
VERSION 0.7
FIELDS x y z
SIZE 4 4 4
TYPE F F F
COUNT 1 1 1
WIDTH 3
HEIGHT 1
VIEWPOINT 0 0 0 1 0 0 0
POINTS 3
DATA ascii
1.0 2.0 3.0
4.0 5.0 6.0
7.0 8.0 9.0
```

### Organized Cloud Example
```
VERSION 0.7
FIELDS x y z rgb
SIZE 4 4 4 4
TYPE F F F U
COUNT 1 1 1 1
WIDTH 640
HEIGHT 480
VIEWPOINT 0 0 0 1 0 0 0
POINTS 307200
DATA binary
[binary data follows]
```

## Best Practices

1. **Field Ordering**: Keep xyz fields first for compatibility
2. **Data Alignment**: Align fields to natural boundaries
3. **Compression**: Use for large clouds with redundant data
4. **Validation**: Always verify POINTS = WIDTH × HEIGHT
5. **NaN Handling**: Use for invalid points in organized clouds