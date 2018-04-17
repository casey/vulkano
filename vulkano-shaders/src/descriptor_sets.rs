// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use types::Type;

pub struct NewDescriptor {
    pub descriptor_set: u32,
    pub binding_point:  u32,
    pub spirv_type:     Type,
    pub name:           String,
    // FIXME: delet this
    pub type_id:        u32,
}
