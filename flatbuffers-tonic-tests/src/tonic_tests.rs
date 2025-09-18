use flatbuffers_util::ownedfb::OwnedFB;
use tokio_util::sync::CancellationToken;

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
            let svc = generated::greeter_server::GreeterServer::new(Greeter {});
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
        flatbuffers_util::ownedfb::OwnedFB::<generated::fbs::helloworld::HelloRequest>::new_from_builder_collapse(builder.collapse())
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

mod sample_test {
    use flatbuffers_tonic::OwnedFB;
    use tokio_stream::StreamExt;
    use tokio_util::sync::CancellationToken;

    pub struct SampleSvc {}

    #[tonic::async_trait]
    impl crate::generated::sample_server::Sample for SampleSvc {
        async fn say_hello(
            &self,
            request: tonic::Request<crate::generated::Ownedsample_request>,
        ) -> Result<tonic::Response<crate::generated::Ownedsample_reply>, tonic::Status> {
            let request = request.into_inner();
            let name = request.get_ref().name();
            println!("Got a name: {name:?}");
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let hello_str = builder.create_string(&format!("hello {}", name.unwrap_or("")));
            let reply = crate::generated::sample::sample_reply::create(
                &mut builder,
                &crate::generated::sample::sample_replyArgs {
                    message: Some(hello_str),
                },
            );
            builder.finish_minimal(reply);
            let resp = unsafe { OwnedFB::new_from_builder_collapse(builder.collapse()) };
            Ok(tonic::Response::new(crate::generated::Ownedsample_reply(
                resp,
            )))
        }

        async fn client_stream(
            &self,
            request: tonic::Request<tonic::Streaming<crate::generated::Ownedclient_stream_request>>,
        ) -> Result<tonic::Response<crate::generated::Ownedclient_stream_response>, tonic::Status>
        {
            // read all the stream chunks and count
            let mut stream = request.into_inner();
            let mut count = 0;
            while let Some(req) = stream.message().await? {
                let req = req.get_ref();
                let index = req.index();
                println!("Got stream chunk with index: {index}");
                count += 1;
            }
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let reply = crate::generated::sample::client_stream_response::create(
                &mut builder,
                &crate::generated::sample::client_stream_responseArgs { count },
            );
            builder.finish_minimal(reply);
            let resp = unsafe { OwnedFB::new_from_builder_collapse(builder.collapse()) };
            Ok(tonic::Response::new(
                crate::generated::Ownedclient_stream_response(resp),
            ))
        }
    }

    #[tokio::test]
    async fn test_sample_server_client() {
        let (listener, addr) = crate::tonic_tests::create_listener_server().await;
        let token = CancellationToken::new();
        // run server in task
        let svh = {
            let token = token.clone();
            tokio::spawn(async move {
                let svc = crate::generated::sample_server::SampleServer::new(SampleSvc {});
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
            crate::generated::sample_client::SampleClient::connect(format!("http://{}", addr))
                .await
                .unwrap();
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let name_str = builder.create_string("tonic fbs");
        let req = crate::generated::sample::sample_request::create(
            &mut builder,
            &crate::generated::sample::sample_requestArgs {
                name: Some(name_str),
            },
        );
        builder.finish_minimal(req);
        let owned = unsafe {
            flatbuffers_util::ownedfb::OwnedFB::<crate::generated::sample::sample_request>::new_from_builder_collapse(builder.collapse())
        };
        let response = client
            .say_hello(tonic::Request::new(crate::generated::Ownedsample_request(
                owned,
            )))
            .await
            .unwrap();
        let reply = response.into_inner();
        let reply_ref = reply.get_ref();
        assert_eq!(reply_ref.message(), Some("hello tonic fbs"));

        // test client stream
        // create 10 messages
        let request_stream = tokio_stream::iter(0..10).map(|i| {
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let req = crate::generated::sample::client_stream_request::create(
                &mut builder,
                &crate::generated::sample::client_stream_requestArgs { index: i },
            );
            builder.finish_minimal(req);
            let owned = unsafe {
                flatbuffers_util::ownedfb::OwnedFB::<
                        crate::generated::sample::client_stream_request,
                    >::new_from_builder_collapse(builder.collapse())
            };
            crate::generated::Ownedclient_stream_request(owned)
        });
        let response = client
            .client_stream(tonic::Request::new(request_stream))
            .await
            .unwrap();
        let reply = response.into_inner();
        let reply_ref = reply.get_ref();
        assert_eq!(reply_ref.count(), 10);

        token.cancel();
        svh.await.unwrap();
    }
}
