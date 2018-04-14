// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::fmt;

use enums;
use parse;
use codegen;

use format_from_id;
use is_builtin;
use location_decoration;
use name_from_id;

use enums::ExecutionModel;
use shader::Shader;
use entry_point::EntryPoint;

pub fn write_entry_points(shader: &Shader, destination: &mut fmt::Write) -> fmt::Result {
    for entry_point in &shader.entry_points {
        codegen::entry_points::write_entry_point(&shader, entry_point, destination)?;
    }
    Ok(())
}

fn write_entry_point(
    shader: &Shader,
    entry_point: &EntryPoint,
    destination: &mut fmt::Write,
) -> fmt::Result {
    let doc = &shader.spirv;
    let &EntryPoint{execution_model: execution, id, name: ref ep_name, ..} = entry_point;

    let capitalized_ep_name = codegen::capitalize(ep_name);

    let spec_consts_struct = if !shader.specialization_constants.is_empty() {
        "SpecializationConstants"
    } else {
        "()"
    };

    let (ty, f_call) = {
        if let enums::ExecutionModel::ExecutionModelGLCompute = execution {
            (format!("::vulkano::pipeline::shader::ComputeEntryPoint<{}, Layout>",
                     spec_consts_struct),
             format!("compute_entry_point(::std::ffi::CStr::from_ptr(NAME.as_ptr() as *const _), \
                      Layout(ShaderStages {{ compute: true, .. ShaderStages::none() }}))"))

        } else {
            let ty = match execution {
                enums::ExecutionModel::ExecutionModelVertex => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::Vertex".to_owned()
                },

                enums::ExecutionModel::ExecutionModelTessellationControl => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::TessellationControl"
                        .to_owned()
                },

                enums::ExecutionModel::ExecutionModelTessellationEvaluation => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::TessellationEvaluation"
                        .to_owned()
                },

                enums::ExecutionModel::ExecutionModelGeometry => {
                    let mut execution_mode = None;

                    for instruction in doc.instructions.iter() {
                        if let &parse::Instruction::ExecutionMode {
                            target_id,
                            ref mode,
                            ..
                        } = instruction
                        {
                            if target_id != id {
                                continue;
                            }
                            execution_mode = match mode {
                                &enums::ExecutionMode::ExecutionModeInputPoints => Some("Points"),
                                &enums::ExecutionMode::ExecutionModeInputLines => Some("Lines"),
                                &enums::ExecutionMode::ExecutionModeInputLinesAdjacency =>
                                    Some("LinesWithAdjacency"),
                                &enums::ExecutionMode::ExecutionModeTriangles => Some("Triangles"),
                                &enums::ExecutionMode::ExecutionModeInputTrianglesAdjacency =>
                                    Some("TrianglesWithAdjacency"),
                                _ => continue,
                            };
                            break;
                        }
                    }

                    format!(
                        "::vulkano::pipeline::shader::GraphicsShaderType::Geometry(
                        \
                         ::vulkano::pipeline::shader::GeometryShaderExecutionMode::{0}
                    \
                         )",
                        execution_mode.unwrap()
                    )
                },

                enums::ExecutionModel::ExecutionModelFragment => {
                    "::vulkano::pipeline::shader::GraphicsShaderType::Fragment".to_owned()
                },

                enums::ExecutionModel::ExecutionModelGLCompute => {
                    unreachable!()
                },

                enums::ExecutionModel::ExecutionModelKernel => panic!("Kernels are not supported"),
            };

            let stage = match execution {
                enums::ExecutionModel::ExecutionModelVertex => {
                    "ShaderStages { vertex: true, .. ShaderStages::none() }"
                },
                enums::ExecutionModel::ExecutionModelTessellationControl => {
                    "ShaderStages { tessellation_control: true, .. ShaderStages::none() }"
                },
                enums::ExecutionModel::ExecutionModelTessellationEvaluation => {
                    "ShaderStages { tessellation_evaluation: true, .. ShaderStages::none() }"
                },
                enums::ExecutionModel::ExecutionModelGeometry => {
                    "ShaderStages { geometry: true, .. ShaderStages::none() }"
                },
                enums::ExecutionModel::ExecutionModelFragment => {
                    "ShaderStages { fragment: true, .. ShaderStages::none() }"
                },
                enums::ExecutionModel::ExecutionModelGLCompute => unreachable!(),
                enums::ExecutionModel::ExecutionModelKernel => unreachable!(),
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
        ep_name = ep_name,
        ep_name_lenp1 = ep_name.chars().count() + 1,
        ty = ty,
        encoded_ep_name = ep_name
            .chars()
            .map(|c| (c as u32).to_string())
            .collect::<Vec<String>>()
            .join(", "),
        f_call = f_call
    )
}

