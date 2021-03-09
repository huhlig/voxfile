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

#![allow(non_snake_case)]

use nom::{
    IResult,
    bytes::complete::{
        take,
        tag,
    },
    combinator::{
        map,
        map_res,
    },
    multi::{
        count,
        many0,
    },
    number::complete::{
        le_u8,
        le_u32,
        le_f32,
        le_i32,
    },
    sequence::tuple,
};
use crate::types::*;
use std::iter::FromIterator;
use std::convert::TryInto;

const MAGIC_NUMBER: &'static str = "VOX ";

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


#[tracing::instrument]
fn parse_file(input: &[u8]) -> IResult<&[u8], VoxFile> {
    tracing::trace!("parse_file(len: {})", input.len());
    let (input, _) = tag(MAGIC_NUMBER)(input)?;
    let (input, _version) = le_u32(input)?;
    let (input, main) = parse_chunk(input)?;
    if let Chunk::MAIN(children) = main {
        let mut file = VoxFile::default();
        let mut model_id = 0;
        let mut model_size = None;
        for child in children {
            match child {
                Chunk::MAIN(_) => {
                    tracing::error!("Attempting to process a MAIN chunk inside a MAIN Chunk");
                }
                Chunk::SIZE(size) => {
                    model_size = Some(size);
                }
                Chunk::XYZI(voxels) => {
                    if let Some(size) = model_size {
                        file.models.push(Model { id: model_id, size, voxels });
                        model_id += 1;
                        model_size = None;

                    }
                }
                Chunk::PACK(pack) => {
                    tracing::debug!("Got a pack with value of: {:?}", pack);
                }
                Chunk::RGBA(colors) => {
                    file.palette = colors.try_into().unwrap();
                }
                Chunk::MATT(material) => {
                    file.materials.push(Material::V1(material))
                }
                Chunk::MATL(material) => {
                    file.materials.push(Material::V2(material))
                }
                Chunk::rOBJ(_obj) => {}
                Chunk::rCAM(_cam) => {}
                Chunk::IMAP(_imap) => {}
                Chunk::NOTE(_note) => {}
                Chunk::nTRN(_transform) => {}
                Chunk::nGRP(_group) => {}
                Chunk::nSHP(_shape) => {}
                Chunk::LAYR(_layer) => {}
                Chunk::Unknown { .. } => {}
            }
        }
        Ok((input, file))
    } else {
        Err(nom::Err::Error(nom::error::make_error(input, nom::error::ErrorKind::Eof)))
    }
}

#[tracing::instrument]
fn parse_chunk(input: &[u8]) -> IResult<&[u8], Chunk> {
    let (input, kind) = map_res(take(4usize), std::str::from_utf8)(input)?;
    let (input, content_size) = le_u32(input)?;
    let (input, children_size) = le_u32(input)?;
    let (input, chunk_content) = take(content_size)(input)?;
    let (input, child_content) = take(children_size)(input)?;
    tracing::trace!(
        "parse_chunk({}, Content Size: {}, Child Size: {})",
        kind, content_size, children_size
    );
    println!("{} Content: {} Children Size: {}", kind, content_size, children_size);
    let children = if children_size > 0 {
        many0(parse_chunk)(child_content)?.1
    } else {
        Vec::new()
    };
    Ok((input, match kind {
        "MAIN" => Chunk::MAIN(children),
        "PACK" => Chunk::PACK(parse_PACK(chunk_content)?.1),
        "SIZE" => Chunk::SIZE(parse_SIZE(chunk_content)?.1),
        "XYZI" => Chunk::XYZI(parse_XYZI(chunk_content)?.1),
        "RGBA" => Chunk::RGBA(parse_RGBA(chunk_content)?.1),
        "MATT" => Chunk::MATT(parse_MATT(chunk_content)?.1),
        "MATL" => Chunk::MATL(parse_MATL(chunk_content)?.1),
        "rOBJ" => Chunk::rOBJ(parse_rOBJ(chunk_content)?.1),
        "rCAM" => Chunk::rCAM(parse_rCAM(chunk_content)?.1),
        "IMAP" => Chunk::IMAP(parse_IMAP(chunk_content)?.1),
        "NOTE" => Chunk::NOTE(parse_NOTE(chunk_content)?.1),
        "nTRN" => Chunk::nTRN(parse_nTRN(chunk_content)?.1),
        "nGRP" => Chunk::nGRP(parse_nGRP(chunk_content)?.1),
        "nSHP" => Chunk::nSHP(parse_nSHP(chunk_content)?.1),
        "LAYR" => Chunk::LAYR(parse_LAYR(chunk_content)?.1),
        _ => Chunk::Unknown {
            kind: kind.to_owned(),
            contents: Vec::from(chunk_content),
            children,
        }
    }))
}

