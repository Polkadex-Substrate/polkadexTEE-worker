use chain_relay::{Header, storage_proof::StorageProofChecker};
use frame_support::PalletId;
use polkadex_primitives::PolkadexAccounts;
use sp_runtime::traits::Header as HeaderT;
use std::collections::HashMap;
// TODO: Fix this import
use std::sync::{Arc, SgxMutex, atomic::{AtomicPtr, Ordering} };

static GLOBAL_ACCOUNTS_STORAGE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub fn verify_pdex_account_read_proofs(
    header: Header,
    accounts: Vec<PolkadexAccounts>) -> SgxResult<(), sgx_status_t> {
    let last_account: AccountID = PalletId(*b"polka/ga").into_account();
    for account in accounts.iter() {
        if account.account.prev == last_account {
            StorageProofChecker::<<Header as HeaderT>::Hashing>::check_proof(
                header.state_root,
                account.account.current, // QUESTION: How is this key defined? What about storage prefix?
                account.proof.to_vec(),
            )
                .sgx_error_with_log("Erroneous StorageProof")?;

            last_account = account.account;
        }
    };

    Ok(())
}

pub fn create_in_memory_account_storage(accounts: Vec<PolkadexAccounts>) -> SgxResult<(), sgx_status_t> {
    let accounts_storage = PolkadexAccountsStorage::create(accounts);
    let storage_ptr = Arc::new(SgxMutex::<PolkadexAccountsStorage>::new(accounts_storage));
    let ptr = Arc::into_raw(storage_ptr);
    GLOBAL_ACCOUNTS_STORAGE.store(ptr as *mut (), Ordering::SeqCst);
    Ok(())
}




pub struct PolkadexAccountsStorage {
    accounts: HashMap<AccountId, Vec<AccountId>>
}

impl PolkadexAccountsStorage {
    pub fn create(&self, accounts: Vec<PolkadexAccounts>) -> PolkadexAccountsStorage {
        let mut in_memory_map: PolkadexAccountsStorage = PolkadexAccountsStorage {
            accounts: HashMap::new(),
        };
        for account in accounts {
            in_memory_map.accounts.insert(account.account.current, account.account.proxies)
        }
        in_memory_map
    }

    pub fn check_main_account(acc: AccountId) -> bool {
        // TODO
        true
    }

    pub fn check_proxy_account(main_acc: AccountId, proxy: AccountId) -> bool {
        // TODO
        true
    }
}

