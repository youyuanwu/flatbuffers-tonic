use flatbuffers::FlatBufferBuilder;
use flatbuffers_util::ownedfb::OwnedFB;

use crate::helloworld_gen::helloworld::{HelloRequest, HelloRequestArgs};

#[test]
fn fbs_test() {
    let mut builder = FlatBufferBuilder::new();

    let bar_str = builder.create_string("hello world");

    let req = HelloRequest::create(
        &mut builder,
        &HelloRequestArgs {
            name: Some(bar_str),
        },
    );
    builder.finish_minimal(req);

    let req = builder.finished_data();

    let boxed_buff = req.to_owned().into_boxed_slice();
    {
        let req_x = flatbuffers::root::<HelloRequest>(&boxed_buff).unwrap();
        assert_eq!(req_x.name(), Some("hello world"));
    }
    {
        let owned = OwnedFB::<HelloRequest>::new(&boxed_buff);
        let req_x = owned.get_ref();
        assert_eq!(req_x.name(), Some("hello world"));
    }
    {
        let owned = OwnedFB::<HelloRequest>::new_boxed(boxed_buff).unwrap();
        let req_x = owned.get_ref();
        assert_eq!(req_x.name(), Some("hello world"));
    }
}
