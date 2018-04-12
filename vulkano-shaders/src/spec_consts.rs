// Copyright (c) 2017 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

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
