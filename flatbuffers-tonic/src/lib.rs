mod codec;
pub use codec::FlatBuffersCodec;

mod wrapper;
pub use flatbuffers_util::{FBBuilder, OwnedFB};
pub use wrapper::OwnedFBCodecable;
