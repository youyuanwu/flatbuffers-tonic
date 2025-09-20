#[test]
fn test_reflect_hello() {
    let schema = flatbuffers_util::reflect::compile_reflection_schema(std::path::Path::new(
        "../fbs/fbs.helloworld.fbs",
    ));
    let schema = schema.get_ref();
    // println!("Schema: {:?}", schema.services());
    schema.services().unwrap().iter().for_each(|service| {
        println!("Service: {}", service.name());
        service.calls().unwrap().iter().for_each(|call| {
            println!("  Call: {}", call.name());
            println!("    Request Type: {:?}", call.request().name());
            println!("    Response Type: {:?}", call.response().name());
        });
    });

    let ctx = flatbuffers_util::reflect::GeneratorContext::parse_from_schema(&schema);
    let services = ctx.get_services();
    assert_eq!(services.len(), 1);
    let service = &services[0];
    assert_eq!(service.name, "Greeter");
    assert_eq!(service.methods.len(), 1);
    let method = &service.methods[0];
    assert_eq!(method.name, "SayHello");
    assert_eq!(method.request_name, "fbs.helloworld.HelloRequest");
    assert_eq!(method.response_name, "fbs.helloworld.HelloReply");
    assert!(!method.server_streaming);
    assert!(!method.client_streaming);
    assert_eq!(service.svc_type(), "Greeter");
    assert_eq!(method.request_type(), "HelloRequest");
    assert_eq!(method.response_type(), "HelloReply");

    let types = flatbuffers_util::reflect::collect_in_out_types(services);
    assert_eq!(types.len(), 2);
    assert_eq!(types[0].fb_type, "HelloRequest");
    assert_eq!(types[1].fb_type, "HelloReply");
}

#[test]
fn test_reflect_sample() {
    let schema = flatbuffers_util::reflect::compile_reflection_schema(std::path::Path::new(
        "../fbs/sample.fbs",
    ));
    let schema = schema.get_ref();
    // println!("Schema: {:?}", schema.services());
    schema.services().unwrap().iter().for_each(|service| {
        println!("Service: {}", service.name());
        service.calls().unwrap().iter().for_each(|call| {
            println!("  Call: {}", call.name());
            println!("    Request Type: {:?}", call.request().name());
            println!("    Response Type: {:?}", call.response().name());
        });
    });

    let ctx = flatbuffers_util::reflect::GeneratorContext::parse_from_schema(&schema);
    let services = ctx.get_services();
    assert_eq!(services.len(), 2);
    let service = &services[1];
    assert_eq!(service.name, "Sample");
    assert_eq!(service.methods.len(), 3);
    for method in &service.methods {
        println!("Method: {method:?}");
        if method.name.as_str() == "client_stream" {
            assert_eq!(method.request_name, "sample.client_stream_request");
            assert_eq!(method.response_name, "sample.client_stream_response");
            assert!(!method.server_streaming);
            assert!(method.client_streaming);
        }
    }
}
