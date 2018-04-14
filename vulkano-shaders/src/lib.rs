// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

extern crate glsl_to_spirv;

use std::env;
use std::fs;
use std::fs::File;
use std::io::Error as IoError;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub use glsl_to_spirv::ShaderType;
pub use parse::ParseError;

mod descriptor_sets;
mod entry_point;
mod enums;
mod parse;
mod spec_consts;
mod structs;
mod shader;
mod types;
mod codegen;

use shader::Shader;
use entry_point::EntryPoint;

pub fn build_glsl_shaders<'a, I>(shaders: I)
    where I: IntoIterator<Item = (&'a str, ShaderType)>
{
    let destination = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&destination);

    let shaders = shaders.into_iter().collect::<Vec<_>>();
    for &(shader, _) in &shaders {
        // Run this first so that a panic won't interfere with rerun
        println!("cargo:rerun-if-changed={}", shader);
    }

    for (shader, ty) in shaders {
        let shader = Path::new(shader);

        let shader_content = {
            let mut s = String::new();
            File::open(shader)
                .expect("failed to open shader")
                .read_to_string(&mut s)
                .expect("failed to read shader content");
            s
        };

        fs::create_dir_all(&destination.join("shaders").join(shader.parent().unwrap())).unwrap();
        let mut file_output = File::create(&destination.join("shaders").join(shader))
            .expect("failed to open shader output");

        let content = match glsl_to_spirv::compile(&shader_content, ty) {
            Ok(compiled) => compiled,
            Err(message) => panic!("{}\nfailed to compile shader", message),
        };

        let output = reflect("Shader", content).unwrap();
        write!(file_output, "{}", output).unwrap();
    }
}

pub fn reflect<R: Read>(name: &str, mut spirv_reader: R) -> Result<String, Error> {
    let mut data = Vec::new();
    spirv_reader.read_to_end(&mut data)?;

    // now parsing the document
    let spirv = parse::parse_spirv(&data)?;

    let shader = Shader::from_spirv(spirv);

    let mut output = String::new();
    output.push_str(
        r#"
        #[allow(unused_imports)]
        use std::sync::Arc;
        #[allow(unused_imports)]
        use std::vec::IntoIter as VecIntoIter;

        #[allow(unused_imports)]
        use vulkano::device::Device;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::DescriptorDesc;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::DescriptorDescTy;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::DescriptorBufferDesc;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::DescriptorImageDesc;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::DescriptorImageDescDimensions;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::DescriptorImageDescArray;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor::ShaderStages;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor_set::DescriptorSet;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor_set::UnsafeDescriptorSet;
        #[allow(unused_imports)]
        use vulkano::descriptor::descriptor_set::UnsafeDescriptorSetLayout;
        #[allow(unused_imports)]
        use vulkano::descriptor::pipeline_layout::PipelineLayout;
        #[allow(unused_imports)]
        use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;
        #[allow(unused_imports)]
        use vulkano::descriptor::pipeline_layout::PipelineLayoutDescPcRange;
        #[allow(unused_imports)]
        use vulkano::pipeline::shader::SpecializationConstants as SpecConstsTrait;
        #[allow(unused_imports)]
        use vulkano::pipeline::shader::SpecializationMapEntry;
    "#,
    );

    {
        // contains the data that was passed as input to this function
        let spirv_data = data.iter()
            .map(|&byte| byte.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        // writing the header
        output.push_str(&format!(
            r#"
pub struct {name} {{
    shader: ::std::sync::Arc<::vulkano::pipeline::shader::ShaderModule>,
}}

impl {name} {{
    /// Loads the shader in Vulkan as a `ShaderModule`.
    #[inline]
    #[allow(unsafe_code)]
    pub fn load(device: ::std::sync::Arc<::vulkano::device::Device>)
                -> Result<{name}, ::vulkano::OomError>
    {{

        "#,
            name = name
        ));

        // checking whether each required capability is enabled in the vulkan device
        for capability in &shader.capabilities {
            if let Some(capability) = capability.name() {
                output.push_str(&format!(
                    r#"
                    if !device.enabled_features().{capability} {{
                        panic!("capability {{:?}} not enabled", "{capability}")  // FIXME: error
                        //return Err(CapabilityNotEnabled);
                    }}"#,
                    capability = capability
                ));
            }
        }

        // follow-up of the header
        output.push_str(&format!(
            r#"
        unsafe {{
            let data = [{spirv_data}];

            Ok({name} {{
                shader: try!(::vulkano::pipeline::shader::ShaderModule::new(device, &data))
            }})
        }}
    }}

    /// Returns the module that was created.
    #[allow(dead_code)]
    #[inline]
    pub fn module(&self) -> &::std::sync::Arc<::vulkano::pipeline::shader::ShaderModule> {{
        &self.shader
    }}
        "#,
            name = name,
            spirv_data = spirv_data
        ));
        
        let entry_points = shader.entry_points
            .iter().cloned().collect::<Vec<EntryPoint>>();

        codegen::entry_points::write_entry_points(
            &entry_points, 
            !shader.specialization_constants.is_empty(),
            &mut output,
        )?;

        // footer
        output.push_str(&format!(
            r#"
}}
        "#
        ));

        codegen::entry_points::write_interface_structs(&entry_points, &mut output)?;

        // struct definitions
        output.push_str("pub mod ty {");
        output.push_str(&structs::write_structs(&shader));
        output.push_str("}");

        // descriptor sets
        output.push_str(&descriptor_sets::write_descriptor_sets(&shader.spirv));

        codegen::specialization_constants::write(&shader.specialization_constants, &mut output)?;
    }

    Ok(output)
}

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    ParseError(ParseError),
    FmtError(fmt::Error),
}

