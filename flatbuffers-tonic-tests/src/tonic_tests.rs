use flatbuffers_util::ownedfb::OwnedFB;
use tokio_util::sync::CancellationToken;

use crate::helloworld_gen::{self, OwnedHelloReply, OwnedHelloRequest};

pub struct Greeter {}

#[tonic::async_trait]
impl helloworld_gen::greeter_server::Greeter for Greeter {
    async fn say_hello(
        &self,
        request: tonic::Request<OwnedHelloRequest>,
    ) -> Result<tonic::Response<OwnedHelloReply>, tonic::Status> {
        let request = request.into_inner();
        let name = request.get_ref().name();
        println!("Got a name: {name:?}");
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let hello_str = builder.create_string(&format!("hello {}", name.unwrap_or("")));
        let reply = helloworld_gen::fbs::helloworld::HelloReply::create(
            &mut builder,
            &helloworld_gen::fbs::helloworld::HelloReplyArgs {
                message: Some(hello_str),
            },
        );
        builder.finish_minimal(reply);
        let resp = unsafe { OwnedFB::new_from_builder_collapse(builder.collapse()) };
        Ok(tonic::Response::new(OwnedHelloReply(resp)))
    }
}

// creates a listener on a random port from os, and return the addr.
pub async fn create_listener_server() -> (tokio::net::TcpListener, std::net::SocketAddr) {
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    (listener, local_addr)
}

#[tokio::test]
async fn test_server_client() {
    let (listener, addr) = create_listener_server().await;
    let token = CancellationToken::new();

    // run server in task
    let svh = {
        let token = token.clone();
        tokio::spawn(async move {
            let svc = helloworld_gen::greeter_server::GreeterServer::new(Greeter {});
            tonic::transport::Server::builder()
                .add_service(svc)
                .serve_with_incoming_shutdown(
                    tonic::transport::server::TcpIncoming::from(listener),
                    token.cancelled(),
                )
                .await
                .unwrap();
        })
    };

    // run client to send a msg
    let mut client =
        helloworld_gen::greeter_client::GreeterClient::connect(format!("http://{}", addr))
            .await
            .unwrap();

    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let name_str = builder.create_string("tonic fbs");
    let req = helloworld_gen::fbs::helloworld::HelloRequest::create(
        &mut builder,
        &helloworld_gen::fbs::helloworld::HelloRequestArgs {
            name: Some(name_str),
        },
    );
    builder.finish_minimal(req);
    let owned = unsafe {
        flatbuffers_util::ownedfb::OwnedFB::<helloworld_gen::fbs::helloworld::HelloRequest>::new_from_builder_collapse(builder.collapse())
    };
    let response = client
        .say_hello(tonic::Request::new(OwnedHelloRequest(owned)))
        .await
        .unwrap();
    let reply = response.into_inner();
    let reply_ref = reply.get_ref();
    assert_eq!(reply_ref.message(), Some("hello tonic fbs"));

    token.cancel();
    svh.await.unwrap();
}
