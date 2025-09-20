# flatbuffers-tonic
[![build](https://github.com/youyuanwu/flatbuffers-tonic/actions/workflows/CI.yml/badge.svg)](https://github.com/youyuanwu/flatbuffers-tonic/actions/workflows/CI.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://raw.githubusercontent.com/youyuanwu/flatbuffers-tonic/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/flatbuffers-tonic)](https://crates.io/crates/flatbuffers-tonic)
[![Documentation](https://docs.rs/flatbuffers-tonic/badge.svg)](https://docs.rs/flatbuffers-tonic)

Experimental implementation of using [flatbuffers](https://github.com/google/flatbuffers/tree/master/rust/flatbuffers) with [tonic](https://github.com/hyperium/tonic) gRPC for Rust.

## Get started
The usage is almost the same as tonic with prost.

Add deps to your cargo.toml
```toml
[dependencies]
flatbuffers-tonic = { version = "*" }
flatbuffers = { version = "*" }

[build-dependencies]
flatbuffers-tonic-build = { version = "*" }
```
Add to your build.rs
```rs
fn main() {
    flatbuffers_tonic_build::compile_flatbuffers_tonic(&[
        "../fbs/fbs.helloworld.fbs",
        "../fbs/sample.fbs",
    ])
    .expect("flatbuffers tonic compilation failed");
}
```
Include the generated code the same way as using tonic directly:
```rs
// flatbuffers code has warnings.
#![allow(warnings)]
tonic::include_proto!("flatbuffers_tonic.fbs.helloworld");
tonic::include_proto!("flatbuffers_tonic.sample");
```
Write tonic server:
```rs
use crate::generated::{self, OwnedHelloReply, OwnedHelloRequest};

pub struct Greeter {}

#[tonic::async_trait]
impl generated::greeter_server::Greeter for Greeter {
    async fn say_hello(
        &self,
        request: tonic::Request<OwnedHelloRequest>,
    ) -> Result<tonic::Response<OwnedHelloReply>, tonic::Status> {
        let request = request.into_inner();
        let name = request.get_ref().name();
        println!("Got a name: {name:?}");
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let hello_str = builder.create_string(&format!("hello {}", name.unwrap_or("")));
        let reply = generated::fbs::helloworld::HelloReply::create(
            &mut builder,
            &generated::fbs::helloworld::HelloReplyArgs {
                message: Some(hello_str),
            },
        );
        builder.finish_minimal(reply);
        let resp =
            unsafe { flatbuffers_tonic::OwnedFB::new_from_builder_collapse(builder.collapse()) };
        Ok(tonic::Response::new(OwnedHelloReply(resp)))
    }
}

async fn run_server(listener: tokio::net::TcpListener) {
  let svc = generated::greeter_server::GreeterServer::new(Greeter {});
  tonic::transport::Server::builder()
      .add_service(svc)
      .serve_with_incoming_shutdown(
          tonic::transport::server::TcpIncoming::from(listener),
          token.cancelled(),
      )
      .await
      .unwrap();
}
```
Use tonic client:
```rs
    // run client to send a msg
    let mut client = generated::greeter_client::GreeterClient::connect(format!("http://{}", addr))
        .await
        .unwrap();

    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let name_str = builder.create_string("tonic fbs");
    let req = generated::fbs::helloworld::HelloRequest::create(
        &mut builder,
        &generated::fbs::helloworld::HelloRequestArgs {
            name: Some(name_str),
        },
    );
    builder.finish_minimal(req);
    let owned = unsafe {
        flatbuffers_tonic::OwnedFB::<generated::fbs::helloworld::HelloRequest>::new_from_builder_collapse(builder.collapse())
    };
    let response = client
        .say_hello(tonic::Request::new(OwnedHelloRequest(owned)))
        .await
        .unwrap();
    let reply = response.into_inner();
    let reply_ref = reply.get_ref();
    assert_eq!(reply_ref.message(), Some("hello tonic fbs"));
```

## License

MIT license. See [LICENSE](LICENSE).