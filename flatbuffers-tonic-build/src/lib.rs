use std::path::Path;

pub(crate) mod flatbuffers_self;
pub(crate) mod flatbuffers_tonic;

pub fn compile_flat_buffer_tonic(fbs_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Compile flatbuffers first
    flatbuffers_self::compile_flat_buffer_self(fbs_path);

    // Then compile tonic
    flatbuffers_tonic::compile_flatbuffers_tonic_only(fbs_path)?;
    Ok(())
}
