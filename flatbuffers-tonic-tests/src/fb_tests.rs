use flatbuffers::FlatBufferBuilder;
use flatbuffers_util::ownedfb::OwnedFB;

use crate::helloworld_gen::fbs::helloworld::{HelloRequest, HelloRequestArgs};

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
        let owned = OwnedFB::<HelloRequest>::new(&boxed_buff).unwrap();
        let req_x = owned.get_ref();
        assert_eq!(req_x.name(), Some("hello world"));
    }
    {
        let owned = OwnedFB::<HelloRequest>::new_from_vec(boxed_buff.to_vec(), 0).unwrap();
        let req_x = owned.get_ref();
        assert_eq!(req_x.name(), Some("hello world"));
    }
    // Move the data
    {
        let mut builder = FlatBufferBuilder::new();

        let bar_str = builder.create_string("hello world2");

        let req = HelloRequest::create(
            &mut builder,
            &HelloRequestArgs {
                name: Some(bar_str),
            },
        );
        builder.finish_minimal(req);
        let owned =
            unsafe { OwnedFB::<HelloRequest>::new_from_builder_collapse(builder.collapse()) };
        let req_x = owned.get_ref();
        assert_eq!(req_x.name(), Some("hello world2"));
    }
    // Check the offset of vector.
    {
        let mut builder = FlatBufferBuilder::new();

        let bar_str = builder.create_string("hello world2");

        let req = HelloRequest::create(
            &mut builder,
            &HelloRequestArgs {
                name: Some(bar_str),
            },
        );
        builder.finish_minimal(req);
        let (buf, _index) = builder.collapse();
        // This condition can ensure the encode to bytes is zero copy.
        assert_eq!(buf.capacity(), buf.len());
        let owned = OwnedFB::<HelloRequest>::new_from_bytes(buf.into()).unwrap();
        let bytes = owned.into_bytes();
        bytes
            .try_into_mut()
            .expect("Should have the full ownership of the vec");
    }
}
