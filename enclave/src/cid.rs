use crate::{
    ed25519, hash_from_slice, nonce_handler, write_slice_and_whitespace_pad, NonceHandler,
    OCEX_MODULE, OCEX_UPLOAD_CID, RUNTIME_SPEC_VERSION, RUNTIME_TRANSACTION_VERSION,
};
use codec::Encode;
use sgx_tstd::sync::SgxMutexGuard;
use sgx_types::sgx_status_t;
use sp_application_crypto::Pair;
use std::slice;
use substrate_api_client::compose_extrinsic_offline;

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
    let signer = match ed25519::unseal_pair() {
        Ok(pair) => pair,
        Err(status) => return status,
    };

    let mutex = nonce_handler::load_nonce_storage().unwrap();
    let nonce_storage: SgxMutexGuard<NonceHandler> = mutex.lock().unwrap();
    let enclave_nonce = nonce_storage.nonce;

    let genesis_hash = hash_from_slice(genesis_hash_slice);

    let data = slice::from_raw_parts(cid, cid_size as usize);

    let xt_block = [OCEX_MODULE, OCEX_UPLOAD_CID];

    let xt = compose_extrinsic_offline!(
        signer,
        (xt_block, data),
        //    nonce,
        enclave_nonce, // TODO: Fix nonce being out of sync
        Era::Immortal,
        genesis_hash,
        genesis_hash,
        RUNTIME_SPEC_VERSION,
        RUNTIME_TRANSACTION_VERSION
    );

    let encoded = xt.encode();

    write_slice_and_whitespace_pad(extrinsic_slice, encoded);

    sgx_status_t::SGX_SUCCESS
}
