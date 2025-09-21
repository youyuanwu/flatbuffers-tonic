use flatbuffers_util::FBBuilder;
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
        let mut builder = FBBuilder::new();
        let hello_str = builder
            .get_mut()
            .create_string(&format!("hello {}", name.unwrap_or("")));
        let reply = generated::fbs::helloworld::HelloReply::create(
            builder.get_mut(),
            &generated::fbs::helloworld::HelloReplyArgs {
                message: Some(hello_str),
            },
        );
        let resp = builder.finish_owned(reply).into();
        Ok(tonic::Response::new(resp))
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

    let mut builder = FBBuilder::new();
    let name_str = builder.get_mut().create_string("tonic fbs");
    let req = generated::fbs::helloworld::HelloRequest::create(
        builder.get_mut(),
        &generated::fbs::helloworld::HelloRequestArgs {
            name: Some(name_str),
        },
    );
    let req = builder.finish_owned(req).into();
    let response = client.say_hello(tonic::Request::new(req)).await.unwrap();
    let reply = response.into_inner();
    let reply_ref = reply.get_ref();
    assert_eq!(reply_ref.message(), Some("hello tonic fbs"));

    token.cancel();
    svh.await.unwrap();
}

mod sample_test {
    use flatbuffers_tonic::FBBuilder;
    use tokio::sync::mpsc;
    use tokio_stream::StreamExt;
    use tokio_util::sync::CancellationToken;
    use tonic::transport::Endpoint;

    pub struct HelloSampleSvc {}

    #[tonic::async_trait]
    impl crate::generated::hello_sample_server::HelloSample for HelloSampleSvc {
        async fn say_hello(
            &self,
            request: tonic::Request<crate::generated::Ownedsample_request>,
        ) -> Result<tonic::Response<crate::generated::Ownedsample_reply>, tonic::Status> {
            let request = request.into_inner();
            let name = request.get_ref().name();
            println!("Got a name: {name:?}");
            let mut builder = FBBuilder::new();
            let hello_str = builder
                .get_mut()
                .create_string(&format!("hello {}", name.unwrap_or("")));
            let reply = crate::generated::sample::sample_reply::create(
                builder.get_mut(),
                &crate::generated::sample::sample_replyArgs {
                    message: Some(hello_str),
                },
            );
            let resp = builder.finish_owned(reply).into();
            Ok(tonic::Response::new(resp))
        }
    }

    pub struct SampleSvc {}

    #[tonic::async_trait]
    impl crate::generated::sample_server::Sample for SampleSvc {
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
            let mut builder = FBBuilder::new();
            let reply = crate::generated::sample::client_stream_response::create(
                builder.get_mut(),
                &crate::generated::sample::client_stream_responseArgs { count },
            );
            let resp = builder.finish_owned(reply).into();
            Ok(tonic::Response::new(resp))
        }

        type server_streamStream = std::pin::Pin<
            Box<
                dyn tokio_stream::Stream<
                        Item = Result<crate::generated::Ownedserver_stream_response, tonic::Status>,
                    > + Send
                    + 'static,
            >,
        >;

        async fn server_stream(
            &self,
            request: tonic::Request<crate::generated::Ownedserver_stream_request>,
        ) -> Result<tonic::Response<Self::server_streamStream>, tonic::Status> {
            let request = request.into_inner();
            let count = request.get_ref().count();
            println!("server_stream request count: {count}");
            let response_stream = tokio_stream::iter(0..count).map(|i| {
                let mut builder = FBBuilder::new();
                let message = builder
                    .get_mut()
                    .create_string(&format!("server response {i}"));
                let resp = crate::generated::sample::server_stream_response::create(
                    builder.get_mut(),
                    &crate::generated::sample::server_stream_responseArgs {
                        message: Some(message),
                    },
                );
                let resp = builder.finish_owned(resp).into();
                Ok(resp)
            });
            Ok(tonic::Response::new(Box::pin(response_stream)))
        }

        type bidi_streamStream = std::pin::Pin<
            Box<
                dyn tokio_stream::Stream<
                        Item = Result<crate::generated::Ownedsample_reply, tonic::Status>,
                    > + Send
                    + 'static,
            >,
        >;

