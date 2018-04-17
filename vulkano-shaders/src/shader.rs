// Copyright (c) 2018 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use descriptor_sets::NewDescriptor;
use entry_point::{EntryPoint, InterfaceVariable};
use enums::{Capability, Decoration, StorageClass};
use parse::{Instruction, Spirv};
use spec_consts::{SpecializationConstant, SpecializationConstantKind};
use types::{extract_types, Type};

use std::collections::{BTreeSet, BTreeMap};

pub struct Shader {
    /// The shader's parsed SPIR-V bytecode
    pub spirv: Spirv,

    /// The device capabilities required by this shader. Since we will
    /// use these capabilities during codegen, and codegen should be
    /// deterministic, we store them in a sorted BTreeSet instead of a HashSet.
    pub capabilities: BTreeSet<Capability>,

    /// Entry Points to the shader binary
    pub entry_points: BTreeSet<EntryPoint>,

    /// Specialization constants defined in the shader binary
    pub specialization_constants: Vec<SpecializationConstant>,

    /// A map of (result id, Decoration) tuples to the decoration content
    pub decorations: BTreeMap<(u32, Decoration), Vec<u32>>,

    /// A map of (result id, member number, Decoration) tuples to the decoration content
    pub member_decorations: BTreeMap<(u32, u32, Decoration), Vec<u32>>,

    /// Types described by the shader binary
    pub types: BTreeMap<u32, Type>,

    /// Shader descriptors
    pub descriptors: Vec<NewDescriptor>,

    /// Shader push constants
    pub push_constants: Vec<Type>,
}

impl Shader {
    pub fn _decoration(&self, target_id: u32, decoration: Decoration) -> Option<&[u32]> {
        self.decorations.get(&(target_id, decoration)).map(Vec::as_slice)
    }

    pub fn _member_decoration(
        &self,
        target_id:     u32,
        member_number: u32,
        decoration:    Decoration,
    ) -> Option<&[u32]> {
        self.member_decorations.get(&(target_id, member_number, decoration)).map(Vec::as_slice)
    }

