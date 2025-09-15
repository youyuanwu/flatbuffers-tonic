use flatbuffers_reflection::reflection;

fn get_services_from_schema(schema: &reflection::Schema) -> Vec<Service> {
    schema
        .services()
        .unwrap()
        .iter()
        .map(|service| Service::new_from_schema(&service))
        .collect()
}

pub struct GeneratorContext {
    pub services: Vec<Service>,
}

impl GeneratorContext {
    pub fn parse_from_schema(schema: &reflection::Schema) -> Self {
        let services = get_services_from_schema(schema);
        GeneratorContext { services }
    }

    pub fn get_services(&self) -> &[Service] {
        &self.services
    }

    pub fn collect_in_out_types(&self) -> Vec<MessageType> {
        collect_in_out_types(&self.services)
    }

    /// get the namespace in raw form
    /// useful for creating files.
    pub fn get_namespace(&self) -> String {
        assert_ne!(self.services.len(), 0, "no services found");
        self.services[0].namespace.as_ref().unwrap().clone()
    }

    /// replace dot with double colon
    /// and to lowercase.
    /// This is the rust mod path to be used for accessing
    /// the flatbuffers generated code from the wrapper types.
    pub fn get_namespace_rs(&self) -> String {
        self.get_namespace().replace('.', "::").to_lowercase()
    }
}

#[derive(Debug, Clone)]
pub struct MessageType {
    /// type without namespace
    pub fb_type: String,
    pub namespace: Option<String>,
}

/// Return all unique in/out types from services
pub fn collect_in_out_types(services: &[Service]) -> Vec<MessageType> {
    let mut types = Vec::new();
    for svc in services {
        for method in &svc.methods {
            let req_type = method.request_type();
            let resp_type = method.response_type();
            if !types
                .iter()
                .map(|t: &MessageType| &t.fb_type)
                .any(|x| x == &req_type)
            {
                types.push(MessageType {
                    fb_type: req_type,
                    namespace: svc.namespace.clone(),
                });
            }
            if !types
                .iter()
                .map(|t: &MessageType| &t.fb_type)
                .any(|x| x == &resp_type)
            {
                types.push(MessageType {
                    fb_type: resp_type,
                    namespace: svc.namespace.clone(),
                });
            }
        }
    }
    types
}

/// rpc Service
#[derive(Debug)]
pub struct Service {
    pub namespace: Option<String>,
    pub name: String,
    pub methods: Vec<Method>,
}
/// rpc Method
#[derive(Debug)]
pub struct Method {
    pub name: String,
    /// Unparsed request type name
    pub request_name: String,
    /// Unparsed response type name
    pub response_name: String,
    pub server_streaming: bool,
    pub client_streaming: bool,
}

impl Service {
    pub fn new_from_schema(schema: &reflection::Service) -> Self {
        let name = schema.name().to_string();
        // split by last dot to get namespace and name
        // if no dot, then no namespace
        let (namespace, name) = if let Some(pos) = name.rfind('.') {
            let (ns, n) = name.split_at(pos);
            (Some(ns.to_string()), n[1..].to_string())
        } else {
            (None, name)
        };
        let methods = schema
            .calls()
            .unwrap()
            .iter()
            .map(|call| Method::new_from_schema(&call))
            .collect();
        Service {
            namespace,
            name,
            methods,
        }
    }

    pub fn svc_type(&self) -> String {
        // find the last part after the last dot
        let mut parts = self.name.rsplitn(2, '.');
        parts.next().unwrap().to_string()
    }
}

impl Method {
    pub fn new_from_schema(call: &reflection::RPCCall) -> Self {
        let name = call.name().to_string();
        let request_type = call.request().name().to_string();
        let response_type = call.response().name().to_string();
        let server_streaming = call
            .attributes()
            .unwrap()
            .iter()
            .any(|kv| kv.key() == "streaming" && kv.value() == Some("server"));
        let client_streaming = call
            .attributes()
            .unwrap()
            .iter()
            .any(|kv| kv.key() == "streaming" && kv.value() == Some("client"));
        Method {
            name,
            request_name: request_type,
            response_name: response_type,
            server_streaming,
            client_streaming,
        }
    }

    pub fn is_unary(&self) -> bool {
        !self.server_streaming && !self.client_streaming
    }

    /// Just return the plain type without namespace
    pub fn request_type(&self) -> String {
        // find the last part after the last dot
        self.request_name.rsplit('.').next().unwrap().to_string()
    }

    /// Just return the plain type without namespace
    pub fn response_type(&self) -> String {
        // find the last part after the last dot
        self.response_name.rsplit('.').next().unwrap().to_string()
    }
}
