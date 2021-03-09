# Revised MagicaVoxel .vox File Format Specification

Adapted from the Specifications housed 
[here](https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox.txt) 
and [here](https://github.com/ephtracy/voxel-model/blob/master/MagicaVoxel-file-format-vox-extension.txt)
as well as personal investigations of the file format.

## File Format Basics

The Magica Voxel File format uses a Chunked schema to organize it's data. Sequence order does matter for both reading
and writing as chunks must be processed in a specific order for data to be associated correctly. 

### File Structure

| Type                          | Notes              |
|-------------------------------|--------------------|
| [Header](#File Header)        | RIFF Header        |
| [Chunk::MAIN](#MAIN Chunk)    | MAIN Chunk         | 

### File Header

Based on the Resource Interchange File Format Container.

| Type        | Literal    | Value
|-------------|------------|--------------------------------------
| [u32](#u32) | 0x564F5820 | RIFF Type _'V', 'O', 'X', 'space'_
| [u32](#u32) | 0x00000096 | Version Number 150

### Chunk Structure

Common Chunk Layout

| Type            | Value
|-----------------|-----------------------------------
| [u32](#u32)     | Chunk Type
| [u32](#u32)     | Size of chunk content in bytes (N)
| [u32](#u32)     | Size of child content in bytes (M)
| [[u8](#u8);N]   | Chunk content
| [CHUNK](#Chunk) | Children chunks (M bytes)

Chunk Types

| Type | ID         | Description 
|------|------------|----------------------------
| MAIN | 0x4D41494E | [MAIN Chunk](#MAIN Chunk)
| PACK | 0x5041434B | [PACK Chunk](#PACK Chunk) 
| SIZE | 0x53495A45 | [SIZE Chunk](#SIZE Chunk)
| XYZI | 0x58595A49 | [XYZI Chunk](#XYZI Chunk)
| RGBA | 0x52474241 | [RGBA Chunk](#RGBA Chunk)
| MATT | 0x4D415454 | [MATT Chunk](#MATT Chunk)
| MATL | 0x4D41544C | [MATL Chunk](#MATL Chunk)
| rOBJ | 0x724F424A | [rOBJ Chunk](#rOBJ Chunk)
| IMAP | 0x494D4150 | [IMAP Chunk](#IMAP Chunk)
| NOTE | 0x4E4F5445 | [NOTE Chunk](#NOTE Chunk)
| nTRN | 0x6E54524E | [nTRN Chunk](#nTRN Chunk)
| nGRP | 0x6E475250 | [nGRP Chunk](#nGRP Chunk)
| nSHP | 0x6E534850 | [nSHP Chunk](#nSHP Chunk)
| LAYR | 0x4C415952 | [LAYR Chunk](#LAYR Chunk)

## Chunk Formats

### MAIN Chunk

* __Description:__ Container Chunk used to house all other Chunks
* __Content:__ None
* __Children:__ The root chunk is the parent chunk of all the other chunks.

```
Chunk 'MAIN'
{
    // pack of models
    Chunk 'PACK'    : optional

    // models
    Chunk 'SIZE'
    Chunk 'XYZI'

    Chunk 'SIZE'
    Chunk 'XYZI'

    ...

    Chunk 'SIZE'
    Chunk 'XYZI'

    // palette
    Chunk 'RGBA'    : optional

    // materials
    Chunk 'MATT'    : optional
    Chunk 'MATT'
    ...
    Chunk 'MATT'
}
```

### PACK Chunk

Indicates the number of Models in the file. If missing only one model should be present.

| Type        | Value
|-------------|-------------------------------------------
| [u32](#u32) | numModels : number of SIZE and XYZI chunks

### SIZE Chunk

Size of a model in X, Y, Z dimensions. There can be multiple SIZE and XYZI chunks for multiple models; model id is their index in the stored order.

| Bytes | Type        | Value
|-------|-------------|---------------------------------------------------------------
| 4     | [u32](#u32) | X Width
| 4     | [u32](#u32) | Y Depth
| 4     | [u32](#u32) | Z Height (Gravity Direction)

### XYZI Chunk

Volumetric Pixels that make up this model. There can be multiple SIZE and XYZI chunks for multiple models; model id is their index in the stored order.

| Type            | Value
|-----------------|---------------------------------------------------------------
| [u32](#u32)     | numVoxels (N)
| [[u32](#u32);N] | (x, y, z, colorIndex) : 1 byte for each component


### RGBA Chunk

Indexed Color Palette. The palette chunk should always stored into the file, so default palette is not needed any more.

NOTICE: color [0-254] are mapped to palette index [1-255], e.g :
```c
for ( int i = 0; i <= 254; i++ ) { 
    palette[i + 1] = ReadRGBA();
}
```

| Type                                                | Value
|-----------------------------------------------------|--------------------------------------------
| [([u8](#u8), [u8](#u8), [u8](#u8), [u8](#u8)); 256] | (Red, Green, Blue, Alpha)

-------------------------------------------------------------------------------
Default Palette if one not provided (Deprecated in v2.0.0+)
```c
unsigned int default_palette[256] = { 
    0x00000000, 0xffffffff, 0xffccffff, 0xff99ffff, 0xff66ffff, 0xff33ffff, 0xff00ffff, 0xffffccff, 
    0xffccccff, 0xff99ccff, 0xff66ccff, 0xff33ccff, 0xff00ccff, 0xffff99ff, 0xffcc99ff, 0xff9999ff, 
    0xff6699ff, 0xff3399ff, 0xff0099ff, 0xffff66ff, 0xffcc66ff, 0xff9966ff, 0xff6666ff, 0xff3366ff, 
    0xff0066ff, 0xffff33ff, 0xffcc33ff, 0xff9933ff, 0xff6633ff, 0xff3333ff, 0xff0033ff, 0xffff00ff, 
    0xffcc00ff, 0xff9900ff, 0xff6600ff, 0xff3300ff, 0xff0000ff, 0xffffffcc, 0xffccffcc, 0xff99ffcc,
    0xff66ffcc, 0xff33ffcc, 0xff00ffcc, 0xffffcccc, 0xffcccccc, 0xff99cccc, 0xff66cccc, 0xff33cccc,
    0xff00cccc, 0xffff99cc, 0xffcc99cc, 0xff9999cc, 0xff6699cc, 0xff3399cc, 0xff0099cc, 0xffff66cc,
    0xffcc66cc, 0xff9966cc, 0xff6666cc, 0xff3366cc, 0xff0066cc, 0xffff33cc, 0xffcc33cc, 0xff9933cc, 
    0xff6633cc, 0xff3333cc, 0xff0033cc, 0xffff00cc, 0xffcc00cc, 0xff9900cc, 0xff6600cc, 0xff3300cc,
    0xff0000cc, 0xffffff99, 0xffccff99, 0xff99ff99, 0xff66ff99, 0xff33ff99, 0xff00ff99, 0xffffcc99,
    0xffcccc99, 0xff99cc99, 0xff66cc99, 0xff33cc99, 0xff00cc99, 0xffff9999, 0xffcc9999, 0xff999999,
    0xff669999, 0xff339999, 0xff009999, 0xffff6699, 0xffcc6699, 0xff996699, 0xff666699, 0xff336699,
    0xff006699, 0xffff3399, 0xffcc3399, 0xff993399, 0xff663399, 0xff333399, 0xff003399, 0xffff0099,
    0xffcc0099, 0xff990099, 0xff660099, 0xff330099, 0xff000099, 0xffffff66, 0xffccff66, 0xff99ff66,
    0xff66ff66, 0xff33ff66, 0xff00ff66, 0xffffcc66, 0xffcccc66, 0xff99cc66, 0xff66cc66, 0xff33cc66,
    0xff00cc66, 0xffff9966, 0xffcc9966, 0xff999966, 0xff669966, 0xff339966, 0xff009966, 0xffff6666,
    0xffcc6666, 0xff996666, 0xff666666, 0xff336666, 0xff006666, 0xffff3366, 0xffcc3366, 0xff993366,
    0xff663366, 0xff333366, 0xff003366, 0xffff0066, 0xffcc0066, 0xff990066, 0xff660066, 0xff330066,
    0xff000066, 0xffffff33, 0xffccff33, 0xff99ff33, 0xff66ff33, 0xff33ff33, 0xff00ff33, 0xffffcc33,
    0xffcccc33, 0xff99cc33, 0xff66cc33, 0xff33cc33, 0xff00cc33, 0xffff9933, 0xffcc9933, 0xff999933,
    0xff669933, 0xff339933, 0xff009933, 0xffff6633, 0xffcc6633, 0xff996633, 0xff666633, 0xff336633,
    0xff006633, 0xffff3333, 0xffcc3333, 0xff993333, 0xff663333, 0xff333333, 0xff003333, 0xffff0033,
    0xffcc0033, 0xff990033, 0xff660033, 0xff330033, 0xff000033, 0xffffff00, 0xffccff00, 0xff99ff00,
    0xff66ff00, 0xff33ff00, 0xff00ff00, 0xffffcc00, 0xffcccc00, 0xff99cc00, 0xff66cc00, 0xff33cc00,
    0xff00cc00, 0xffff9900, 0xffcc9900, 0xff999900, 0xff669900, 0xff339900, 0xff009900, 0xffff6600,
    0xffcc6600, 0xff996600, 0xff666600, 0xff336600, 0xff006600, 0xffff3300, 0xffcc3300, 0xff993300,
    0xff663300, 0xff333300, 0xff003300, 0xffff0000, 0xffcc0000, 0xff990000, 0xff660000, 0xff330000,
    0xff0000ee, 0xff0000dd, 0xff0000bb, 0xff0000aa, 0xff000088, 0xff000077, 0xff000055, 0xff000044,
    0xff000022, 0xff000011, 0xff00ee00, 0xff00dd00, 0xff00bb00, 0xff00aa00, 0xff008800, 0xff007700,
    0xff005500, 0xff004400, 0xff002200, 0xff001100, 0xffee0000, 0xffdd0000, 0xffbb0000, 0xffaa0000,
    0xff880000, 0xff770000, 0xff550000, 0xff440000, 0xff220000, 0xff110000, 0xffeeeeee, 0xffdddddd,
    0xffbbbbbb, 0xffaaaaaa, 0xff888888, 0xff777777, 0xff555555, 0xff444444, 0xff222222, 0xff111111
};
```

### MATT Chunk

Material, if it is absent, it is diffuse material. Deprecated in v2.0.0 in favor of [MATL Chunk](#MATL Chunk)

| Type             | Value                                                                
|------------------|----------------------------------------------------------------------
| [u32](#u32)      | Material Id [1-255]                                                   
| [u32](#u32)      | [Material Type](#Material Type)                                   
| [u32](#u32)      | [Material Weight](#Material Weight)                                  
| [u32](#u32)      | [Property bits](#Property Bits) Set if value is saved in next section (Each bit N+=1)
| [[f32](#f32); N] | Normalized Property Value in Range of (0.0 - 1.0]. Must map to real range. Plastic material only accepts {0.0, 1.0} for this version.

#### Material Type

| TypeId | Type     |
|--------|----------|
| 0      | Diffuse  |
| 1      | Metal    |
| 2      | Glass    |
| 3      | Emissive |

#### Material Weight
| TypeId | Range       | Description                              |
|--------|-------------|------------------------------------------|
| 0      | 1.0         | Ignored and assumed to be 1.0            |
| 1      | (0.0 - 1.0] | Blend between metal and diffuse material |
| 2      | (0.0 - 1.0] | Blend between glass and diffuse material |
| 3      | (0.0 - 1.0] | Self-illuminated material                |

#### Property Bits

| Bit    | Description              |
|--------|--------------------------|
| bit(0) | Plastic                  |
| bit(1) | Roughness                |
| bit(2) | Specular                 |
| bit(3) | IOR                      |
| bit(4) | Attenuation              |
| bit(5) | Power                    |
| bit(6) | Glow                     |
| bit(7) | isTotalPower (*no value) |

### MATL Chunk

Second Version of a Material

| Type             | Value
|------------------|----------------------------------------------------------------------
| [u32](#u32)      | Material Id
| [DICT](#DICT)    | [Material Properties](#Material Properties)

#### Material Properties

All keys and values in Material Properties are stored as a [STRING](#STRING).

| Property | Description
|----------|---------------------------------------------------------------------
| _type    | "_diffuse", "_metal", "_glass", or "_emit" 
| _weight  | (0.0 - 1.0]
| _rough   | Roughness
| _spec    | Specular Reflection (Float)
| _ior     | Index of Refraction (Float)
| _att     | Light Attenuation (Float)
| _flux    | Light Flux (Float)
| _plastic |

### nTRN Chunk

Scene Graph - Transform Node

| Type                 | Description
|----------------------|---------------------------------------------------------
| [u32](#u32)          | Node Id
| [DICT](#DICT Type)   | Node Attributes
| [u32](#u32)          | Child Node id
| [u32](#u32)          | Reserved Id [Must be 0xFFFFFFFF]
| [u32](#u32)          | Layer id
| [u32](#u32)          | Number of Frames (N)
| [[DICT](#DICT); N]   | [Frame Transform](#Frame Transform)

Current Version is limited to 1 Frame.

#### FRAME

| Property | Description
|----------|------------------
| _r       | [Rotation](#Rotation)
| -t       | [Translation](#Translation)

#### Rotation

| Type      | Description
|-----------|--------------------
| [u8](#u8) | Rotation "Matrix"

store a row-major rotation in the bits of a byte

Example: 
```
R =
  0  1  0
  0  0 -1
 -1  0  0
```
becomes
```c
unsigned char _r = (1 << 0) | (2 << 2) | (0 << 4) | (1 << 5) | (1 << 6)
```
| bit | value
|-----|--------------------------------------------------------------
| 0-1 | 1 : index of the non-zero entry in the first row
| 2-3 | 2 : index of the non-zero entry in the second row
| 4   | 0 : the sign in the first row (0 : positive; 1 : negative)
| 5   | 1 : the sign in the second row (0 : positive; 1 : negative)
| 6   | 1 : the sign in the third row (0 : positive; 1 : negative)

#### Translation

| Type        | Description
|-------------|---------------------------------------------------------
| [u32](#u32) | X Translation
| [u32](#u32) | Y Translation
| [u32](#u32) | Z Translation

### nGRP Chunk

Scene Graph - Group Node

| Type                 | Description
|----------------------|---------------------------------------------------------
| [u32](#u32)          | Node Id
| [DICT](#DICT)   | Node Attributes
| [u32](#u32)          | Number of Child Nodes (N)
| [[u32](#u32); N]     | Child Node Ids 

### nSHP Chunk

Scene Graph - Shape Node

| Type                 | Description
|----------------------|---------------------------------------------------------
| [u32](#u32)          | Node Id
| [DICT](#DICT)        | Node Attributes
| [u32](#u32)          | Number of Models (N)
| [[Model](#Model); N] | Models

Note: Number of Models MUST be 1

#### Model

| Type                 | Description
|----------------------|---------------------------------------------------------
| [u32](#u32)          | Model Id
| [DICT](#DICT)        | Model Attributes

### LAYR Chunk

| Type                 | Description
|----------------------|---------------------------------------------------------
| [u32](#u32)          | Layer Id
| [DICT](#DICT)        | Layer Attributes
| [u32](#u32)          | Reserved Id [Must be 0xFFFFFFFF]

#### Layer Attributes

| Attribute | Description
|-----------|---------------------------------------------------------------------
| _hidden   | 0/1

### rOBJ Chunk

Render Objects Chunk

| Type          | Description
|---------------|---------------------------------------------------------
| [DICT](#DICT) | Rendering Attributes

### rCAM Chunk

Render Camera Chunk

| Type          | Description
|---------------|---------------------------------------------------------
| [u32](#u32)   | Camera Id
| [DICT](#DICT) | [Camera Attributes](#Camera Attributes)

#### Camera Attributes

| Attribute | Description
|-----------|---------------------------------------------------------------------
| _mode     | String.
| _focus    | String. 'X Y Z'
| _angle    | String. 'X Y Z'
| _radius   | String. int
| _frustum  | String. float
| _fov      | String. int

### NOTE Chunk

Palette Note Chunk

| Type          | Description
|---------------|---------------------------------------------------------
| [DICT](#DICT) | Rendering Attributes


## Scene Graph

```
T : Transform Node
G : Group Node
S : Shape Node

     T
     |
     G
    / \
   T   T
   |   |
   G   S
  / \
 T   T
 |   |
 S   S
```

## Notes

* there can be multiple SIZE and XYZI chunks for multiple models; model id is their index in the stored order
* the MATT chunk is deprecated, replaced by the MATL chunk, see (4)
* (a), (b), (c) are special data types; (d) is the scene graph in the world editor
