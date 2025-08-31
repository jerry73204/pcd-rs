# Header Format

The PCD header contains essential metadata about the point cloud. This chapter provides detailed information about each header field.

## Header Structure

The PCD header is ASCII text with one field per line, in a **strict order**:

```
VERSION 0.7
FIELDS x y z rgb
SIZE 4 4 4 4
TYPE F F F U
COUNT 1 1 1 1
WIDTH 307200
HEIGHT 1
VIEWPOINT 0 0 0 1 0 0 0
POINTS 307200
DATA binary
```

## Field Details

### VERSION

**Format:** `VERSION <version_number>`

**Description:** Specifies the PCD file format version.

**Required:** Yes

**Valid Values:**
- `0.7` - Current standard (supported)
- `0.6` - Legacy (not yet supported)
- `0.5` - Legacy (not yet supported)

**Example:**
```
VERSION 0.7
```

### FIELDS

**Format:** `FIELDS <name1> <name2> ... <nameN>`

**Description:** Names of each dimension/field in the point data.

**Required:** Yes

**Common Field Names:**
- Geometry: `x`, `y`, `z`
- Color: `rgb`, `rgba`, `r`, `g`, `b`, `a`
- Normals: `normal_x`, `normal_y`, `normal_z`
- Other: `intensity`, `label`, `curvature`
- Padding: `_` (ignored field)

**Examples:**
```
FIELDS x y z
FIELDS x y z rgb
FIELDS x y z normal_x normal_y normal_z curvature
FIELDS x y z _ intensity _
```

**Special Cases:**
- `_` indicates padding/ignored fields
- Custom names are allowed
- Order matters for binary format

### SIZE

**Format:** `SIZE <size1> <size2> ... <sizeN>`

**Description:** Size in bytes of each field's data type.

**Required:** Yes

**Valid Values:**
| Size | Types |
|------|-------|
| 1 | i8, u8 |
| 2 | i16, u16 |
| 4 | i32, u32, f32 |
| 8 | f64 |

**Example:**
```
SIZE 4 4 4 4    # Four 4-byte fields
SIZE 4 4 4 1 1 1  # Three floats, three bytes
```

**Validation:**
- Must match the number of FIELDS
- Must correspond to valid TYPE entries

### TYPE

**Format:** `TYPE <type1> <type2> ... <typeN>`

**Description:** Data type of each field.

**Required:** Yes

**Valid Values:**
| Character | Meaning | Rust Types |
|-----------|---------|------------|
| I | Signed integer | i8, i16, i32 |
| U | Unsigned integer | u8, u16, u32 |
| F | Floating point | f32, f64 |

**Examples:**
```
TYPE F F F      # Three floats
TYPE F F F U    # Three floats, one unsigned
TYPE I I I F    # Three signed ints, one float
```

**Type-Size Combinations:**
| TYPE | SIZE | Rust Type |
|------|------|-----------|
| I | 1 | i8 |
| I | 2 | i16 |
| I | 4 | i32 |
| U | 1 | u8 |
| U | 2 | u16 |
| U | 4 | u32 |
| F | 4 | f32 |
| F | 8 | f64 |

### COUNT

**Format:** `COUNT <count1> <count2> ... <countN>`

**Description:** Number of elements in each field (for array fields).

**Required:** Yes

**Default:** 1 (scalar field)

**Examples:**
```
COUNT 1 1 1           # Three scalar fields
COUNT 3 1             # One 3-element array, one scalar
COUNT 1 1 1 128       # Three scalars, one 128-element array
```

**Use Cases:**
- `COUNT 3` for position/normal vectors
- `COUNT 128` for feature descriptors
- `COUNT 1` for scalar values

### WIDTH

**Format:** `WIDTH <width>`

**Description:** Width of the point cloud dataset.

**Required:** Yes

**Interpretation:**
- **Unorganized clouds:** Total number of points in the cloud
- **Organized clouds:** Width of the image/matrix structure

**Examples:**
```
WIDTH 307200    # Unorganized: 307,200 points
WIDTH 640       # Organized: 640 pixels wide
```

### HEIGHT

**Format:** `HEIGHT <height>`

