// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::fmt;

use codegen::capitalize;
use entry_point::{EntryPoint, InterfaceVariable};
use enums::{ExecutionModel, ExecutionMode};
use types::Type;

pub fn write_entry_points(
    entry_points:                 &[EntryPoint],
    has_specialization_constants: bool,
    destination:                  &mut fmt::Write,
) -> fmt::Result {
    for entry_point in entry_points {
        write_entry_point(entry_point, has_specialization_constants, destination)?;
    }
    Ok(())
}

fn write_entry_point(
    entry_point:                  &EntryPoint,
    has_specialization_constants: bool,
    destination:                  &mut fmt::Write,
) -> fmt::Result {
    let capitalized_ep_name = capitalize(&entry_point.name);

    let spec_consts_struct = if has_specialization_constants {
        "SpecializationConstants"
    } else {
        "()"
    };

    let (ty, f_call) = {
        if let ExecutionModel::ExecutionModelGLCompute = entry_point.execution_model {
            (format!("::vulkano::pipeline::shader::ComputeEntryPoint<{}, Layout>",
                     spec_consts_struct),
             format!("compute_entry_point(::std::ffi::CStr::from_ptr(NAME.as_ptr() as *const _), \
                      Layout(ShaderStages {{ compute: true, .. ShaderStages::none() }}))"))

        } else {
            let ty = match entry_point.execution_model {
                ExecutionModel::ExecutionModelVertex => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::Vertex".to_owned()
                },

                ExecutionModel::ExecutionModelTessellationControl => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::TessellationControl"
                        .to_owned()
                },

                ExecutionModel::ExecutionModelTessellationEvaluation => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::TessellationEvaluation"
                        .to_owned()
                },

                ExecutionModel::ExecutionModelGeometry => {
                    let mut execution_mode = None;

                    for mode in &entry_point.execution_modes {
                        execution_mode = match *mode {
                            ExecutionMode::ExecutionModeInputPoints => Some("Points"),
                            ExecutionMode::ExecutionModeInputLines => Some("Lines"),
                            ExecutionMode::ExecutionModeInputLinesAdjacency =>
                                Some("LinesWithAdjacency"),
                            ExecutionMode::ExecutionModeTriangles => Some("Triangles"),
                            ExecutionMode::ExecutionModeInputTrianglesAdjacency =>
                                Some("TrianglesWithAdjacency"),
                            _ => continue,
                        };
                        break;
                    };

                    format!(
                        "::vulkano::pipeline::shader::GraphicsShaderType::Geometry(
                        \
                         ::vulkano::pipeline::shader::GeometryShaderExecutionMode::{0}
                    \
                         )",
                        execution_mode.unwrap()
                    )
                },

                ExecutionModel::ExecutionModelFragment => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::Fragment".to_owned()
                },

                ExecutionModel::ExecutionModelGLCompute => {
                    unreachable!()
                },

                ExecutionModel::ExecutionModelKernel => panic!("Kernels are not supported"),
            };

            let stage = match entry_point.execution_model {
                ExecutionModel::ExecutionModelVertex => {
                    "ShaderStages { vertex: true, .. ShaderStages::none() }"
                },
                ExecutionModel::ExecutionModelTessellationControl => {
                    "ShaderStages { tessellation_control: true, .. ShaderStages::none() }"
                },
                ExecutionModel::ExecutionModelTessellationEvaluation => {
                    "ShaderStages { tessellation_evaluation: true, .. ShaderStages::none() }"
                },
                ExecutionModel::ExecutionModelGeometry => {
                    "ShaderStages { geometry: true, .. ShaderStages::none() }"
                },
                ExecutionModel::ExecutionModelFragment => {
                    "ShaderStages { fragment: true, .. ShaderStages::none() }"
                },
                ExecutionModel::ExecutionModelGLCompute => unreachable!(),
                ExecutionModel::ExecutionModelKernel => unreachable!(),
            };

            let t = format!("::vulkano::pipeline::shader::GraphicsEntryPoint<{0}, {1}Input, \
                                {1}Output, Layout>",
                            spec_consts_struct,
                            capitalized_ep_name);
            let f = format!("graphics_entry_point(::std::ffi::CStr::from_ptr(NAME.as_ptr() \
                                as *const _), {0}Input, {0}Output, Layout({2}), {1})",
                            capitalized_ep_name,
                            ty,
                            stage);

            (t, f)
        }
    };

    write!(
        destination,
        r#"
    /// Returns a logical struct describing the entry point named `{ep_name}`.
    #[inline]
    #[allow(unsafe_code)]
    pub fn {ep_name}_entry_point(&self) -> {ty} {{
        unsafe {{
            #[allow(dead_code)]
            static NAME: [u8; {ep_name_lenp1}] = [{encoded_ep_name}, 0];     // "{ep_name}"
            self.shader.{f_call}
        }}
    }}
            "#,
        ep_name = entry_point.name,
        ep_name_lenp1 = entry_point.name.chars().count() + 1,
        ty = ty,
        encoded_ep_name = entry_point.name
            .chars()
            .map(|c| (c as u32).to_string())
            .collect::<Vec<String>>()
            .join(", "),
        f_call = f_call
    )
}

