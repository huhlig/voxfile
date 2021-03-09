//
// Copyright 2021 Hans W. Uhlig. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::collections::HashMap;
use nom::lib::std::fmt::Formatter;

const MAGIC_NUMBER: &'static str = "VOX ";

/// (String, String) Dictionary
pub type Dict = HashMap<String, String>;

/// RGBA 32 bit color
#[derive(Clone, Debug)]
pub struct Color {
    pub name: Option<String>,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn from_u32(val: u32) -> Self {
        Color {
            name: None,
            r: ((val & 0xFF000000) >> 24) as u8,
            g: ((val & 0x00FF0000) >> 16) as u8,
            b: ((val & 0x0000FF00) >> 08) as u8,
            a: ((val & 0x000000FF) >> 00) as u8,
        }
    }
}

/// TODO: Figure out how to parse this.
///
/// (c) ROTATION type
///
/// store a row-major rotation in the bits of a byte
///
/// for example :
/// R =
///  0  1  0
///  0  0 -1
/// -1  0  0
/// ==>
/// unsigned char _r = (1 << 0) | (2 << 2) | (0 << 4) | (1 << 5) | (1 << 6)
///
/// bit | value
/// 0-1 : 1 : index of the non-zero entry in the first row
/// 2-3 : 2 : index of the non-zero entry in the second row
/// 4   : 0 : the sign in the first row (0 : positive; 1 : negative)
/// 5   : 1 : the sign in the second row (0 : positive; 1 : negative)
/// 6   : 1 : the sign in the third row (0 : positive; 1 : negative)
///
#[derive(Clone, Debug)]
pub struct Rotation(pub u8);

impl Rotation {
    /*
    pub fn from_matrix(matrix: [[f32; 3]; 3]) -> Rotation {
        let mut result = 0u8;
        //which idx of row has the +/- 1
        let idx0 = 0;
        let idx1 = 0;
        let idx2 = 0;

        /* Get idx0,1,2 correct here */

        result |= (idx0 << 0);
        result |= (idx1 << 2);
        result |= ((matrix[0][idx0 as usize] == -1.0) << 4);
        result |= ((matrix[1][idx1 as usize] == -1.0) << 5);
        result |= ((matrix[2][idx2 as usize] == -1.0) << 6);

        Rotation(result)
    }
     */
    pub fn to_matrix(&self) -> [[f32; 3]; 3] {
        let mut result = [[0.0; 3]; 3];
        let (_01, _23, _4, _5, _6) = (
            0x03 & self.0 >> 0,
            0x0C & self.0 >> 2,
            0x10 & self.0 >> 3,
            0x20 & self.0 >> 4,
            0x40 & self.0 >> 5,
        );
        let idx0 = _01;
        let idx1 = _23;
        let idx2 = 3 - idx0 - idx1;

        result[0][idx0 as usize] = 1.0 - _4 as f32 * 2.0;
        result[1][idx1 as usize] = 1.0 - _5 as f32 * 2.0;
        result[2][idx2 as usize] = 1.0 - _6 as f32 * 2.0;
        result
    }
}

/// Container for .vox file data
#[derive(Clone, Debug)]
pub struct VoxFile {
    /// Version number of the .vox file
    pub version: u32,
    /// A Vec of all models contained in this file.
    pub models: Vec<Model>,
    /// A Vec containing the color palette.
    pub palette: [Color; 256],
    /// A Vec containing all the Materials in this file.
    pub materials: Vec<Material>,
    /// A Scene Graph
    pub scenegraph: Vec<SceneNode>,
}

impl VoxFile {}

impl Default for VoxFile {
    fn default() -> VoxFile {
        VoxFile {
            version: 150,
            models: Vec::new(),
            palette: DEFAULT_PALETTE.clone(),
            materials: Vec::new(),
            scenegraph: Vec::new(),
        }
    }
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Chunk {
    /// Main Chunk, No Content, Many Children
    MAIN(Vec<Chunk>),
    /// Size Chunk, Multiple Content
    SIZE(Size),
    /// XYZI Chunk,
    XYZI(Vec<Voxel>),
    /// Pack Chunk
    PACK(Pack),
    /// Color Palette
    RGBA(Vec<Color>),
    /// V1 Material
    MATT(MaterialV1),
    /// V2 Material
    MATL(MaterialV2),
    /// Unknown
    rOBJ(Dict),
    /// Unknown
    rCAM(Camera),
    /// Unknown
    IMAP(Vec<u8>),
    /// Unknown
    NOTE(Vec<String>),
    /// Scene Graph Transform Node
    nTRN(TransformNode),
    /// Scene Graph Group Node
    nGRP(GroupNode),
    /// Scene Graph Shape Node
    nSHP(ShapeNode),
    /// Layer Node
    LAYR(Layer),
    /// Unknown Node
    Unknown {
        kind: String,
        contents: Vec<u8>,
        children: Vec<Chunk>,
    },
}

#[derive(Clone, Debug)]
pub struct Pack(pub u32);

/// A Sparse Volumetric Pixel Model.
///
/// Sparse Voxel Models store each voxel as an (x,y,z) point in space and a palette index.
#[derive(Clone, Debug)]
pub struct Model {
    pub id: u32,
    /// The size of the model in voxels.
    pub size: Size,
    /// The list of Voxels in the model.
    pub voxels: Vec<Voxel>,
}

/// The size of a model in voxels.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Size {
    /// The width of the model in voxels.
    pub x: u32,
    /// The height of the model in voxels.
    pub y: u32,
    /// The depth of the model in voxels.
    pub z: u32,
}

