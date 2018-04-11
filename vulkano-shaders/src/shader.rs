// Copyright (c) 2018 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use parse::{Instruction, Spirv};
use enums::{Capability, Decoration};
use entry_point::EntryPoint;
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
    pub specialization_constants: BTreeSet<SpecializationConstant>,

    /// A map of (result id, Decoration) tuples to the decoration content
    pub decorations: BTreeMap<(u32, Decoration), Vec<u32>>,

    /// A map of (result id, member number, Decoration) tuples to the decoration content
    pub member_decorations: BTreeMap<(u32, u32, Decoration), Vec<u32>>,

    /// Types described by the shader binary
    pub types: BTreeMap<u32, Type>,
}

impl Shader {
    /// Build a shader from parsed SPIR-V bytecode
    pub fn from_spirv(spirv: Spirv) -> Shader {
        let mut capabilities                = BTreeSet::new();
        let mut entry_points                = BTreeSet::new();
        let mut names                       = BTreeMap::new();
        let mut decorations                 = BTreeMap::new();
        let mut member_decorations          = BTreeMap::new();

        for instruction in &spirv.instructions {
            match instruction {
                &Instruction::Capability(capability) => {
                    capabilities.insert(capability);
                }
                &Instruction::EntryPoint{ref execution, id, ref name, ref interface} => {
                    entry_points.insert(EntryPoint {
                        execution_model: *execution,
                        id:              id,
                        interface:       interface.clone(),
                        name:            name.clone(),
                    });
                }
                &Instruction::Name{target_id, ref name} => {
                    if names.contains_key(&target_id) {
                        panic!("Duplicate name: {} {}", target_id, name);
                    }
                    names.insert(target_id, name.clone());
                }
                &Instruction::MemberDecorate{target_id, member, decoration, ref params} => {
                    member_decorations.insert((target_id, member, decoration), params.clone());
                }
                &Instruction::Decorate{target_id, decoration, ref params} => {
                    decorations.insert((target_id, decoration), params.clone());
                    /*
                    match decoration {
                        Decoration::DecorationSpecId => {
                        let constant_id = params[0];
                        if specialization_constant_ids.contains_key(&constant_id) {
                            panic!("Duplicate specialization constant decoration: {}", constant_id);
                        }
                        specialization_constant_ids.insert(target_id, constant_id);
                        },
                        _ => {}
                    }
                    */
                }
                _ => {}
            };
        }

        let types = extract_types(&spirv.instructions, &names)
            .expect("failed to extract types");

        let mut specialization_constants = BTreeSet::new();

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

            specialization_constants.insert(SpecializationConstant {
                constant_id,
                kind,
                name,
                rust_size,
                rust_type,
                spirv_type,
            });
        }

        Shader {
            specialization_constants,
            capabilities,
            decorations,
            entry_points,
            member_decorations,
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
            id:              4,
            name:            "main".to_string(),
            interface:       vec![13, 17],
        }]);
    }
}
