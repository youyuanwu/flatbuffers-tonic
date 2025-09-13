fn main() {
    flatbuffers_tonic_build::compile_flat_buffer_tonic(std::path::Path::new(
        "../fbs/helloworld.fbs",
    ))
    .expect("flatbuffers tonic compilation failed");
}