#[tracing::instrument]
fn parse_PACK(input: &[u8]) -> IResult<&[u8], Pack> {
    tracing::trace!("parse_PACK(len: {})", input.len());
    let (input, model_count) = le_u32(input)?;
    Ok((input, Pack(model_count)))
}

#[tracing::instrument]
fn parse_SIZE(input: &[u8]) -> IResult<&[u8], Size> {
    tracing::trace!("parse_SIZE(len: {})", input.len());
    let (input, x) = le_u32(input)?;
    let (input, y) = le_u32(input)?;
    let (input, z) = le_u32(input)?;
    Ok((input, Size { x, y, z }))
}

#[tracing::instrument]
fn parse_XYZI(input: &[u8]) -> IResult<&[u8], Vec<Voxel>> {
    tracing::trace!("parse_XYZI(len: {})", input.len());
    let (input, voxel_count) = le_u32(input)?;
    let (input, voxels) = count(|input| {
        let (input, x) = le_u8(input)?;
        let (input, y) = le_u8(input)?;
        let (input, z) = le_u8(input)?;
        let (input, i) = le_u8(input)?;
        Ok((input, Voxel { x, y, z, i }))
    }, voxel_count as usize)(input)?;
    Ok((input, voxels))
}

#[tracing::instrument]
fn parse_RGBA(input: &[u8]) -> IResult<&[u8], Vec<Color>> {
    tracing::trace!("parse_RGBA(len: {})", input.len());
    let (input, colors) = count(|input| {
        let (input, r) = le_u8(input)?;
        let (input, g) = le_u8(input)?;
        let (input, b) = le_u8(input)?;
        let (input, a) = le_u8(input)?;
        Ok((input, Color { name: None, r, g, b, a }))
    }, 256)(input)?;
    Ok((input, colors))
}

#[tracing::instrument]
fn parse_MATT(input: &[u8]) -> IResult<&[u8], MaterialV1> {
    tracing::trace!("parse_MATT(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, kind) = le_u32(input)?;
    let (input, weight) = le_f32(input)?;
    let (input, property_bits) = le_u32(input)?;
    let (input, plastic) = if (property_bits & 0x01) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, roughness) = if (property_bits & 0x02) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, specular) = if (property_bits & 0x04) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, ior) = if (property_bits & 0x08) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, attenuation) = if (property_bits & 0x10) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, power) = if (property_bits & 0x20) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, glow) = if (property_bits & 0x40) != 0 {
        map(le_f32, |v| Some(v))(input)
    } else { Ok((input, None)) }?;
    let (input, is_total_power) = if (property_bits & 0x80) != 0 {
        (input, true)
    } else { (input, false) };
    Ok((input, MaterialV1 { id, kind, weight, plastic, roughness, specular, ior, attenuation, power, glow, is_total_power }))
}

