
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
    
pub struct Shader {
    shader: ::std::sync::Arc<::vulkano::pipeline::shader::ShaderModule>,
}

impl Shader {
    /// Loads the shader in Vulkan as a `ShaderModule`.
    #[inline]
    #[allow(unsafe_code)]
    pub fn load(device: ::std::sync::Arc<::vulkano::device::Device>)
                -> Result<Shader, ::vulkano::OomError>
    {

        
        unsafe {
            let data = [3, 2, 35, 7, 0, 0, 1, 0, 1, 0, 8, 0, 43, 0, 0, 0, 0, 0, 0, 0, 17, 0, 2, 0, 1, 0, 0, 0, 11, 0, 6, 0, 1, 0, 0, 0, 71, 76, 83, 76, 46, 115, 116, 100, 46, 52, 53, 48, 0, 0, 0, 0, 14, 0, 3, 0, 0, 0, 0, 0, 1, 0, 0, 0, 15, 0, 7, 0, 4, 0, 0, 0, 4, 0, 0, 0, 109, 97, 105, 110, 0, 0, 0, 0, 29, 0, 0, 0, 37, 0, 0, 0, 16, 0, 3, 0, 4, 0, 0, 0, 7, 0, 0, 0, 3, 0, 3, 0, 2, 0, 0, 0, 194, 1, 0, 0, 5, 0, 4, 0, 4, 0, 0, 0, 109, 97, 105, 110, 0, 0, 0, 0, 5, 0, 3, 0, 11, 0, 0, 0, 83, 0, 0, 0, 6, 0, 5, 0, 11, 0, 0, 0, 0, 0, 0, 0, 118, 97, 108, 49, 0, 0, 0, 0, 6, 0, 5, 0, 11, 0, 0, 0, 1, 0, 0, 0, 118, 97, 108, 50, 0, 0, 0, 0, 5, 0, 4, 0, 12, 0, 0, 0, 66, 108, 111, 99, 107, 0, 0, 0, 6, 0, 5, 0, 12, 0, 0, 0, 0, 0, 0, 0, 117, 95, 100, 97, 116, 97, 0, 0, 5, 0, 4, 0, 14, 0, 0, 0, 98, 108, 111, 99, 107, 0, 0, 0, 5, 0, 4, 0, 18, 0, 0, 0, 105, 110, 100, 101, 120, 0, 0, 0, 5, 0, 4, 0, 29, 0, 0, 0, 102, 95, 99, 111, 108, 111, 114, 0, 5, 0, 5, 0, 33, 0, 0, 0, 117, 95, 116, 101, 120, 116, 117, 114, 101, 0, 0, 0, 5, 0, 5, 0, 37, 0, 0, 0, 118, 95, 116, 101, 120, 99, 111, 111, 114, 100, 115, 0, 71, 0, 4, 0, 10, 0, 0, 0, 6, 0, 0, 0, 16, 0, 0, 0, 72, 0, 5, 0, 11, 0, 0, 0, 0, 0, 0, 0, 35, 0, 0, 0, 0, 0, 0, 0, 72, 0, 5, 0, 11, 0, 0, 0, 1, 0, 0, 0, 35, 0, 0, 0, 16, 0, 0, 0, 72, 0, 5, 0, 12, 0, 0, 0, 0, 0, 0, 0, 35, 0, 0, 0, 0, 0, 0, 0, 71, 0, 3, 0, 12, 0, 0, 0, 2, 0, 0, 0, 71, 0, 4, 0, 14, 0, 0, 0, 34, 0, 0, 0, 0, 0, 0, 0, 71, 0, 4, 0, 14, 0, 0, 0, 33, 0, 0, 0, 1, 0, 0, 0, 71, 0, 4, 0, 18, 0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0, 71, 0, 4, 0, 29, 0, 0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 71, 0, 4, 0, 33, 0, 0, 0, 34, 0, 0, 0, 0, 0, 0, 0, 71, 0, 4, 0, 33, 0, 0, 0, 33, 0, 0, 0, 0, 0, 0, 0, 71, 0, 4, 0, 37, 0, 0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 19, 0, 2, 0, 2, 0, 0, 0, 33, 0, 3, 0, 3, 0, 0, 0, 2, 0, 0, 0, 22, 0, 3, 0, 6, 0, 0, 0, 32, 0, 0, 0, 23, 0, 4, 0, 7, 0, 0, 0, 6, 0, 0, 0, 3, 0, 0, 0, 21, 0, 4, 0, 8, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 43, 0, 4, 0, 8, 0, 0, 0, 9, 0, 0, 0, 5, 0, 0, 0, 28, 0, 4, 0, 10, 0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0, 30, 0, 4, 0, 11, 0, 0, 0, 7, 0, 0, 0, 10, 0, 0, 0, 30, 0, 3, 0, 12, 0, 0, 0, 11, 0, 0, 0, 32, 0, 4, 0, 13, 0, 0, 0, 2, 0, 0, 0, 12, 0, 0, 0, 59, 0, 4, 0, 13, 0, 0, 0, 14, 0, 0, 0, 2, 0, 0, 0, 21, 0, 4, 0, 15, 0, 0, 0, 32, 0, 0, 0, 1, 0, 0, 0, 43, 0, 4, 0, 15, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 43, 0, 4, 0, 15, 0, 0, 0, 17, 0, 0, 0, 1, 0, 0, 0, 50, 0, 4, 0, 15, 0, 0, 0, 18, 0, 0, 0, 2, 0, 0, 0, 32, 0, 4, 0, 19, 0, 0, 0, 2, 0, 0, 0, 8, 0, 0, 0, 20, 0, 2, 0, 22, 0, 0, 0, 43, 0, 4, 0, 8, 0, 0, 0, 23, 0, 0, 0, 0, 0, 0, 0, 23, 0, 4, 0, 27, 0, 0, 0, 6, 0, 0, 0, 4, 0, 0, 0, 32, 0, 4, 0, 28, 0, 0, 0, 3, 0, 0, 0, 27, 0, 0, 0, 59, 0, 4, 0, 28, 0, 0, 0, 29, 0, 0, 0, 3, 0, 0, 0, 25, 0, 9, 0, 30, 0, 0, 0, 6, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 27, 0, 3, 0, 31, 0, 0, 0, 30, 0, 0, 0, 32, 0, 4, 0, 32, 0, 0, 0, 0, 0, 0, 0, 31, 0, 0, 0, 59, 0, 4, 0, 32, 0, 0, 0, 33, 0, 0, 0, 0, 0, 0, 0, 23, 0, 4, 0, 35, 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, 0, 32, 0, 4, 0, 36, 0, 0, 0, 1, 0, 0, 0, 35, 0, 0, 0, 59, 0, 4, 0, 36, 0, 0, 0, 37, 0, 0, 0, 1, 0, 0, 0, 43, 0, 4, 0, 6, 0, 0, 0, 41, 0, 0, 0, 0, 0, 128, 63, 44, 0, 7, 0, 27, 0, 0, 0, 42, 0, 0, 0, 41, 0, 0, 0, 41, 0, 0, 0, 41, 0, 0, 0, 41, 0, 0, 0, 54, 0, 5, 0, 2, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 248, 0, 2, 0, 5, 0, 0, 0, 65, 0, 7, 0, 19, 0, 0, 0, 20, 0, 0, 0, 14, 0, 0, 0, 16, 0, 0, 0, 17, 0, 0, 0, 18, 0, 0, 0, 61, 0, 4, 0, 8, 0, 0, 0, 21, 0, 0, 0, 20, 0, 0, 0, 171, 0, 5, 0, 22, 0, 0, 0, 24, 0, 0, 0, 21, 0, 0, 0, 23, 0, 0, 0, 247, 0, 3, 0, 26, 0, 0, 0, 0, 0, 0, 0, 250, 0, 4, 0, 24, 0, 0, 0, 25, 0, 0, 0, 40, 0, 0, 0, 248, 0, 2, 0, 25, 0, 0, 0, 61, 0, 4, 0, 31, 0, 0, 0, 34, 0, 0, 0, 33, 0, 0, 0, 61, 0, 4, 0, 35, 0, 0, 0, 38, 0, 0, 0, 37, 0, 0, 0, 87, 0, 5, 0, 27, 0, 0, 0, 39, 0, 0, 0, 34, 0, 0, 0, 38, 0, 0, 0, 62, 0, 3, 0, 29, 0, 0, 0, 39, 0, 0, 0, 249, 0, 2, 0, 26, 0, 0, 0, 248, 0, 2, 0, 40, 0, 0, 0, 62, 0, 3, 0, 29, 0, 0, 0, 42, 0, 0, 0, 249, 0, 2, 0, 26, 0, 0, 0, 248, 0, 2, 0, 26, 0, 0, 0, 253, 0, 1, 0, 56, 0, 1, 0];

            Ok(Shader {
                shader: try!(::vulkano::pipeline::shader::ShaderModule::new(device, &data))
            })
        }
    }