impl From<IoError> for Error {
    #[inline]
    fn from(err: IoError) -> Error {
        Error::IoError(err)
    }
}

impl From<ParseError> for Error {
    #[inline]
    fn from(err: ParseError) -> Error {
        Error::ParseError(err)
    }
}

impl From<fmt::Error> for Error {
    #[inline]
    fn from(err: fmt::Error) -> Error {
        Error::FmtError(err)
    }
}

fn name_from_id(doc: &parse::Spirv, searched: u32) -> String {
    doc.instructions
        .iter()
        .filter_map(|i| if let &parse::Instruction::Name {
            target_id,
            ref name,
        } = i
        {
            if target_id == searched {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        })
        .next()
        .and_then(|n| if !n.is_empty() { Some(n) } else { None })
        .unwrap_or("__unnamed".to_owned())
}

fn member_name_from_id(doc: &parse::Spirv, searched: u32, searched_member: u32) -> String {
    doc.instructions
        .iter()
        .filter_map(|i| if let &parse::Instruction::MemberName {
            target_id,
            member,
            ref name,
        } = i
        {
            if target_id == searched && member == searched_member {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        })
        .next()
        .and_then(|n| if !n.is_empty() { Some(n) } else { None })
        .unwrap_or("__unnamed".to_owned())
}

/// Returns true if a `BuiltIn` decorator is applied on an id.
/// TODO: Why does this also return true when the type is built in?
fn _is_builtin(doc: &parse::Spirv, id: u32) -> bool {
    for instruction in &doc.instructions {
        match *instruction {
            parse::Instruction::Decorate {
                target_id,
                decoration: enums::Decoration::DecorationBuiltIn,
                ..
            } if target_id == id => {
                return true;
            },
            parse::Instruction::MemberDecorate {
                target_id,
                decoration: enums::Decoration::DecorationBuiltIn,
                ..
            } if target_id == id => {
                return true;
            },
            _ => (),
        }
    }

    for instruction in &doc.instructions {
        match *instruction {
            parse::Instruction::Variable {
                result_type_id,
                result_id,
                ..
            } if result_id == id => {
                return _is_builtin(doc, result_type_id);
            },
            parse::Instruction::TypeArray { result_id, type_id, .. } if result_id == id => {
                return _is_builtin(doc, type_id);
            },
            parse::Instruction::TypeRuntimeArray { result_id, type_id } if result_id == id => {
                return _is_builtin(doc, type_id);
            },
            parse::Instruction::TypeStruct {
                result_id,
                ref member_types,
            } if result_id == id => {
                for &mem in member_types {
                    if _is_builtin(doc, mem) {
                        return true;
                    }
                }
            },
            parse::Instruction::TypePointer { result_id, type_id, .. } if result_id == id => {
                return _is_builtin(doc, type_id);
            },
            _ => (),
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactor_is_noop() {
        let content = glsl_to_spirv::compile(
            include_str!("../data/example-shader.glsl"),
            glsl_to_spirv::ShaderType::Fragment
        ).unwrap();

        let actual = reflect("Shader", content).unwrap();
        let actual_lines = actual.lines().collect::<Vec<_>>();
        let expected_lines = include_str!("../data/pre-refactor-shader.rs").lines().collect::<Vec<_>>();

        assert_eq!(actual_lines.len(), expected_lines.len());

        for (i, (actual, expected)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
            if actual != expected {
                println!("line {} has changed", i);
                assert_eq!(actual, expected);
            }
        }
    }
}
