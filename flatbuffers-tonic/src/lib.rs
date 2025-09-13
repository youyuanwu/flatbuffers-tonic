mod codec;
pub use codec::FlatBuffersCodec;

mod wrapper;
pub use flatbuffers_util::ownedfb::OwnedFB;
pub use wrapper::OwnedFBCodecable;
