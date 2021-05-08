#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

mod tests;
mod tlsclient;
mod types;

use sgx_types::sgx_status_t;

// Create a WS Client to OpenFinex
pub fn subscribe_to_openfinex_api(address: &str) -> sgx_status_t {
    sgx_status_t::SGX_SUCCESS
}
