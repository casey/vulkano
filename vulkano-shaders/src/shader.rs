// Copyright (c) 2018 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use parse::{Instruction, Spirv};
use enums::Capability;
use entry_point::EntryPoint;

use std::collections::BTreeSet;

pub struct Shader {
    /// The shader's parsed SPIR-V bytecode
    pub spirv: Spirv,

    /// The device capabilities required by this shader. Since we will
    /// use these capabilities during codegen, and codegen should be
    /// deterministic, we store them in a sorted BTreeSet instead of a HashSet.
    pub capabilities: BTreeSet<Capability>,

    /// Entry Points to the shader binary
    pub entry_points: BTreeSet<EntryPoint>,
}

impl Shader {
    /// Build a shader from parsed SPIR-V bytecode
    pub fn from_spirv(spirv: Spirv) -> Shader {
        let capabilities = spirv.instructions.iter().filter_map(|instruction| {
            if let &Instruction::Capability(capability) = instruction {
                Some(capability)
            } else {
                None
            }
        }).collect();

        let entry_points = spirv.instructions.iter().filter_map(|instruction| {
            if let &Instruction::EntryPoint{ref execution, id, ref name, ref interface} = instruction {
                Some(EntryPoint{
                    execution_model: *execution,
                    id:              id,
                    interface:       interface.clone(),
                    name:            name.clone(),
                })
            } else {
                None
            }
        }).collect();

        Shader {
            spirv,
            capabilities,
            entry_points,
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
