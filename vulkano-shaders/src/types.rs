// Copyright (c) 2018 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use parse::Instruction;
use std::collections::BTreeMap;
use std::mem;
use enums::{StorageClass, Dim, ImageFormat, AccessQualifier, Decoration};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Void,
    Opaque{name: String},
    Bool,
    Int{width: u32, signedness: bool},
    Float{width: u32},
    Vector{element_count: u32, element_type: Box<Type>},
    Matrix{column_count: u32, column_type: Box<Type>},
    Array{element_count: u64, element_type: Box<Type>},
    RuntimeArray{element_type: Box<Type>},
    Struct{
        name:            String,
        member_types:    Vec<Type>,
        member_type_ids: Vec<u32>,
        interface_block: Option<InterfaceBlock>,
    },
    Pointer{storage_class: StorageClass, pointee_type: Box<Type>},
    SampledImage{image_type: Box<Type>},
    Sampler,
    Image{
        sampled_type:      Box<Type>,
        dim:               Dim,
        depth:             Option<bool>,
        arrayed:           bool,
        ms:                bool,
        sampled:           Option<bool>,
        format:            ImageFormat,
        access_qualifier:  Option<AccessQualifier>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterfaceBlock {
    Block,
    BufferBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustType {
    pub name:      String,
    pub size:      Option<usize>,
    pub alignment: usize,
}

macro_rules! rust_scalar_type {
    ($type:ident) => {
        {
            #[allow(dead_code)]
            #[repr(C)]
            struct Wrapper {
                data:  $type,
                after: u8,
            }
            let size = unsafe { (&(&*(0 as *const Wrapper)).after) as *const u8 as usize };
            RustType {
                name:      stringify!($type).to_string(),
                size:      Some(size),
                alignment: mem::align_of::<Wrapper>(),
            }
        }
    }
}

impl Type {
    // TODO: Should this return an error if there are no occupied locations?
    // TODO: Is this general enough to be on all types, or should it be in
    //       entrypoints.rs
    pub fn occupied_locations(&self) -> Option<usize> {
        use self::Type::*;
        match *self {
            Bool => None,
            Int{width: 8,  signedness: false} => Some(1),
            Int{width: 8,  signedness: true } => Some(1),
            Int{width: 16, signedness: false} => Some(1),
            Int{width: 16, signedness: true } => Some(1),
            Int{width: 32, signedness: false} => Some(1),
            Int{width: 32, signedness: true } => Some(1),
            Int{width: 64, signedness: false} => Some(1),
            Int{width: 64, signedness: true } => Some(1),
            Int{width: _,  signedness: _    } => None,
            Float{width: 32} => Some(1),
            Float{width: 64} => Some(1),
            Float{width: _ } => None,
            Vector{ref element_type, ..} => {
                // FIXME: This seems wrong
                element_type.occupied_locations()
            }
            Matrix{column_count, ref column_type} => 
                column_type.occupied_locations().map(|size| size * column_count as usize),
            Array{ref element_type, element_count} =>
                element_type.occupied_locations().map(|size| size * element_count as usize),
            RuntimeArray{..} => None,
            Struct{..} => None,
            Pointer{ref pointee_type, ..} => pointee_type.occupied_locations(),
            Image{..} => None,
            SampledImage{..} => None,
            Sampler => None,
            Opaque{..} => None,
            Void{..} => None,
        }
    }

    // TODO: Should this be a function on Type or RustType?
    // TODO: SHould these use the vulkano format types?
    // TODO: SHould this return result with descriptive errors?
    // TODO: Is this general enough to be on all types, or should it be in
    //       entrypoints.rs
    pub fn format(&self) -> Option<String> {
        use self::Type::*;
        match *self {
            Bool => None,
            Int{width: 8,  signedness: false} => Some("R8Uint".to_string()),
            Int{width: 8,  signedness: true } => Some("R8Sint".to_string()),
            Int{width: 16, signedness: false} => Some("R16Uint".to_string()),
            Int{width: 16, signedness: true } => Some("R16Sint".to_string()),
            Int{width: 32, signedness: false} => Some("R32Uint".to_string()),
            Int{width: 32, signedness: true } => Some("R32Sint".to_string()),
            Int{width: 64, signedness: false} => Some("R64Uint".to_string()),
            Int{width: 64, signedness: true } => Some("R64Sint".to_string()),
            Int{width: _,  signedness: _    } => None,
            Float{width: 32} => Some("R32Sfloat".to_string()),
            Float{width: 64} => Some("R64Sfloat".to_string()),
            Float{width: _ } => None,
            Vector{element_count, ref element_type} => {
                let element_format =  if let Some(element_format) = element_type.format() {
                    element_format
                } else {
                    return None;
                };
                if !element_format.starts_with("R32") {
                    // TODO: Why do we give up here?
                    return None;
                }
                let element_occupied_locations = element_type.occupied_locations();
                if element_occupied_locations != Some(1) {
                    // TODO: Why do we give up here?
                    return None;
                }
                match element_count {
                    1 => Some(element_format),
                    2 => Some(format!("R32G32{}", &element_format[3..])),
                    3 => Some(format!("R32G32B32{}", &element_format[3..])),
                    4 => Some(format!("R32G32B32A32{}", &element_format[3..])),
                    _ => None,
                }
            }
            Matrix{ref column_type, ..} => {
                column_type.format()
            }
            Array{ref element_type, ..} => {
                element_type.format()
            }
            RuntimeArray{..} => None,
            Struct{..} => None,
            // TODO: This seems wrong, why is the format of a pointer the same
            //       as the format of the pointee?
            Pointer{ref pointee_type, ..} => pointee_type.format(),
            Image{..} => None,
            SampledImage{..} => None,
            Sampler => None,
            Opaque{..} => None,
            Void{..} => None,
        }
    }

    // TODO: Should this be moved into codegen?
    pub fn rust_type(&self) -> Option<RustType> {
        use self::Type::*;
        match *self {
            Bool => Some(RustType {
                name:     "u32".to_string(),
                size:      Some(mem::size_of::<u32>()),
                alignment: mem::align_of::<u32>(),
            }),
            Int{width: 8,  signedness: false} => Some(rust_scalar_type!( u8)),
            Int{width: 8,  signedness: true } => Some(rust_scalar_type!( i8)),
            Int{width: 16, signedness: false} => Some(rust_scalar_type!(u16)),
            Int{width: 16, signedness: true } => Some(rust_scalar_type!(i16)),
            Int{width: 32, signedness: false} => Some(rust_scalar_type!(i32)),
            Int{width: 32, signedness: true } => Some(rust_scalar_type!(i32)),
            Int{width: 64, signedness: false} => Some(rust_scalar_type!(u64)),
            Int{width: 64, signedness: true } => Some(rust_scalar_type!(i64)),
            Int{width: _,  signedness: _    } => None,
            Float{width: 32} => Some(rust_scalar_type!(f32)),
            Float{width: 64} => Some(rust_scalar_type!(f64)),
            Float{width: _ } => None,
            Vector{element_count, ref element_type} => {
                // FIXME: This is intended to test that the alignment of a 3 element vector
                //        is the same as the alignment of a single element. Does this also
                //        guarantee that an N element vector of an arbitrary type will have
                //        the same alignment as a single element?
                debug_assert_eq!(mem::align_of::<[u32; 3]>(), mem::align_of::<u32>());
                element_type.rust_type().map(|RustType{name, size, alignment}| RustType {
                    name: format!("[{}; {}]", name, element_count),
                    size: size.map(|s| s * element_count as usize),
                    alignment,
                })
            }
            Matrix{column_count, ref column_type} => {
                // FIXME: Is this row-major or column-major? Also, see FIXME above.
                debug_assert_eq!(mem::align_of::<[u32; 3]>(), mem::align_of::<u32>());
                column_type.rust_type().map(|RustType{name, size, alignment}| RustType {
                    name: format!("[{}; {}]", name, column_count),
                    size: size.map(|s| s * column_count as usize),
                    alignment,
                })
            }
            Array{element_count, ref element_type} => {
                // FIXME: See above. Also, this had an extra FIXME with no comment in the
                //        original source.
                debug_assert_eq!(mem::align_of::<[u32; 3]>(), mem::align_of::<u32>());
                element_type.rust_type().map(|RustType{name, size, alignment}| RustType {
                    name:      format!("[{}; {}]", name, element_count),
                    size:      size.map(|s| s * element_count as usize),
                    alignment: alignment,
                })
            }
            RuntimeArray{ref element_type} => {
                element_type.rust_type().map(|RustType{name, size: _, alignment}| RustType {
                    name:      format!("[{}]", name),
                    size:      None,
                    alignment: alignment,
                })
            }
            Struct{name: ref _name, ref member_types, member_type_ids: ref _member_type_ids, interface_block: _interface_block} => {
                member_types.iter()
                    .map(|ty| ty.rust_type())
                    // If any of the member types has no rust type, this collect will
                    // return None, returning None for the struct as a whole.
                    .collect::<Option<Vec<RustType>>>()
                    .map(|_member_rust_types| {
                        panic!();
                        /*
                        RustType {
                            name:      name.clone(),
                            alignment: member_rust_types.iter()
                                        .map(|rust_type| rust_type.alignment)
                                        .max()
                                        .unwrap_or(1),
                            size:      Some(1)
                        }
                        */
                    })
            }
            Pointer{..} => None,
            Image{..} => None,
            SampledImage{..} => None,
            Sampler => None,
            Opaque{..} => None,
            Void{..} => None,
        }
    }
}

pub fn extract_types(
    instructions: &[Instruction],
    names:        &BTreeMap<u32, String>,
    decorations:  &BTreeMap<(u32, Decoration), Vec<u32>>,
) -> Result<BTreeMap<u32, Type>, u32> {
    let constants = instructions.iter().filter_map(|instruction| {
        if let Instruction::Constant{result_id, ref data, ..} = *instruction {
            Some((result_id, data.as_slice()))
        } else {
            None
        }
    }).collect::<BTreeMap<u32, &[u32]>>();

    let mut incomplete_types = BTreeMap::new();

    for instruction in instructions {
        let (result_id, incomplete_type) = match *instruction {
            Instruction::TypeBool{result_id} => 
                (result_id, IncompleteType::Bool),
            Instruction::TypeInt{result_id, width, signedness} => 
                (result_id, IncompleteType::Int{width, signedness}),
            Instruction::TypeFloat{result_id, width} =>
                (result_id, IncompleteType::Float{width}),
            Instruction::TypeVector{result_id, component_id, count} =>
                (result_id, IncompleteType::Vector{element_count: count, element_type: component_id}),
            Instruction::TypeMatrix{result_id, column_type_id, column_count} =>
                (result_id, IncompleteType::Matrix{column_count, column_type: column_type_id}),
            Instruction::TypeArray{result_id, type_id, length_id} => {
                if let Some(length_data) = constants.get(&length_id) {
                    // FIXME: This is very hard to understand.
                    let element_count = length_data
                        .iter()
                        .rev()
                        .fold(0u64, |a, &b| (a << 32) | b as u64);
                    (result_id, IncompleteType::Array{element_type: type_id, element_count})
                } else {
                    panic!("could not find array length");
                }
            }
            Instruction::TypeRuntimeArray{result_id, type_id} =>
                (result_id, IncompleteType::RuntimeArray{element_type: type_id}),
            Instruction::TypeStruct{result_id, ref member_types} => {
                let name = names.get(&result_id).expect("could not find struct name").clone();

                use enums::Decoration::{DecorationBlock, DecorationBufferBlock};
                let interface_block = if decorations.contains_key(&(result_id, DecorationBlock)) {
                    Some(InterfaceBlock::Block)
                } else if decorations.contains_key(&(result_id, DecorationBufferBlock)) {
                    Some(InterfaceBlock::BufferBlock)
                } else {
                    None
                };

                (
                    result_id,
                    IncompleteType::Struct{name, member_types: member_types.clone(), interface_block},
                )
            }
            Instruction::TypePointer{result_id, storage_class, type_id} => {
                (result_id, IncompleteType::Pointer{storage_class, pointee_type: type_id})
            }
            Instruction::TypeSampledImage{result_id, image_type_id} => {
                (result_id, IncompleteType::SampledImage{image_type: image_type_id})
            }
            Instruction::TypeSampler{result_id} => {
                (result_id, IncompleteType::Sampler)
            }
            Instruction::TypeImage{
                result_id,
                sampled_type_id,
                dim,
                depth,
                arrayed,
                ms,
                sampled,
                format,
                access,
            } => {
                let image_type = IncompleteType::Image{
                    access_qualifier: access,
                    sampled_type: sampled_type_id,
                    dim,
                    depth,
                    arrayed,
                    ms,
                    sampled,
                    format,
                };
                (result_id, image_type)
            }
            Instruction::TypeVoid{result_id} => {
                (result_id, IncompleteType::Void)
            },
            Instruction::TypeOpaque{result_id, ref name} => {
                (result_id, IncompleteType::Opaque{name: name.clone()})
            },
            _ => continue,
        };
        if incomplete_types.contains_key(&result_id) {
            panic!("Duplicate type: {}", result_id)
        }
        incomplete_types.insert(result_id, incomplete_type);
    };

    TypeResolver {
        complete_types: BTreeMap::new(),
        incomplete_types,
    }.resolve()
}

#[derive(Debug)]
enum IncompleteType {
    Void,
    Opaque{name: String},
    Bool,
    Int{width: u32, signedness: bool},
    Float{width: u32},
    Vector{element_count: u32, element_type: u32},
    Matrix{column_count: u32, column_type: u32},
    Array{element_count: u64, element_type: u32},
    RuntimeArray{element_type: u32},
    Struct{name: String, member_types: Vec<u32>, interface_block: Option<InterfaceBlock>},
    Pointer{storage_class: StorageClass, pointee_type: u32},
    SampledImage{image_type: u32},
    Sampler,
    Image{
        sampled_type:     u32,
        dim:              Dim,
        depth:            Option<bool>,
        arrayed:          bool,
        ms:               bool,
        sampled:          Option<bool>,
        format:           ImageFormat,
        access_qualifier: Option<AccessQualifier>,
    },
}

struct TypeResolver {
    incomplete_types: BTreeMap<u32, IncompleteType>,
    complete_types:   BTreeMap<u32, Type>,
}

impl TypeResolver {
    fn resolve(mut self) -> Result<BTreeMap<u32, Type>, u32> {
        for id in self.incomplete_types.keys() {
            let complete_type = self.resolve_id(*id)?;
            self.complete_types.insert(*id, complete_type);
        }
        Ok(self.complete_types)
    }

    fn resolve_id(&self, id: u32) -> Result<Type, u32> {
        if let Some(complete_type) = self.complete_types.get(&id) {
            Ok(complete_type.clone())
        } else {
            self.resolve_type(self.incomplete_types.get(&id).ok_or(id)?)
        }
    }

    fn resolve_type(&self, incomplete_type: &IncompleteType) -> Result<Type, u32> {
        Ok(match incomplete_type {
            &IncompleteType::Void => Type::Void,
            &IncompleteType::Opaque{ref name} => Type::Opaque{name: name.clone()},
            &IncompleteType::Bool => Type::Bool,
            &IncompleteType::Int{width, signedness} => Type::Int{width, signedness},
            &IncompleteType::Float{width} => Type::Float{width},
            &IncompleteType::Vector{element_count, element_type} => Type::Vector {
                element_type: Box::new(self.resolve_id(element_type)?),
                element_count,
            },
            &IncompleteType::Matrix{column_count, column_type} => Type::Matrix {
                column_type: Box::new(self.resolve_id(column_type)?),
                column_count,
            },
            &IncompleteType::Array{element_count, element_type} => Type::Array {
                element_type: Box::new(self.resolve_id(element_type)?),
                element_count,
            },
            &IncompleteType::RuntimeArray{element_type} => Type::RuntimeArray {
                element_type: Box::new(self.resolve_id(element_type)?),
            },
            &IncompleteType::Struct{ref name, ref member_types, interface_block} => Type::Struct {
                name:         name.clone(),
                member_types: member_types
                    .iter()
                    .map(|type_id| self.resolve_id(*type_id))
                    .collect::<Result<Vec<Type>, u32>>()?,
                member_type_ids: member_types.clone(),
                interface_block,
            },
            &IncompleteType::Pointer{storage_class, pointee_type} => Type::Pointer {
                pointee_type: Box::new(self.resolve_id(pointee_type)?),
                storage_class,
            },
            &IncompleteType::SampledImage{image_type} => Type::SampledImage {
                image_type: Box::new(self.resolve_id(image_type)?),
            },
            &IncompleteType::Sampler => Type::Sampler,
            &IncompleteType::Image {
                access_qualifier,
                sampled_type,
                dim,
                depth,
                arrayed,
                ms,
                sampled,
                format,
            } => Type::Image{
                sampled_type: Box::new(self.resolve_id(sampled_type)?),
                access_qualifier,
                dim,
                depth,
                arrayed,
                ms,
                sampled,
                format,
            },
        })
    }
}
