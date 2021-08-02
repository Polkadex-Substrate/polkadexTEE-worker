/*
    Copyright 2019 Supercomputing Systems AG

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

*/

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