pub fn write_interface_structs(
    entry_points: &[EntryPoint],
    destination: &mut fmt::Write,
) -> fmt::Result {
    for entry_point in entry_points {
        write_entry_point_interface_structs(entry_point, destination)?;
    }
    Ok(())
}

fn write_entry_point_interface_structs(
    entry_point: &EntryPoint,
    destination: &mut fmt::Write,
) -> fmt::Result {
    let capitalized_name = capitalize(&entry_point.name);

    // TODO: Should this logic be moved into shader parsing? i.e. should the type of each
    //       interface variable in entry_point.interfaces be stripped of the outer
    //       array wrapper before we get here?
    // TODO: Find out and document why we do this
    let ignore_first_array_in = match entry_point.execution_model {
        ExecutionModel::ExecutionModelTessellationControl |
        ExecutionModel::ExecutionModelTessellationEvaluation |
        ExecutionModel::ExecutionModelGeometry => true,
        _ => false,
    };

    let ignore_first_array_out =
        entry_point.execution_model == ExecutionModel::ExecutionModelTessellationControl;

    write_interface_struct(
        &format!("{}Input", capitalized_name),
        &entry_point.inputs,
        ignore_first_array_in,
        destination,
    )?;

    write_interface_struct(
        &format!("{}Output", capitalized_name),
        &entry_point.outputs,
        ignore_first_array_out,
        destination,
    )
}

fn write_interface_struct(
    struct_name:       &str,
    interfaces:        &[InterfaceVariable],
    strip_outer_array: bool,
    destination:       &mut fmt::Write,
) -> fmt::Result {
    let attributes = interfaces.iter().map(|interface| {
        let mut spirv_type = interface.spirv_type.clone();

        if strip_outer_array {
            if let Type::Array{element_type, ..} = spirv_type {
                spirv_type = *element_type;
            } else {
                panic!("tried to strip outer array from type that was not an array: {:?}",
                       interface.spirv_type);
            }
        }

        // TODO: do something better than these unwraps. They could just be moved onto the
        //       InterfaceVariable itself, without the option wrapper, and we could complain
        //       at shader parse time if an interface variable contained a type without a format
        //       or an occupied locations. we might also want to move occupied_locations() and
        //       format() off of the Type object if they aren't useful in any other context.
        (interface.location, interface.name.clone(), (spirv_type.format().unwrap(), spirv_type.occupied_locations().unwrap()))
    }).collect::<Vec<(u32, String, (String, usize))>>();

    // Checking for overlapping elements.
    // TODO: Move this check into shader parsing.
    for (offset, &(loc, ref name, (_, loc_len))) in attributes.iter().enumerate() {
        for &(loc2, ref name2, (_, loc_len2)) in attributes.iter().skip(offset + 1) {
            if loc == loc2 || (loc < loc2 && loc + loc_len as u32 > loc2) ||
                (loc2 < loc && loc2 + loc_len2 as u32 > loc)
            {
                panic!("The locations of attributes `{}` (start={}, size={}) \
                        and `{}` (start={}, size={}) overlap",
                       name,
                       loc,
                       loc_len,
                       name2,
                       loc2,
                       loc_len2);
            }
        }
    }

    let body = attributes
        .iter()
        .enumerate()
        .map(|(num, &(loc, ref name, (ref ty, num_locs)))| {
            assert!(num_locs >= 1);

            format!(
                "if self.num == {} {{
            self.num += 1;

            return Some(::vulkano::pipeline::shader::ShaderInterfaceDefEntry {{
                location: {} .. {},
                format: ::vulkano::format::Format::{},
                name: Some(::std::borrow::Cow::Borrowed(\"{}\"))
            }});
        }}",
                num,
                loc,
                loc as usize + num_locs,
                ty,
                name
            )
        })
        .collect::<Vec<_>>()
        .join("");

    write!(
        destination,
        "
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct {name};

        \
         #[allow(unsafe_code)]
        unsafe impl ::vulkano::pipeline::shader::ShaderInterfaceDef for \
         {name} {{
            type Iter = {name}Iter;
            fn elements(&self) -> {name}Iter {{
                \
         {name}Iter {{ num: 0 }}
            }}
        }}

        #[derive(Debug, Copy, Clone)]
        \
         pub struct {name}Iter {{ num: u16 }}
        impl Iterator for {name}Iter {{
            type \
         Item = ::vulkano::pipeline::shader::ShaderInterfaceDefEntry;

            #[inline]
            \
         fn next(&mut self) -> Option<Self::Item> {{
                {body}
                None
            \
         }}

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {{
                \
         let len = ({len} - self.num) as usize;
                (len, Some(len))
            }}
        \
         }}

        impl ExactSizeIterator for {name}Iter {{}}
    ",
        name = struct_name,
        body = body,
        len = attributes.len()
    )
}