pub fn write_interface_structs(shader: &Shader, destination: &mut fmt::Write) -> fmt::Result {
    for entry_point in &shader.entry_points {
        let doc = &shader.spirv;
        let interface_structs = write_entry_point_interface_structs(entry_point, doc);
        write!(destination, "{}", interface_structs)?;
    }
    Ok(())
}

fn write_entry_point_interface_structs(
    entry_point: &EntryPoint,
    doc:         &parse::Spirv,
) -> String {
    let capitalized_name = codegen::capitalize(&entry_point.name);

    let ignore_first_array_in = match entry_point.execution_model {
        ExecutionModel::ExecutionModelTessellationControl |
        ExecutionModel::ExecutionModelTessellationEvaluation |
        ExecutionModel::ExecutionModelGeometry => true,
        _ => false,
    };

    let ignore_first_array_out =
        entry_point.execution_model == ExecutionModel::ExecutionModelTessellationControl;

    let mut input_elements = Vec::new();
    let mut output_elements = Vec::new();

    /*
    // Filling `input_elements` and `output_elements`.
    for interface in entry_point.interface_ids.iter() {
        for i in doc.instructions.iter() {
            match i {
                &parse::Instruction::Variable {
                    result_type_id,
                    result_id,
                    ref storage_class,
                    ..
                } if &result_id == interface => {
                    if is_builtin(doc, result_id) {
                        continue;
                    }

                    let (to_write, ignore_first_array) = match storage_class {
                        &enums::StorageClass::StorageClassInput => (&mut input_elements,
                                                                    ignore_first_array_in),
                        &enums::StorageClass::StorageClassOutput => (&mut output_elements,
                                                                     ignore_first_array_out),
                        _ => continue,
                    };

                    let name = name_from_id(doc, result_id);

                    if name == "__unnamed" {
                        continue;
                    } // FIXME: hack

                    let loc = match location_decoration(doc, result_id) {
                        Some(l) => l,
                        None => panic!("Attribute `{}` (id {}) is missing a location",
                                       name,
                                       result_id),
                    };

                    to_write
                        .push((loc, name, format_from_id(doc, result_type_id, ignore_first_array)));
                },
                _ => (),
            }
        }
    }
    */

    for input in &entry_point.inputs {
        let format = input.spirv_type.format();

        if format.is_none() {
            panic!("entry point input without format: {:?}", input);
        }

        input_elements.push((
            input.location,
            input.name.clone(),
            (input.spirv_type.format().unwrap(), input.spirv_type.occupied_locations().unwrap()),
        ));
    }

    for output in &entry_point.outputs {
        let format = output.spirv_type.format();

        if format.is_none() {
            panic!("entry point output without format: {:?}", output);
        }

        output_elements.push((
            output.location,
            output.name.clone(),
            (output.spirv_type.format().unwrap(), output.spirv_type.occupied_locations().unwrap()),
        ));
    }

    // location, name, vulkano format, number of occupied locations

    write_interface_struct(&format!("{}Input", capitalized_name), &input_elements) +
        &write_interface_struct(&format!("{}Output", capitalized_name), &output_elements)
}

fn write_interface_struct(struct_name: &str, attributes: &[(u32, String, (String, usize))])
                          -> String {
    // Checking for overlapping elements.
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

    format!(
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