**Description:** Height of the point cloud dataset.

**Required:** Yes

**Interpretation:**
- **Unorganized clouds:** Always 1
- **Organized clouds:** Height of the image/matrix structure

**Examples:**
```
HEIGHT 1        # Unorganized cloud
HEIGHT 480      # Organized: 480 pixels high
```

**Organized vs Unorganized:**
```
Unorganized: WIDTH=N, HEIGHT=1 (N total points)
Organized: WIDTH=W, HEIGHT=H (W×H total points)
```

### VIEWPOINT

**Format:** `VIEWPOINT <tx> <ty> <tz> <qw> <qx> <qy> <qz>`

**Description:** Sensor acquisition pose as translation + quaternion.

**Required:** Yes

**Components:**
- `tx, ty, tz`: Translation (position) in meters
- `qw, qx, qy, qz`: Quaternion (orientation), where qw is the scalar part

**Default:** `0 0 0 1 0 0 0` (identity transform)

**Examples:**
```
VIEWPOINT 0 0 0 1 0 0 0              # Default/identity
VIEWPOINT 1.5 0 2.0 1 0 0 0          # Translated to (1.5, 0, 2.0)
VIEWPOINT 0 0 0 0.707 0 0.707 0      # 90° rotation around Y axis
```

**Quaternion Properties:**
- Must be normalized: qw² + qx² + qy² + qz² = 1
- Identity rotation: qw=1, qx=qy=qz=0

### POINTS

**Format:** `POINTS <count>`

**Description:** Total number of points in the cloud.

**Required:** Yes

**Validation:** Must equal WIDTH × HEIGHT

**Examples:**
```
POINTS 307200    # Must match WIDTH × HEIGHT
POINTS 1000      # 1000 points total
```

### DATA

**Format:** `DATA <format>`

**Description:** Data storage format for the point cloud.

**Required:** Yes

**Valid Values:**
| Format | Description | Use Case |
|--------|-------------|----------|
| `ascii` | Human-readable text | Debugging, small files |
| `binary` | Raw binary data | Performance, accuracy |
| `binary_compressed` | LZF compressed | Large files, bandwidth |

**Examples:**
```
DATA ascii              # Text format
DATA binary             # Raw binary
DATA binary_compressed  # Compressed
```

## Complete Header Example

### Minimal Header
```
VERSION 0.7
FIELDS x y z
SIZE 4 4 4
TYPE F F F
COUNT 1 1 1
WIDTH 100
HEIGHT 1
VIEWPOINT 0 0 0 1 0 0 0
POINTS 100
DATA ascii
```

### Complex Header
```
VERSION 0.7
FIELDS x y z rgb normal_x normal_y normal_z curvature
SIZE 4 4 4 4 4 4 4 4
TYPE F F F U F F F F
COUNT 1 1 1 1 1 1 1 1
WIDTH 640
HEIGHT 480
VIEWPOINT 1.0 0.5 2.0 0.924 0 0.383 0
POINTS 307200
DATA binary
```

## Parsing Rules

1. **Line-by-line parsing**: Each field on its own line
2. **Strict ordering**: Fields must appear in the specified order
3. **No comments**: Comments after `#` are not standard
4. **Space separation**: Values separated by spaces
5. **Case sensitivity**: Field names are case-sensitive

## Common Pitfalls

### Incorrect Field Order
```
# WRONG - fields out of order
VERSION 0.7
WIDTH 100
FIELDS x y z  # Should come before WIDTH
```

### Mismatched Counts
```
# WRONG - SIZE count doesn't match FIELDS
FIELDS x y z
SIZE 4 4  # Missing one size value
```

### Invalid Point Count
```
# WRONG - POINTS doesn't match WIDTH × HEIGHT
WIDTH 100
HEIGHT 10
POINTS 999  # Should be 1000
```

## Best Practices

1. **Always validate**: Check WIDTH × HEIGHT = POINTS
2. **Use standard names**: Prefer `x y z` over custom names
3. **Binary for performance**: Use binary format for large clouds
4. **Normalize quaternions**: Ensure valid rotation representation
5. **Document custom fields**: Comment or document non-standard field names