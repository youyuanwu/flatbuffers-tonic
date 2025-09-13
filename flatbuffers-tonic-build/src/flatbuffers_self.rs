// Compiles flatbuffers rust files.

use std::path::Path;

pub(crate) fn compile_flat_buffer_self(fbs_path: &Path) {
    use flatbuffers_build::BuilderOptions;
    BuilderOptions::new_with_files([fbs_path])
        .compile()
        .expect("flatbuffer compilation failed");
}