        async fn bidi_stream(
            &self,
            request: tonic::Request<tonic::Streaming<crate::generated::Ownedsample_request>>,
        ) -> Result<tonic::Response<Self::bidi_streamStream>, tonic::Status> {
            let mut in_stream = request.into_inner();
            let (tx, rx) = mpsc::channel(128);
            tokio::spawn(async move {
                while let Some(req) = in_stream.message().await.unwrap() {
                    let req = req.get_ref();
                    let name = req.name();
                    println!("Got a name in bidi stream: {name:?}");
                    let mut builder = FBBuilder::new();
                    let hello_str = builder
                        .get_mut()
                        .create_string(&format!("hello {}", name.unwrap_or("")));
                    let reply = crate::generated::sample::sample_reply::create(
                        builder.get_mut(),
                        &crate::generated::sample::sample_replyArgs {
                            message: Some(hello_str),
                        },
                    );
                    let resp = builder.finish_owned(reply).into();
                    match tx.send(Ok(resp)).await {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Failed to send response");
                            break;
                        }
                    };
                }
            });
            let out_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            Ok(tonic::Response::new(Box::pin(out_stream)))
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
                tonic::transport::Server::builder()
                    .add_service(crate::generated::sample_server::SampleServer::new(
                        SampleSvc {},
                    ))
                    .add_service(
                        crate::generated::hello_sample_server::HelloSampleServer::new(
                            HelloSampleSvc {},
                        ),
                    )
                    .serve_with_incoming_shutdown(
                        tonic::transport::server::TcpIncoming::from(listener),
                        token.cancelled(),
                    )
                    .await
                    .unwrap();
            })
        };

        let ch = Endpoint::from_shared(format!("http://{}", addr))
            .unwrap()
            .connect()
            .await
            .unwrap();

        // run client to send a msg
        let mut hello_client =
            crate::generated::hello_sample_client::HelloSampleClient::new(ch.clone());
        let mut client = crate::generated::sample_client::SampleClient::new(ch);
        let mut builder = FBBuilder::new();
        let name_str = builder.get_mut().create_string("tonic fbs");
        let req = crate::generated::sample::sample_request::create(
            builder.get_mut(),
            &crate::generated::sample::sample_requestArgs {
                name: Some(name_str),
            },
        );
        let req = builder.finish_owned(req).into();
        let response = hello_client
            .say_hello(tonic::Request::new(req))
            .await
            .unwrap();
        let reply = response.into_inner();
        let reply_ref = reply.get_ref();
        assert_eq!(reply_ref.message(), Some("hello tonic fbs"));

        // test client stream
        // create 10 messages
        let request_stream = tokio_stream::iter(0..10).map(|i| {
            let mut builder = FBBuilder::new();
            let req = crate::generated::sample::client_stream_request::create(
                builder.get_mut(),
                &crate::generated::sample::client_stream_requestArgs { index: i },
            );
            builder.finish_owned(req).into()
        });
        let response = client
            .client_stream(tonic::Request::new(request_stream))
            .await
            .unwrap();
        let reply = response.into_inner();
        let reply_ref = reply.get_ref();
        assert_eq!(reply_ref.count(), 10);

        // test server stream
        let mut builder = FBBuilder::new();
        let req = crate::generated::sample::server_stream_request::create(
            builder.get_mut(),
            &crate::generated::sample::server_stream_requestArgs { count: 5 },
        );
        let req = builder.finish_owned(req).into();
        let mut response = client
            .server_stream(tonic::Request::new(req))
            .await
            .unwrap()
            .into_inner();
        let mut idx = 0;
        while let Some(msg) = response.next().await {
            let msg = msg.unwrap();
            let msg_ref = msg.get_ref();
            let expected = format!("server response {idx}");
            assert_eq!(msg_ref.message(), Some(expected.as_str()));
            idx += 1;
        }
        assert_eq!(idx, 5);

        // test bidi stream
        let request_stream = tokio_stream::iter(0..5).map(|i| {
            let mut builder = FBBuilder::new();
            let name_str = builder.get_mut().create_string(&format!("name {i}"));
            let req = crate::generated::sample::sample_request::create(
                builder.get_mut(),
                &crate::generated::sample::sample_requestArgs {
                    name: Some(name_str),
                },
            );
            builder.finish_owned(req).into()
        });
        let mut response = client
            .bidi_stream(tonic::Request::new(request_stream))
            .await
            .unwrap()
            .into_inner();
        let mut idx = 0;
        while let Some(msg) = response.next().await {
            let msg = msg.unwrap();
            let msg_ref = msg.get_ref();
            let expected = format!("hello name {idx}");
            assert_eq!(msg_ref.message(), Some(expected.as_str()));
            idx += 1;
        }
        assert_eq!(idx, 5);

        token.cancel();
        svh.await.unwrap();
    }
}
