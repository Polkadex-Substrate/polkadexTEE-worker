pub extern crate alloc;
use alloc::{borrow::ToOwned, str, string::String, vec::Vec};

use jsonrpc_core::IoHandler;

pub fn get_all_rpc_methods_string(io_handler: &IoHandler) -> String {
    let method_string = io_handler
        .iter()
        .map(|rp_tuple| rp_tuple.0.to_owned())
        .collect::<Vec<String>>()
        .join(", ");

    format!("methods: [{}]", method_string)
}

pub mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::alloc::string::ToString;
    use super::*;
    use jsonrpc_core::Params;
    use serde_json::Value;

    pub fn test_given_io_handler_methods_then_retrieve_all_names_as_string() {
        let mut io = IoHandler::new();
        let method_names: [&str; 4] = ["method1", "another_method", "fancy_thing", "solve_all"];

        for method_name in method_names.iter() {
            io.add_sync_method(method_name, |_: Params| Ok(Value::String("".to_string())));
        }

        let method_string = get_all_rpc_methods_string(&io);

        for method_name in method_names.iter() {
            assert!(method_string.contains(method_name));
        }
    }
}
