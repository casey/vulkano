use std::fmt;
use spec_consts::SpecializationConstant;

/// Writes the `SpecializationConstants` struct that contains the specialization constants and
/// implements the `Default` and the `vulkano::pipeline::shader::SpecializationConstants` traits.
pub fn write(
    specialization_constants: &[SpecializationConstant],
    destination:              &mut fmt::Write,
) -> fmt::Result {
    fn default_value(specialization_constant: &SpecializationConstant) -> String {
        use spec_consts::SpecializationConstantKind::*;
        match specialization_constant.kind {
            True => "1u32".to_string(),
            False => "0u32".to_string(),
            Scalar{ref default_value} |
            Composite{ref default_value} => format!(
                "unsafe {{ ::std::mem::transmute([{}]) }}",
                default_value
                    .iter()
                    .map(|x| format!("{}u32", x))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    let map_entries = {
        let mut map_entries = Vec::new();
        let mut curr_offset = 0;
        for c in specialization_constants {
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

    write!(
        destination,
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
        struct_def = specialization_constants
            .iter()
            .map(|c| format!("pub {}: {}", c.name, c.rust_type.name))
            .collect::<Vec<_>>()
            .join(", "),
        def_vals = specialization_constants
            .iter()
            .map(|c| format!("{}: {}", c.name, default_value(c)))
            .collect::<Vec<_>>()
            .join(", "),
        num_map_entries = map_entries.len(),
        map_entries = map_entries.join(", ")
    )
}