/// A sparse volumetric pixel.
#[derive(Clone, Debug)]
pub struct Voxel {
    /// The X coordinate of the voxel.
    pub x: u8,
    /// The Y coordinate of the voxel.
    pub y: u8,
    /// The Z coordinate of the voxel.
    pub z: u8,
    /// Index in the Color Palette.
    ///
    /// Note: This value will be 1 less than the value stored in the file. The .vox file format uses
    /// index ranges from 1-255 where memory array indexing is from 0-254.
    pub i: u8,
}

/// A material used to render this model.
#[derive(Clone, Debug, PartialEq)]
pub enum Material {
    V1(MaterialV1),
    V2(MaterialV2),
}

/// A material used to render this model.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialV1 {
    /// The Material's ID
    pub id: u32,
    /// Material Type.
    /// 0 - Diffuse
    /// 1 - Metal
    /// 2 - Glass
    /// 3 - Emissive
    pub kind: u32,
    /// Material Weight
    /// diffuse  : 1.0
    /// metal    : (0.0 - 1.0] Blend between metal and diffuse material
    /// glass    : (0.0 - 1.0] Blend between Glass and diffuse material
    /// emissive : (0.0 - 1.0] self illuminated material
    pub weight: f32,
    /// Normalized Property Value for Plastic
    pub plastic: Option<f32>,
    /// Normalized Property Value for Roughness
    pub roughness: Option<f32>,
    /// Normalized Property Value for Specular
    pub specular: Option<f32>,
    /// Normalized Property Value for IOR
    pub ior: Option<f32>,
    /// Normalized Property Value for Attenuation
    pub attenuation: Option<f32>,
    /// Normalized Property Value for Power
    pub power: Option<f32>,
    /// Normalized Property Value for Glow
    pub glow: Option<f32>,
    /// Property set
    pub is_total_power: bool,
}

/// A material used to render this model.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialV2 {
    /// The Material's ID
    pub id: u32,
    /// Properties of the material, mapped by property name.
    ///
    pub properties: Dict,
}

#[derive(Clone, Debug)]
pub enum SceneNode {
    Transform(TransformNode),
    Group(GroupNode),
    Shape(ShapeNode),
}

#[derive(Clone, Debug)]
pub struct TransformNode {
    pub id: u32,
    pub attrib: Dict,
    pub child_node_id: u32,
    pub reserved_id: i32,
    pub layer_id: u32,
    pub frames: Vec<Dict>,
}