    /// Build a shader from parsed SPIR-V bytecode
    pub fn from_spirv(spirv: Spirv) -> Shader {
        // TODO: These should probbaly all be hashmaps/sets, with sorting happening
        //       during codegen.
        let mut capabilities                = BTreeSet::new();
        let mut names                       = BTreeMap::new();
        let mut decorations                 = BTreeMap::new();
        let mut member_decorations          = BTreeMap::new();
        let mut variables                   = BTreeMap::new();
        let mut execution_modes             = BTreeMap::new();
        let mut descriptors                 = Vec::new();
        let mut push_constants              = Vec::new();

        for instruction in &spirv.instructions {
            match instruction {
                &Instruction::Capability(capability) => {
                    capabilities.insert(capability);
                }
                &Instruction::Name{target_id, ref name} => {
                    if names.contains_key(&target_id) {
                        panic!("Duplicate name: {} {}", target_id, name);
                    }
                    names.insert(target_id, name.clone());
                }
                &Instruction::Decorate{target_id, decoration, ref params} => {
                    decorations.insert((target_id, decoration), params.clone());
                }
                &Instruction::MemberDecorate{target_id, member, decoration, ref params} => {
                    member_decorations.insert((target_id, member, decoration), params.clone());
                }
                &Instruction::Variable{result_type_id, result_id, storage_class, .. } => {
                    variables.insert(result_id, (result_type_id, storage_class));
                }
                &Instruction::ExecutionMode{target_id, mode, ..} => {
                    (*execution_modes.entry(target_id).or_insert_with(|| Vec::new())).push(mode);
                }
                _ => {}
            };
        }

        let types = extract_types(&spirv.instructions, &names, &decorations)
            .expect("failed to extract types");

        for spirv_type in types.values() {
            if let Type::Pointer{
                storage_class: StorageClass::StorageClassPushConstant,
                ref            pointee_type,
            } = *spirv_type {
                push_constants.push(*pointee_type.clone());
            }
        }

        for (&(variable_id, decoration), params) in &decorations {
            if decoration != Decoration::DecorationDescriptorSet {
                continue;
            }
            let descriptor_set = params[0];
            let &(type_id, _) = variables.get(&variable_id).unwrap();
            let pointer_type = types.get(&type_id).unwrap().clone();
            let spirv_type = if let Type::Pointer{pointee_type, ..} = pointer_type {
                *pointee_type
            } else {
                panic!();
            };

            let name = names.get(&variable_id).unwrap().clone();
            let binding_point = decorations.get(&(variable_id, Decoration::DecorationBinding))
                .unwrap()[0];

            descriptors.push(NewDescriptor {
                binding_point,
                descriptor_set,
                name,
                spirv_type,
                type_id,
            });
        }

        let entry_points = spirv.instructions.iter().flat_map(|instruction| {
            if let &Instruction::EntryPoint{ref execution, id, ref name, ref interface} = instruction {
                let mut inputs = Vec::new();
                let mut outputs = Vec::new();

                for interface_id in interface {
                    // TODO: ::is_builtin contains checks to see if the type of the interface, not
                    //       just the interface itself had the builtin decorator. I'd like to
                    //       understand those checks better.
                    if decorations.contains_key(&(*interface_id, Decoration::DecorationBuiltIn)) {
                        continue;
                    }

                    if let Some(&(type_id, storage_class)) = variables.get(interface_id) {
                        let destination = match storage_class {
                            StorageClass::StorageClassInput => &mut inputs,
                            StorageClass::StorageClassOutput => &mut outputs,
                            _ => continue,
                        };

                        let name = names
                            .get(&interface_id)
                            .expect("interface with no name").clone();

                        if name == "" {
                            // TODO: What does this mean?
                            continue;
                        }

                        destination.push(InterfaceVariable {
                            spirv_type: types
                                            .get(&type_id)
                                            .expect("interface with no type").clone(),
                            location:   decorations
                                            .get(&(*interface_id, Decoration::DecorationLocation))
                                            .expect("interface with no location")[0],
                            name,
                        });
                    } else {
                        panic!("interface element without associated variable")
                    }
                }

                Some(EntryPoint {
                    execution_model: *execution,
                    execution_modes: execution_modes.remove(&id).unwrap_or_else(Vec::new).clone(),
                    id:              id,
                    name:            name.clone(),
                    inputs,
                    outputs,
                })
            } else {
                None
            }
        }).collect();

        let mut specialization_constants = Vec::new();

        for instruction in &spirv.instructions {
            let (result_type_id, result_id, kind) = match instruction {
                &Instruction::SpecConstantTrue {
                    result_type_id,
                    result_id,
                } => {
                    (result_type_id, result_id, SpecializationConstantKind::True)
                },
                &Instruction::SpecConstantFalse {
                    result_type_id,
                    result_id,
                } => {
                    (result_type_id, result_id, SpecializationConstantKind::False)
                },
                &Instruction::SpecConstant {
                    result_type_id,
                    result_id,
                    ref data,
                } => {
                    let kind = SpecializationConstantKind::Scalar{default_value: data.clone()};
                    (result_type_id, result_id, kind)
                },
                &Instruction::SpecConstantComposite {
                    result_type_id,
                    result_id,
                    ref data,
                } => {
                    let kind = SpecializationConstantKind::Composite{default_value: data.clone()};
                    (result_type_id, result_id, kind)
                },
                _ => continue,
            };

            let constant_id = decorations
                .remove(&(result_id, Decoration::DecorationSpecId))
                .expect("no id for specialization constant")
                [0];

            let name = names.get(&result_id)
                .expect("unnamed specialization constant")
                .clone();

            let spirv_type = types.get(&result_type_id)
                .expect("Specialization constant with no type")
                .clone();

            let rust_type = spirv_type.rust_type()
                .expect("Specialization constant with no rust type");

            let rust_size = rust_type.size
                .expect("Specialization constant with unsized rust type");

            specialization_constants.push(SpecializationConstant {
                constant_id,
                kind,
                name,
                rust_size,
                rust_type,
                spirv_type,
            });
        }

        // Sort specialization constants by their constant IDs
        specialization_constants.sort_by_key(|c| c.constant_id);

        Shader {
            capabilities,
            decorations,
            descriptors,
            entry_points,
            member_decorations,
            push_constants,
            specialization_constants,
            spirv,
            types,
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::prelude::*;

    use enums::ExecutionModel;
    use glsl_to_spirv::{compile, ShaderType};
    use parse::parse_spirv;

    use super::*;
    const PASSTHROUGH_VERTEX_SHADER: &str = include_str!("../data/passthrough-vertex-shader.glsl");

    #[test]
    fn simple_shader() {
        let mut spirv_output_file = compile(PASSTHROUGH_VERTEX_SHADER, ShaderType::Vertex)
            .expect("failed to compile data/passthrough-vertex-shader.glsl");

        let mut spirv_bytes = Vec::new();
        spirv_output_file.read_to_end(&mut spirv_bytes)
            .expect("failed to read SPIR-V output file");

        let spirv = parse_spirv(&spirv_bytes)
            .expect("failed to parse SPIR-V from data/passthrough-vertex-shader.glsl");

        let shader = Shader::from_spirv(spirv);

        let capabilities = shader.capabilities.into_iter().collect::<Vec<_>>();
        let entry_points = shader.entry_points.into_iter().collect::<Vec<_>>();

        assert_eq!(capabilities, &[Capability::CapabilityShader]);
        assert_eq!(entry_points, &[EntryPoint {
            execution_model: ExecutionModel::ExecutionModelVertex,
            execution_modes: vec![],
            id:              4,
            name:            "main".to_string(),
            inputs:          vec![InterfaceVariable {
                name: "position".to_string(),
                location: 0,
                spirv_type: Type::Pointer {
                    storage_class: StorageClass::StorageClassInput,
                    pointee_type: Box::new(Type::Vector{
                        element_count: 4,
                        element_type: Box::new(Type::Float{width: 32}),
                    }),
                },
            }],
            outputs:         vec![],
        }]);
    }
}
