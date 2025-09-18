// Compiles flatbuffers rust files.

use std::path::Path;

pub(crate) fn compile_flat_buffer_self<P>(fbs_path: &[P])
where
    P: AsRef<Path>,
{
    // Use OUT_DIR/flatbuffers/<file_stem> as output path
    // flatc has a bug the multi file generation does not work correctly.
    // So we generate one by one.
    let output_path = std::env::var("OUT_DIR").unwrap();
    for path in fbs_path {
        // get the package name from the file name without extension
        let path = path.as_ref();
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let output_path = Path::new(&output_path).join("flatbuffers").join(file_stem);
        // generate for each fbs file one by one
        use flatbuffers_build::BuilderOptions;
        BuilderOptions::new_with_files([path])
            .set_output_path(&output_path)
            .compile()
            .expect("flatbuffer compilation failed");
    }
}
