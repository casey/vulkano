use std::cmp;

use enums;
use fmt;

use descriptor_sets::NewDescriptor;
use types::{Type, InterfaceBlock};

pub fn descriptor_array_count(spirv_type: &Type) -> u64 {
    if let Type::Array{element_count, ..} = *spirv_type {
        element_count
    } else {
        1
    }
}

fn descriptor_read_only(_spirv_type: &Type) -> bool {
    // TODO: Ask tomaka about the function of the readonly
    // FIXME: this is wrong
    true
}

pub fn write(
    descriptors:    &[NewDescriptor],
    push_constants: &[Type],
    destination:    &mut fmt::Write,
) -> fmt::Result {
    // TODO: not implemented correctly

    let push_constants_size = push_constants.iter().map(|push_constant_type| {
        push_constant_type.rust_type()
            .expect("Found push constant with rust type")
            .size
            .expect("Found runtime-sized push constants")
    }).max().unwrap_or(0);

    // Writing the body of the `descriptor` method.
    let descriptor_body = descriptors
        .iter()
        .map(|d| {
            format!(
                "({set}, {binding}) => Some(DescriptorDesc {{
            ty: {desc_ty},
            array_count: {array_count},
            stages: self.0.clone(),
            readonly: {readonly},
        }}),",
                set = d.descriptor_set,
                binding = d.binding_point,
                desc_ty = descriptor_constructor(&d.spirv_type, false),
                array_count = descriptor_array_count(&d.spirv_type),
                readonly = descriptor_read_only(&d.spirv_type),
            )
        })
        .collect::<Vec<_>>()
        .concat();

    let num_sets = descriptors.iter().fold(0, |s, d| cmp::max(s, d.descriptor_set + 1));

    // Writing the body of the `num_bindings_in_set` method.
    let num_bindings_in_set_body = {
        (0 .. num_sets)
            .map(|set| {
                     let num = descriptors
                         .iter()
                         .filter(|d| d.descriptor_set == set)
                         .fold(0, |s, d| cmp::max(s, 1 + d.binding_point));
                     format!("{set} => Some({num}),", set = set, num = num)
                 })
            .collect::<Vec<_>>()
            .concat()
    };

    // Writing the body of the `num_push_constants_ranges` method.
    let num_push_constants_ranges_body = if push_constants_size == 0 { "0" } else { "1" };

    // Writing the body of the `push_constants_range` method.
    let push_constants_range_body = format!(
        r#"
        if num != 0 || {pc_size} == 0 {{ return None; }}
        Some(PipelineLayoutDescPcRange {{
            offset: 0,                      // FIXME: not necessarily true
            size: {pc_size},
            stages: ShaderStages::all(),     // FIXME: wrong
        }})
    "#,
        pc_size = push_constants_size
    );

    write!(
        destination,
        r#"
        #[derive(Debug, Clone)]
        pub struct Layout(pub ShaderStages);

        #[allow(unsafe_code)]
        unsafe impl PipelineLayoutDesc for Layout {{
            fn num_sets(&self) -> usize {{
                {num_sets}
            }}

            fn num_bindings_in_set(&self, set: usize) -> Option<usize> {{
                match set {{
                    {num_bindings_in_set_body}
                    _ => None
                }}
            }}

            fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {{
                match (set, binding) {{
                    {descriptor_body}
                    _ => None
                }}
            }}

            fn num_push_constants_ranges(&self) -> usize {{
                {num_push_constants_ranges_body}
            }}

            fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange> {{
                {push_constants_range_body}
            }}
        }}
        "#,
        num_sets = num_sets,
        num_bindings_in_set_body = num_bindings_in_set_body,
        descriptor_body = descriptor_body,
        num_push_constants_ranges_body = num_push_constants_ranges_body,
        push_constants_range_body = push_constants_range_body
    )
}

/// Returns a `DescriptorDescTy` constructor, a bool indicating whether the descriptor is
/// read-only, and the number of array elements.
///
/// See also section 14.5.2 of the Vulkan specs: Descriptor Set Interface
pub fn descriptor_constructor(
    spirv_type:                   &Type,
    force_combined_image_sampled: bool,
) -> String {
    match *spirv_type {
        Type::Struct{interface_block, ..} => 
        
                           format!("DescriptorDescTy::Buffer(DescriptorBufferDesc {{
                    dynamic: Some(false),
                    storage: {},
                }})", if interface_block.unwrap() == InterfaceBlock::BufferBlock { "true" } else { "false "}),
        Type::Image{ref dim, arrayed, ms, sampled, ref format, ..} => {
            let sampled = sampled.expect("Vulkan requires that variables of type OpTypeImage \
                                            have a Sampled operand of 1 or 2");

            let ms = if ms { "true" } else { "false" };
            let arrayed = if arrayed {
                "DescriptorImageDescArray::Arrayed { max_layers: None }"
            } else {
                "DescriptorImageDescArray::NonArrayed"
            };

            if let &enums::Dim::DimSubpassData = dim {
                // We are an input attachment.
                assert!(!force_combined_image_sampled, "An OpTypeSampledImage can't point to \
                                                        an OpTypeImage whose dimension is \
                                                        SubpassData");
                assert!(if let &enums::ImageFormat::ImageFormatUnknown = format { true }
                        else { false }, "If Dim is SubpassData, Image Format must be Unknown");
                assert!(!sampled, "If Dim is SubpassData, Sampled must be 2");

                let desc = format!("DescriptorDescTy::InputAttachment {{
                                        multisampled: {},
                                        array_layers: {}
                                    }}", ms, arrayed);

                desc
            } else if let &enums::Dim::DimBuffer = dim {
                // We are a texel buffer.
                let desc = format!("DescriptorDescTy::TexelBuffer {{
                    storage: {},
                    format: None,       // TODO: specify format if known
                }}", !sampled);

                desc

            } else {
                // We are a sampled or storage image.
                let sampled = if sampled { "true" } else { "false" };
                let ty = if force_combined_image_sampled { "CombinedImageSampler" }
                            else { "Image" };
                let dim = match *dim {
                    enums::Dim::Dim1D => "DescriptorImageDescDimensions::OneDimensional",
                    enums::Dim::Dim2D => "DescriptorImageDescDimensions::TwoDimensional",
                    enums::Dim::Dim3D => "DescriptorImageDescDimensions::ThreeDimensional",
                    enums::Dim::DimCube => "DescriptorImageDescDimensions::Cube",
                    enums::Dim::DimRect => panic!("Vulkan doesn't support rectangle textures"),
                    _ => unreachable!()
                };

                let desc = format!("DescriptorDescTy::{}(DescriptorImageDesc {{
                        sampled: {},
                        dimensions: {},
                        format: None,       // TODO: specify format if known
                        multisampled: {},
                        array_layers: {},
                    }})", ty, sampled, dim, ms, arrayed);

                desc
            }
        }
        Type::SampledImage{ref image_type} => descriptor_constructor(image_type, true),
        Type::Sampler{} => "DescriptorDescTy::Sampler".to_string(),
        Type::Array{ref element_type, ..} => {
            if let Type::Array{..} = *element_type.clone() {
                // TODO: Implement arrays containing arrays, and remove this extra clone.
                panic!()
            }

            descriptor_constructor(element_type, false)
        }
        _ => panic!(),
    }
}
