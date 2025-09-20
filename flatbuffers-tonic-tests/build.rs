fn main() {
    flatbuffers_tonic_build::compile_flatbuffers_tonic(&[
        "../fbs/fbs.helloworld.fbs",
        "../fbs/sample.fbs",
    ])
    .expect("flatbuffers tonic compilation failed");
}
