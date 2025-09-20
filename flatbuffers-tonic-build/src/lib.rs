use std::path::Path;

pub(crate) mod flatbuffers_self;
pub(crate) mod flatbuffers_tonic;

/// Currently assumes fbs files are independent.
pub fn compile_flatbuffers_tonic<P>(fbs_path: &[P]) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    // Compile flatbuffers first
    flatbuffers_self::compile_flat_buffer_self(fbs_path);

    // Then compile tonic
    flatbuffers_tonic::compile_flatbuffers_tonic_file_list_only(fbs_path)?;
    Ok(())
}
