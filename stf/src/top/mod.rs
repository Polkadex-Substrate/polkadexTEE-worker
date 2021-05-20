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

use crate::{TrustedCall, TrustedOperation};

pub fn get_rpc_function_name_from_top(trusted_operation: &TrustedOperation) -> Option<String> {
    match trusted_operation {
        TrustedOperation::get(_) => None,
        TrustedOperation::indirect_call(_) => None,
        TrustedOperation::direct_call(trusted_call_signed) => match trusted_call_signed.call {
            TrustedCall::place_order(_, _, _) => Some("place_order".to_owned()),
            TrustedCall::cancel_order(_, _, _) => Some("cancel_order".to_owned()),
            TrustedCall::withdraw(_, _, _, _) => Some("withdraw".to_owned()),
            TrustedCall::get_balance(_, _, _) => Some("get_balance".to_owned()),
            _ => None,
        },
    }
}
