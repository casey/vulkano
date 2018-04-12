use std::io;

use spec_consts::SpecializationConstant;

pub fn write_specialization_constants(
    specialization_constants: &[SpecializationConstant],
    destination:              &io::Write,
) -> io::Result<()> {
    panic!()
}
