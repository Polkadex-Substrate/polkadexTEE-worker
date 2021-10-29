use crate::{
    ed25519::Ed25519, nonce_handler, utils::hash_from_slice, write_slice_and_whitespace_pad,
};
use codec::Encode;
use sgx_types::sgx_status_t;
use sp_application_crypto::Pair;
use std::slice;
use substrate_api_client::compose_extrinsic_offline;
use substratee_settings::node::{
    OCEX_MODULE, OCEX_UPLOAD_CID, RUNTIME_SPEC_VERSION, RUNTIME_TRANSACTION_VERSION,
};
use substratee_sgx_io::SealedIO;

#[no_mangle]
pub unsafe extern "C" fn send_cid(
    genesis_hash: *const u8,
    genesis_hash_size: u32,
    _nonce: u32,
    cid: *const u8,
    cid_size: u32,
    unchecked_extrinsic: *mut u8,
    unchecked_extrinsic_size: u32,
) -> sgx_status_t {
    let genesis_hash_slice = slice::from_raw_parts(genesis_hash, genesis_hash_size as usize);
    let extrinsic_slice =
        slice::from_raw_parts_mut(unchecked_extrinsic, unchecked_extrinsic_size as usize);
    let signer = match Ed25519::unseal() {
        Ok(pair) => pair,
        Err(status) => return status.into(),
    };

    let genesis_hash = hash_from_slice(genesis_hash_slice);

    let data = slice::from_raw_parts(cid, cid_size as usize);

    let xt_block = [OCEX_MODULE, OCEX_UPLOAD_CID];

    let xt = compose_extrinsic_offline!(
        signer,
        (xt_block, data),
        //    nonce,
        {
            if let Ok(mutex) = nonce_handler::load_nonce_storage() {
                if let Ok(locked) = mutex.lock() {
                    locked.nonce
                } else {
                    return sgx_status_t::SGX_ERROR_UNEXPECTED;
                }
            } else {
                return sgx_status_t::SGX_ERROR_UNEXPECTED;
            }
        }, // TODO: Fix nonce being out of sync
        Era::Immortal,
        genesis_hash,
        genesis_hash,
        RUNTIME_SPEC_VERSION,
        RUNTIME_TRANSACTION_VERSION
    );

    write_slice_and_whitespace_pad(extrinsic_slice, xt.encode());

    sgx_status_t::SGX_SUCCESS
}
