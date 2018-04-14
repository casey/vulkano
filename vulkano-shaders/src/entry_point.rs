// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use enums::ExecutionModel;
use types::Type;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct EntryPoint {
    pub execution_model: ExecutionModel,
    pub id:              u32,
    // TODO: remove this
    pub interface_ids:   Vec<u32>,
    pub name:            String,
    pub inputs:          Vec<InterfaceVariable>,
    pub outputs:         Vec<InterfaceVariable>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InterfaceVariable {
    pub name:       String,
    pub spirv_type: Type,
    pub location:   u32,
}
