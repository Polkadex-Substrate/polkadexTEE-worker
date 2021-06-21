// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub extern crate alloc;
use crate::openfinex::openfinex_types::{Preamble, RequestId, RequestType};
use alloc::{string::String, string::ToString, vec::Vec};

#[derive(Debug, Clone)]
pub struct OpenFinexRequest {
    pub request_type: RequestType,
    pub request_preamble: Preamble,
    pub request_id: RequestId,
    pub parameters: Vec<String>,
}

impl OpenFinexRequest {
    pub fn to_request_string(&self) -> String {
        let mut request_string = String::new();
        request_string.push_str(
            format!(
                "[{},{},\"{}\",[",
                self.request_preamble,
                self.request_id,
                self.request_type.to_request_string()
            )
            .as_str(),
        );

        let parameters_string = self.parameters.join(",");
        request_string.push_str(format!("{}]]", parameters_string).as_str());

        request_string
    }
}

const REQUEST_PREAMBLE: Preamble = 1;

/// Request builder
pub struct OpenFinexRequestBuilder {
    request_type: RequestType,
    request_id: RequestId,
    parameters: Vec<String>,
}

impl OpenFinexRequestBuilder {
    pub fn new(request_type: RequestType, request_id: RequestId) -> Self {
        OpenFinexRequestBuilder {
            request_type,
            request_id,
            parameters: Vec::new(),
        }
    }

    pub fn push_parameter(&mut self, param: String) -> &mut Self {
        (self).parameters.push(format!("\"{}\"", param)); // add enclosing double quotes "param"
        self
    }

    pub fn push_optional_parameter(&mut self, param: Option<String>) -> &mut Self {
        match param {
            Some(p) => self.push_parameter(p),
            None => self.push_parameter(OpenFinexRequestBuilder::empty_parameter()),
        }
    }

    pub fn push_list_parameter(&mut self, params: Vec<String>) -> &mut Self {
        (self)
            .parameters
            .push(format!("[\"{}\"]", params.join("\",\"")));
        self
    }

    pub fn build(&self) -> OpenFinexRequest {
        OpenFinexRequest {
            request_type: self.request_type.clone(),
            request_preamble: REQUEST_PREAMBLE,
            request_id: self.request_id,
            parameters: self.parameters.clone(),
        }
    }

    fn empty_parameter() -> String {
        "".to_string()
    }
}
