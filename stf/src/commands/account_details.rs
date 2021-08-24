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

use crate::{
	cli_utils::account_parsing::get_pair_from_str_trusted,
	commands::common_args::{ACCOUNT_ID_ARG_NAME, PROXY_ACCOUNT_ID_ARG_NAME},
};
use clap::ArgMatches;
use sp_application_crypto::sr25519;
use sp_core::{sr25519 as sr25519_core, Pair};

/// Account details parsed from the command line arguments
/// Provides methods to get the signer account, depending on whether an optional
/// proxy account was provided, or just a main account
pub struct AccountDetails {
	main_account: sr25519::AppPair,
	proxy_account: Option<sr25519::AppPair>,
}

impl AccountDetails {
	pub fn new(matches: &ArgMatches<'_>) -> Self {
		let arg_account = matches
			.value_of(ACCOUNT_ID_ARG_NAME)
			.unwrap_or_else(|| panic!("missing main account option ({})", ACCOUNT_ID_ARG_NAME));

		let main_account_pair = get_pair_from_str_trusted(matches, arg_account);

		let arg_proxy_account_option = matches.value_of(PROXY_ACCOUNT_ID_ARG_NAME);
		let proxy_account_pair =
			arg_proxy_account_option.map(|pa| get_pair_from_str_trusted(matches, pa));

		AccountDetails { main_account: main_account_pair, proxy_account: proxy_account_pair }
	}

	pub fn signer_pair(&self) -> sr25519::AppPair {
		match &self.proxy_account {
			Some(ap) => ap.clone(),
			None => self.main_account.clone(),
		}
	}

	pub fn signer_key_pair(&self) -> sr25519_core::Pair {
		sr25519_core::Pair::from(self.signer_pair())
	}

	pub fn signer_public_key(&self) -> sr25519_core::Public {
		sr25519_core::Public::from(self.signer_pair().public())
	}

	pub fn main_account_public_key(&self) -> sr25519_core::Public {
		sr25519_core::Public::from(self.main_account.public())
	}

	/// returns a main account public key, IF the signer is a proxy, none otherwise
	pub fn main_account_public_key_if_not_signer(&self) -> Option<sr25519_core::Public> {
		self.proxy_account
			.as_ref()
			.map(|_| sr25519_core::Public::from(self.main_account.public()))
	}

	#[cfg(test)]
	pub fn proxy_account_public_key(&self) -> Option<sr25519_core::Public> {
		self.proxy_account.clone().map(|pa| sr25519_core::Public::from(pa.public()))
	}
}

#[cfg(test)]
mod tests {
	use crate::commands::{
		account_details::AccountDetails,
		common_args::{
			add_main_account_args, add_proxy_account_args, ACCOUNT_ID_ARG_NAME,
			PROXY_ACCOUNT_ID_ARG_NAME,
		},
		test_utils::utils::{add_identifiers_app_args, create_identifier_args},
	};
	use clap::{App, AppSettings};

	#[test]
	fn given_proxy_account_argument_then_account_details_has_some() {
		let main_account_arg = format!("--{}=//main_ojwf8a", ACCOUNT_ID_ARG_NAME);
		let proxy_account_arg = format!("--{}=//proxy_awf43t", PROXY_ACCOUNT_ID_ARG_NAME);
		let mut matches_args = vec![main_account_arg, proxy_account_arg];
		matches_args.append(&mut create_identifier_args());

		let test_app = create_test_app();

		let matches = test_app.get_matches_from(matches_args);

		let account_details = AccountDetails::new(&matches);

		assert!(account_details.proxy_account.is_some());

		assert_eq!(
			account_details.proxy_account_public_key().unwrap(),
			account_details.signer_public_key()
		);

		assert_ne!(account_details.main_account_public_key(), account_details.signer_public_key());

		assert!(account_details.main_account_public_key_if_not_signer().is_some());
	}

	#[test]
	fn given_no_proxy_account_argument_then_account_details_has_none() {
		let main_account_arg = format!("--{}=//main_ojwf8a", ACCOUNT_ID_ARG_NAME);
		let mut matches_args = vec![main_account_arg];
		matches_args.append(&mut create_identifier_args());

		let test_app = create_test_app();

		let matches = test_app.get_matches_from(matches_args);

		let account_details = AccountDetails::new(&matches);

		assert!(account_details.proxy_account.is_none());

		assert_eq!(account_details.main_account_public_key(), account_details.signer_public_key());

		assert!(account_details.main_account_public_key_if_not_signer().is_none());
	}

	fn create_test_app<'a, 'b>() -> App<'a, 'b> {
		let test_app = App::new("test_account_details").setting(AppSettings::NoBinaryName);

		let app_with_main_account = add_main_account_args(test_app);
		let app_with_proxy_account = add_proxy_account_args(app_with_main_account);
		let app_with_identifiers = add_identifiers_app_args(app_with_proxy_account);

		app_with_identifiers
	}
}