#[tracing::instrument]
fn parse_MATL(input: &[u8]) -> IResult<&[u8], MaterialV2> {
    tracing::trace!("parse_MATL(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, properties) = parse_DICT(input)?;
    Ok((input, MaterialV2 { id, properties }))
}

#[tracing::instrument]
fn parse_rOBJ(input: &[u8]) -> IResult<&[u8], Dict> {
    tracing::trace!("parse_rOBJ(len: {})", input.len());
    parse_DICT(input)
}

#[tracing::instrument]
fn parse_rCAM(input: &[u8]) -> IResult<&[u8], Camera> {
    tracing::trace!("parse_rCAM(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, attributes) = parse_DICT(input)?;
    Ok((input, Camera { id, attributes }))
}

#[tracing::instrument]
fn parse_IMAP(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    tracing::trace!("parse_IMAP(len: {})", input.len());
    count(le_u8, 256)(input)
}

#[tracing::instrument]
fn parse_NOTE(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    tracing::trace!("parse_NOTE(len: {})", input.len());
    let (input, string_count) = le_u32(input)?;
    count(parse_STRING, string_count as usize)(input)
}

#[tracing::instrument]
fn parse_nTRN(input: &[u8]) -> IResult<&[u8], TransformNode> {
    tracing::trace!("parse_nTRN(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, attrib) = parse_DICT(input)?;
    let (input, child_node_id) = le_u32(input)?;
    let (input, reserved_id) = le_i32(input)?;
    let (input, layer_id) = le_u32(input)?;
    let (input, frame_count) = le_u32(input)?;
    let (input, frames) = count(|input| {
        let (input, attrib) = parse_DICT(input)?;
        Ok((input, attrib))
    }, frame_count as usize)(input)?;
    Ok((input, TransformNode { id, attrib, child_node_id, reserved_id, layer_id, frames }))
}

#[tracing::instrument]
fn parse_nGRP(input: &[u8]) -> IResult<&[u8], GroupNode> {
    tracing::trace!("parse_nGRP(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, attrib) = parse_DICT(input)?;
    let (input, child_node_count) = le_u32(input)?;
    let (input, children) = count(le_u32, child_node_count as usize)(input)?;
    Ok((input, GroupNode { id, attrib, children }))
}

#[tracing::instrument]
fn parse_nSHP(input: &[u8]) -> IResult<&[u8], ShapeNode> {
    tracing::trace!("parse_nSHP(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, attrib) = parse_DICT(input)?;
    let (input, model_count) = le_u32(input)?;
    let (input, models) = count(|input| {
        let (input, id) = le_u32(input)?;
        let (input, attrib) = parse_DICT(input)?;
        Ok((input, (id, attrib)))
    }, model_count as usize)(input)?;
    Ok((input, ShapeNode { id, attrib, models }))
}

#[tracing::instrument]
fn parse_LAYR(input: &[u8]) -> IResult<&[u8], Layer> {
    tracing::trace!("parse_LAYR(len: {})", input.len());
    let (input, id) = le_u32(input)?;
    let (input, attrib) = parse_DICT(input)?;
    let (input, reserved) = le_i32(input)?;
    Ok((input, Layer { id, attributes: attrib, reserved }))
}

#[tracing::instrument]
fn parse_DICT(input: &[u8]) -> IResult<&[u8], Dict> {
    tracing::trace!("parse_DICT(len: {})", input.len());
    let (input, entry_count) = le_u32(input)?;
    let (input, entries) = count(tuple((parse_STRING, parse_STRING)), entry_count as usize)(input)?;
    Ok((input, Dict::from_iter(entries)))
}

#[tracing::instrument]
fn parse_STRING(input: &[u8]) -> IResult<&[u8], String> {
    tracing::trace!("parse_STRING(len: {})", input.len());
    let (input, bytes) = le_u32(input)?;
    let (input, buffer) = map_res(take(bytes), std::str::from_utf8)(input)?;
    Ok((input, String::from(buffer)))
}

#[cfg(test)]
mod tests {

    //#[test]
    fn test_3x3x3() {
        println!("Loading 3x3x3.vox");
        let file = std::fs::read("vox/3x3x3.vox").expect("Error opening test file.");
        let result = super::parse_file(&file);
        println!("{:#?}", result)
    }

    //#[test]
    fn test_8x8x8() {
        println!("Loading 8x8x8.vox");
        let file = std::fs::read("vox/8x8x8.vox").expect("Error opening test file.");
        let result = super::parse_file(&file);
        println!("{:#?}", result)
    }

    //#[test]
    fn test_menger() {
        println!("Loading menger.vox");
        let file = std::fs::read("vox/menger.vox").expect("Error opening test file.");
        let result = super::parse_file(&file);
        println!("{:?}", result)
    }

    #[test]
    fn test_streelamp() {
        println!("Loading streetlamp.vox");
        let file = std::fs::read("vox/streetlamp.vox").expect("Error opening test file.");
        let result = super::parse_file(&file);
        //println!("{:#?}", result)
    }
}