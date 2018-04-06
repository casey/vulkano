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

use std::collections::BTreeSet;

pub struct Shader {
    /// The shader's parsed SPIR-V bytecode
    pub spirv: Spirv,
    /// The device capabilities required by this shader. Since we will
    /// use these capabilities during codegen, and codegen should be
    /// deterministic, we store them in a sorted BTreeSet instead of a HashSet.
    pub capabilities: BTreeSet<Capability>,
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

        Shader {
            spirv,
            capabilities,
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::prelude::*;

    use glsl_to_spirv::{compile, ShaderType};
    use parse::parse_spirv;

    use super::*;

    const PASSTHROUGH_VERTEX_SHADER: &str = include_str!("../tests/passthrough-vertex-shader.glsl");

    #[test]
    fn simple_capabilities() {
        let mut spirv_output_file = compile(PASSTHROUGH_VERTEX_SHADER, ShaderType::Vertex)
            .expect("failed to compile tests/passthrough-vertex-shader.glsl");

        let mut spirv_bytes = Vec::new();
        spirv_output_file.read_to_end(&mut spirv_bytes)
            .expect("failed to read SPIR-V output file");

        let spirv = parse_spirv(&spirv_bytes)
            .expect("failed to parse SPIR-V from tests/passthrough-vertex-shader.glsl");

        let shader = Shader::from_spirv(spirv);

        let capabilities = shader.capabilities.iter().cloned().collect::<Vec<_>>();

        assert_eq!(capabilities, &[Capability::CapabilityShader]);
    }
}
