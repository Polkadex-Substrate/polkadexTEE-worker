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

use clap::ArgMatches;
use sp_application_crypto::sr25519;
use sp_core::{sr25519 as sr25519_core, Pair};

use crate::cli_utils::account_parsing::get_pair_from_str;
use crate::commands::common_args::{ACCOUNT_ID_ARG_NAME, PROXY_ACCOUNT_ID_ARG_NAME};

pub struct AccountDetails {
    main_account: sr25519::AppPair,
    proxy_account: Option<sr25519::AppPair>,
}

impl AccountDetails {
    pub fn new(matches: &ArgMatches<'_>) -> Self {
        let arg_account = matches.value_of(ACCOUNT_ID_ARG_NAME).expect(&format!(
            "missing main account option ({})",
            ACCOUNT_ID_ARG_NAME
        ));

        let main_account_pair = get_pair_from_str(matches, arg_account);

        let arg_proxy_account_option = matches.value_of(PROXY_ACCOUNT_ID_ARG_NAME);
        let proxy_account_pair = arg_proxy_account_option.map(|pa| get_pair_from_str(matches, pa));

        AccountDetails {
            main_account: main_account_pair,
            proxy_account: proxy_account_pair,
        }
    }

    pub fn signer_pair(&self) -> sr25519::AppPair {
        match &self.proxy_account {
            Some(ap) => ap.clone(),
            None => self.main_account.clone(),
        }
    }

    pub fn signer_key_pair(&self) -> sr25519_core::Pair {
        sr25519_core::Pair::from(self.signer_pair().clone())
    }

    pub fn main_account_public_key(&self) -> sr25519_core::Public {
        sr25519_core::Public::from(self.main_account.public())
    }

    pub fn proxy_account_public_key(&self) -> Option<sr25519_core::Public> {
        self.proxy_account
            .clone()
            .map(|pa| sr25519_core::Public::from(pa.public()))
    }
}