#[derive(Clone, Debug)]
pub struct GroupNode {
    pub id: u32,
    pub attrib: Dict,
    pub children: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct ShapeNode {
    pub id: u32,
    pub attrib: Dict,
    pub models: Vec<(u32, Dict)>,
}

/// (5) Layer Chunk
#[derive(Clone, Debug)]
pub struct Layer {
    pub id: u32,
    pub attributes: HashMap<String, String>,
    pub reserved: i32,
}

/// Camera
#[derive(Clone, Debug)]
pub struct Camera {
    pub id: u32,
    pub attributes: HashMap<String,String>,
}

const DEFAULT_PALETTE: [Color; 256] = [
    Color::from_u32(0x00000000), Color::from_u32(0xffffffff), Color::from_u32(0xffccffff), Color::from_u32(0xff99ffff),
    Color::from_u32(0xff66ffff), Color::from_u32(0xff33ffff), Color::from_u32(0xff00ffff), Color::from_u32(0xffffccff),
    Color::from_u32(0xffccccff), Color::from_u32(0xff99ccff), Color::from_u32(0xff66ccff), Color::from_u32(0xff33ccff),
    Color::from_u32(0xff00ccff), Color::from_u32(0xffff99ff), Color::from_u32(0xffcc99ff), Color::from_u32(0xff9999ff),
    Color::from_u32(0xff6699ff), Color::from_u32(0xff3399ff), Color::from_u32(0xff0099ff), Color::from_u32(0xffff66ff),
    Color::from_u32(0xffcc66ff), Color::from_u32(0xff9966ff), Color::from_u32(0xff6666ff), Color::from_u32(0xff3366ff),
    Color::from_u32(0xff0066ff), Color::from_u32(0xffff33ff), Color::from_u32(0xffcc33ff), Color::from_u32(0xff9933ff),
    Color::from_u32(0xff6633ff), Color::from_u32(0xff3333ff), Color::from_u32(0xff0033ff), Color::from_u32(0xffff00ff),
    Color::from_u32(0xffcc00ff), Color::from_u32(0xff9900ff), Color::from_u32(0xff6600ff), Color::from_u32(0xff3300ff),
    Color::from_u32(0xff0000ff), Color::from_u32(0xffffffcc), Color::from_u32(0xffccffcc), Color::from_u32(0xff99ffcc),
    Color::from_u32(0xff66ffcc), Color::from_u32(0xff33ffcc), Color::from_u32(0xff00ffcc), Color::from_u32(0xffffcccc),
    Color::from_u32(0xffcccccc), Color::from_u32(0xff99cccc), Color::from_u32(0xff66cccc), Color::from_u32(0xff33cccc),
    Color::from_u32(0xff00cccc), Color::from_u32(0xffff99cc), Color::from_u32(0xffcc99cc), Color::from_u32(0xff9999cc),
    Color::from_u32(0xff6699cc), Color::from_u32(0xff3399cc), Color::from_u32(0xff0099cc), Color::from_u32(0xffff66cc),
    Color::from_u32(0xffcc66cc), Color::from_u32(0xff9966cc), Color::from_u32(0xff6666cc), Color::from_u32(0xff3366cc),
    Color::from_u32(0xff0066cc), Color::from_u32(0xffff33cc), Color::from_u32(0xffcc33cc), Color::from_u32(0xff9933cc),
    Color::from_u32(0xff6633cc), Color::from_u32(0xff3333cc), Color::from_u32(0xff0033cc), Color::from_u32(0xffff00cc),
    Color::from_u32(0xffcc00cc), Color::from_u32(0xff9900cc), Color::from_u32(0xff6600cc), Color::from_u32(0xff3300cc),
    Color::from_u32(0xff0000cc), Color::from_u32(0xffffff99), Color::from_u32(0xffccff99), Color::from_u32(0xff99ff99),
    Color::from_u32(0xff66ff99), Color::from_u32(0xff33ff99), Color::from_u32(0xff00ff99), Color::from_u32(0xffffcc99),
    Color::from_u32(0xffcccc99), Color::from_u32(0xff99cc99), Color::from_u32(0xff66cc99), Color::from_u32(0xff33cc99),
    Color::from_u32(0xff00cc99), Color::from_u32(0xffff9999), Color::from_u32(0xffcc9999), Color::from_u32(0xff999999),
    Color::from_u32(0xff669999), Color::from_u32(0xff339999), Color::from_u32(0xff009999), Color::from_u32(0xffff6699),
    Color::from_u32(0xffcc6699), Color::from_u32(0xff996699), Color::from_u32(0xff666699), Color::from_u32(0xff336699),
    Color::from_u32(0xff006699), Color::from_u32(0xffff3399), Color::from_u32(0xffcc3399), Color::from_u32(0xff993399),
    Color::from_u32(0xff663399), Color::from_u32(0xff333399), Color::from_u32(0xff003399), Color::from_u32(0xffff0099),
    Color::from_u32(0xffcc0099), Color::from_u32(0xff990099), Color::from_u32(0xff660099), Color::from_u32(0xff330099),
    Color::from_u32(0xff000099), Color::from_u32(0xffffff66), Color::from_u32(0xffccff66), Color::from_u32(0xff99ff66),
    Color::from_u32(0xff66ff66), Color::from_u32(0xff33ff66), Color::from_u32(0xff00ff66), Color::from_u32(0xffffcc66),
    Color::from_u32(0xffcccc66), Color::from_u32(0xff99cc66), Color::from_u32(0xff66cc66), Color::from_u32(0xff33cc66),
    Color::from_u32(0xff00cc66), Color::from_u32(0xffff9966), Color::from_u32(0xffcc9966), Color::from_u32(0xff999966),
    Color::from_u32(0xff669966), Color::from_u32(0xff339966), Color::from_u32(0xff009966), Color::from_u32(0xffff6666),
    Color::from_u32(0xffcc6666), Color::from_u32(0xff996666), Color::from_u32(0xff666666), Color::from_u32(0xff336666),
    Color::from_u32(0xff006666), Color::from_u32(0xffff3366), Color::from_u32(0xffcc3366), Color::from_u32(0xff993366),
    Color::from_u32(0xff663366), Color::from_u32(0xff333366), Color::from_u32(0xff003366), Color::from_u32(0xffff0066),
    Color::from_u32(0xffcc0066), Color::from_u32(0xff990066), Color::from_u32(0xff660066), Color::from_u32(0xff330066),
    Color::from_u32(0xff000066), Color::from_u32(0xffffff33), Color::from_u32(0xffccff33), Color::from_u32(0xff99ff33),
    Color::from_u32(0xff66ff33), Color::from_u32(0xff33ff33), Color::from_u32(0xff00ff33), Color::from_u32(0xffffcc33),
    Color::from_u32(0xffcccc33), Color::from_u32(0xff99cc33), Color::from_u32(0xff66cc33), Color::from_u32(0xff33cc33),
    Color::from_u32(0xff00cc33), Color::from_u32(0xffff9933), Color::from_u32(0xffcc9933), Color::from_u32(0xff999933),
    Color::from_u32(0xff669933), Color::from_u32(0xff339933), Color::from_u32(0xff009933), Color::from_u32(0xffff6633),
    Color::from_u32(0xffcc6633), Color::from_u32(0xff996633), Color::from_u32(0xff666633), Color::from_u32(0xff336633),
    Color::from_u32(0xff006633), Color::from_u32(0xffff3333), Color::from_u32(0xffcc3333), Color::from_u32(0xff993333),
    Color::from_u32(0xff663333), Color::from_u32(0xff333333), Color::from_u32(0xff003333), Color::from_u32(0xffff0033),
    Color::from_u32(0xffcc0033), Color::from_u32(0xff990033), Color::from_u32(0xff660033), Color::from_u32(0xff330033),
    Color::from_u32(0xff000033), Color::from_u32(0xffffff00), Color::from_u32(0xffccff00), Color::from_u32(0xff99ff00),
    Color::from_u32(0xff66ff00), Color::from_u32(0xff33ff00), Color::from_u32(0xff00ff00), Color::from_u32(0xffffcc00),
    Color::from_u32(0xffcccc00), Color::from_u32(0xff99cc00), Color::from_u32(0xff66cc00), Color::from_u32(0xff33cc00),
    Color::from_u32(0xff00cc00), Color::from_u32(0xffff9900), Color::from_u32(0xffcc9900), Color::from_u32(0xff999900),
    Color::from_u32(0xff669900), Color::from_u32(0xff339900), Color::from_u32(0xff009900), Color::from_u32(0xffff6600),
    Color::from_u32(0xffcc6600), Color::from_u32(0xff996600), Color::from_u32(0xff666600), Color::from_u32(0xff336600),
    Color::from_u32(0xff006600), Color::from_u32(0xffff3300), Color::from_u32(0xffcc3300), Color::from_u32(0xff993300),
    Color::from_u32(0xff663300), Color::from_u32(0xff333300), Color::from_u32(0xff003300), Color::from_u32(0xffff0000),
    Color::from_u32(0xffcc0000), Color::from_u32(0xff990000), Color::from_u32(0xff660000), Color::from_u32(0xff330000),
    Color::from_u32(0xff0000ee), Color::from_u32(0xff0000dd), Color::from_u32(0xff0000bb), Color::from_u32(0xff0000aa),
    Color::from_u32(0xff000088), Color::from_u32(0xff000077), Color::from_u32(0xff000055), Color::from_u32(0xff000044),
    Color::from_u32(0xff000022), Color::from_u32(0xff000011), Color::from_u32(0xff00ee00), Color::from_u32(0xff00dd00),
    Color::from_u32(0xff00bb00), Color::from_u32(0xff00aa00), Color::from_u32(0xff008800), Color::from_u32(0xff007700),
    Color::from_u32(0xff005500), Color::from_u32(0xff004400), Color::from_u32(0xff002200), Color::from_u32(0xff001100),
    Color::from_u32(0xffee0000), Color::from_u32(0xffdd0000), Color::from_u32(0xffbb0000), Color::from_u32(0xffaa0000),
    Color::from_u32(0xff880000), Color::from_u32(0xff770000), Color::from_u32(0xff550000), Color::from_u32(0xff440000),
    Color::from_u32(0xff220000), Color::from_u32(0xff110000), Color::from_u32(0xffeeeeee), Color::from_u32(0xffdddddd),
    Color::from_u32(0xffbbbbbb), Color::from_u32(0xffaaaaaa), Color::from_u32(0xff888888), Color::from_u32(0xff777777),
    Color::from_u32(0xff555555), Color::from_u32(0xff444444), Color::from_u32(0xff222222), Color::from_u32(0xff111111),
];
