pub use flatbuffers_reflection as reflection;

mod invoke;
pub use invoke::compile_reflection_schema;

mod code_gen;
pub use code_gen::{MessageType, Method, Service, collect_in_out_types, get_services_from_schema};
