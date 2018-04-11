// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use shader::Shader;
use types::{RustType, Type};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct SpecializationConstant {
    pub constant_id:    u32,
    pub name:           String,
    pub kind:           SpecializationConstantKind,
    pub spirv_type:     Type,
    pub rust_type:      RustType,
    pub rust_size:      usize,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum SpecializationConstantKind {
    True,
    False,
    Scalar{default_value: Vec<u32>},
    Composite{default_value: Vec<u32>},
}

fn default_value(specialization_constant: &SpecializationConstant) -> String {
    match specialization_constant.kind {
        SpecializationConstantKind::True => "1u32".to_string(),
        SpecializationConstantKind::False => "0u32".to_string(),
        SpecializationConstantKind::Scalar{ref default_value} |
        SpecializationConstantKind::Composite{ref default_value} => format!(
            "unsafe {{ ::std::mem::transmute([{}]) }}",
            default_value
                .iter()
                .map(|x| format!("{}u32", x))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Writes the `SpecializationConstants` struct that contains the specialization constants and
/// implements the `Default` and the `vulkano::pipeline::shader::SpecializationConstants` traits.
pub fn write_specialization_constants(shader: &Shader) -> String {
    let map_entries = {
        let mut map_entries = Vec::new();
        let mut curr_offset = 0;
        for c in &shader.specialization_constants {
            map_entries.push(format!(
                "SpecializationMapEntry {{
                constant_id: \
                 {},
                offset: {},
                size: {},
            \
                 }}",
                c.constant_id,
                curr_offset,
                c.rust_size
            ));

            assert_ne!(c.rust_size, 0);
            curr_offset += c.rust_size;
            curr_offset = c.rust_type.alignment * (1 + (curr_offset - 1) / c.rust_type.alignment);
        }
        map_entries
    };

    format!(
        r#"

#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct SpecializationConstants {{
    {struct_def}
}}

impl Default for SpecializationConstants {{
    fn default() -> SpecializationConstants {{
        SpecializationConstants {{
            {def_vals}
        }}
    }}
}}

unsafe impl SpecConstsTrait for SpecializationConstants {{
    fn descriptors() -> &'static [SpecializationMapEntry] {{
        static DESCRIPTORS: [SpecializationMapEntry; {num_map_entries}] = [
            {map_entries}
        ];
        &DESCRIPTORS
    }}
}}

    "#,
        struct_def = shader.specialization_constants
            .iter()
            .map(|c| format!("pub {}: {}", c.name, c.rust_type.name))
            .collect::<Vec<_>>()
            .join(", "),
        def_vals = shader.specialization_constants
            .iter()
            .map(|c| format!("{}: {}", c.name, default_value(c)))
            .collect::<Vec<_>>()
            .join(", "),
        num_map_entries = map_entries.len(),
        map_entries = map_entries.join(", ")
    )
}
