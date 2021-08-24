// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
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

use crate::{Getter::*, TrustedCall, TrustedGetter, TrustedOperation};

pub fn get_rpc_function_name_from_top(trusted_operation: &TrustedOperation) -> Option<String> {
	match trusted_operation {
		TrustedOperation::get(getter) => match getter {
			public(_) => None,
			trusted(tgs) => match tgs.getter {
				TrustedGetter::get_balance(_, _, _) => Some("get_balance".to_owned()),
				TrustedGetter::nonce(_) => Some("nonce".to_owned()),
				_ => None,
			},
		},
		TrustedOperation::indirect_call(_) => None,
		TrustedOperation::direct_call(trusted_call_signed) => match trusted_call_signed.call {
			TrustedCall::place_order(_, _, _) => Some("place_order".to_owned()),
			TrustedCall::cancel_order(_, _, _) => Some("cancel_order".to_owned()),
			TrustedCall::withdraw(_, _, _, _) => Some("withdraw".to_owned()),
			_ => None,
		},
	}
}