    /// Returns the module that was created.
    #[allow(dead_code)]
    #[inline]
    pub fn module(&self) -> &::std::sync::Arc<::vulkano::pipeline::shader::ShaderModule> {
        &self.shader
    }
        
    /// Returns a logical struct describing the entry point named `main`.
    #[inline]
    #[allow(unsafe_code)]
    pub fn main_entry_point(&self) -> ::vulkano::pipeline::shader::GraphicsEntryPoint<SpecializationConstants, MainInput, MainOutput, Layout> {
        unsafe {
            #[allow(dead_code)]
            static NAME: [u8; 5] = [109, 97, 105, 110, 0];     // "main"
            self.shader.graphics_entry_point(::std::ffi::CStr::from_ptr(NAME.as_ptr() as *const _), MainInput, MainOutput, Layout(ShaderStages { fragment: true, .. ShaderStages::none() }), ::vulkano::pipeline::shader::GraphicsShaderType::Fragment)
        }
    }
            
}
        
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct MainInput;

        #[allow(unsafe_code)]
        unsafe impl ::vulkano::pipeline::shader::ShaderInterfaceDef for MainInput {
            type Iter = MainInputIter;
            fn elements(&self) -> MainInputIter {
                MainInputIter { num: 0 }
            }
        }

        #[derive(Debug, Copy, Clone)]
        pub struct MainInputIter { num: u16 }
        impl Iterator for MainInputIter {
            type Item = ::vulkano::pipeline::shader::ShaderInterfaceDefEntry;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.num == 0 {
            self.num += 1;

            return Some(::vulkano::pipeline::shader::ShaderInterfaceDefEntry {
                location: 0 .. 1,
                format: ::vulkano::format::Format::R32G32Sfloat,
                name: Some(::std::borrow::Cow::Borrowed("v_texcoords"))
            });
        }
                None
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = (1 - self.num) as usize;
                (len, Some(len))
            }
        }

        impl ExactSizeIterator for MainInputIter {}
    
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct MainOutput;

        #[allow(unsafe_code)]
        unsafe impl ::vulkano::pipeline::shader::ShaderInterfaceDef for MainOutput {
            type Iter = MainOutputIter;
            fn elements(&self) -> MainOutputIter {
                MainOutputIter { num: 0 }
            }
        }

        #[derive(Debug, Copy, Clone)]
        pub struct MainOutputIter { num: u16 }
        impl Iterator for MainOutputIter {
            type Item = ::vulkano::pipeline::shader::ShaderInterfaceDefEntry;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.num == 0 {
            self.num += 1;

            return Some(::vulkano::pipeline::shader::ShaderInterfaceDefEntry {
                location: 0 .. 1,
                format: ::vulkano::format::Format::R32G32B32A32Sfloat,
                name: Some(::std::borrow::Cow::Borrowed("f_color"))
            });
        }
                None
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = (1 - self.num) as usize;
                (len, Some(len))
            }
        }

        impl ExactSizeIterator for MainOutputIter {}
    pub mod ty {#[repr(C)]
#[derive(Copy)]
#[allow(non_snake_case)]
pub struct S {
    pub val1: [f32; 3] /* offset: 0 */,
    pub _dummy0: [u8; 4] ,
    pub val2: [u32; 5] /* offset: 16 */
} /* total_size: None */

impl Clone for S {
    fn clone(&self) -> Self {
        S {
            val1: self.val1,
            _dummy0: self._dummy0,
            val2: self.val2
        }
    }
}

#[repr(C)]
#[derive(Copy)]
#[allow(non_snake_case)]
pub struct Block {
    pub u_data: S /* offset: 0 */
} /* total_size: None */

impl Clone for Block {
    fn clone(&self) -> Self {
        Block {
            u_data: self.u_data
        }
    }
}

}
        #[derive(Debug, Clone)]
        pub struct Layout(pub ShaderStages);

        #[allow(unsafe_code)]
        unsafe impl PipelineLayoutDesc for Layout {
            fn num_sets(&self) -> usize {
                1
            }

            fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
                match set {
                    0 => Some(2),
                    _ => None
                }
            }

            fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
                match (set, binding) {
                    (0, 1) => Some(DescriptorDesc {
            ty: DescriptorDescTy::Buffer(DescriptorBufferDesc {
                    dynamic: Some(false),
                    storage: false ,
                }),
            array_count: 1,
            stages: self.0.clone(),
            readonly: true,
        }),(0, 0) => Some(DescriptorDesc {
            ty: DescriptorDescTy::CombinedImageSampler(DescriptorImageDesc {
                        sampled: true,
                        dimensions: DescriptorImageDescDimensions::TwoDimensional,
                        format: None,       // TODO: specify format if known
                        multisampled: false,
                        array_layers: DescriptorImageDescArray::NonArrayed,
                    }),
            array_count: 1,
            stages: self.0.clone(),
            readonly: true,
        }),
                    _ => None
                }
            }

            fn num_push_constants_ranges(&self) -> usize {
                0
            }

            fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange> {
                
        if num != 0 || 0 == 0 { return None; }
        Some(PipelineLayoutDescPcRange {
            offset: 0,                      // FIXME: not necessarily true
            size: 0,
            stages: ShaderStages::all(),     // FIXME: wrong
        })
    
            }
        }
        

#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct SpecializationConstants {
    pub index: i32
}

impl Default for SpecializationConstants {
    fn default() -> SpecializationConstants {
        SpecializationConstants {
            index: unsafe { ::std::mem::transmute([2u32]) }
        }
    }
}

unsafe impl SpecConstsTrait for SpecializationConstants {
    fn descriptors() -> &'static [SpecializationMapEntry] {
        static DESCRIPTORS: [SpecializationMapEntry; 1] = [
            SpecializationMapEntry {
                constant_id: 5,
                offset: 0,
                size: 4,
            }
        ];
        &DESCRIPTORS
    }
}

    
