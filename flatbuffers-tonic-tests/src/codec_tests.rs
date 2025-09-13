use tonic::codec::Codec;

// The reason to have wrapper struct is the inner struct has a lifetime
// and it makes tonic generated code not compile. Simple struct wrapper
// makes the type also explicit.
mod wrappers {
    use flatbuffers_tonic::OwnedFBCodecable;

    use crate::helloworld_gen::helloworld;

    pub struct OwnedHelloRequest(
        pub flatbuffers_util::ownedfb::OwnedFB<helloworld::HelloRequest<'static>>,
    );

    impl OwnedFBCodecable for OwnedHelloRequest {
        fn new_boxed(buf: Box<[u8]>) -> Result<Self, flatbuffers::InvalidFlatbuffer> {
            let owned =
                flatbuffers_util::ownedfb::OwnedFB::<helloworld::HelloRequest>::new_boxed(buf)?;
            Ok(OwnedHelloRequest(owned))
        }

        fn get_slice(&self) -> &[u8] {
            self.0.get_slice()
        }
    }
}

#[test]
fn codec_test() {
    let mut codec = flatbuffers_tonic::FlatBuffersCodec::<
        wrappers::OwnedHelloRequest,
        wrappers::OwnedHelloRequest,
    >::new();
    let mut _encoder = codec.encoder();
    let mut _decoder = codec.decoder();
}
