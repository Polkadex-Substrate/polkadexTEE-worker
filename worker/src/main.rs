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
	direct_invocation::{
		watch_list_service::{WatchList, WatchListService},
		watching_client::WsWatchingClient,
		ws_direct_server_runner::{RunWsServer, WsDirectServerRunner},
		ws_handler::WsHandlerFactory,
	},
	globals::{
		tokio_handle::{GetTokioHandle, GlobalTokioHandle},
		worker::{GlobalWorker, Worker},
	},
	node_api_factory::{CreateNodeApi, GlobalUrlNodeApiFactory},
	ocall_bridge::{
		bridge_api::Bridge as OCallBridge, component_factory::OCallBridgeComponentFactory,
	},
	sync_block_gossiper::SyncBlockGossiper,
	utils::{check_files, extract_shard},
	worker::worker_url_into_async_rpc_url,
};
use base58::ToBase58;
use clap::{load_yaml, App};
use codec::{Decode, Encode};
use config::Config;
use enclave::{
	api::enclave_init,
	tls_ra::{enclave_request_key_provisioning, enclave_run_key_provisioning_server},
};
use log::*;
use my_node_runtime::{pallet_teerex::ShardIdentifier, Event, Hash, Header};
use sgx_types::*;
use sp_core::{
	crypto::{AccountId32, Ss58Codec},
	sr25519, Pair,
};
use sp_finality_grandpa::VersionedAuthorityList;
use sp_keyring::AccountKeyring;
use std::{
	fs::{self, File},
	io::{stdin, Write},
	path::Path,
	str,
	sync::{
		mpsc::{channel, Sender},
		Arc,
	},
	thread,
	time::{Duration, SystemTime},
};
use substrate_api_client::{rpc::WsRpcClient, utils::FromHexString, Api, GenericAddress, XtStatus};
use substratee_api_client_extensions::{AccountApi, ChainApi};
use substratee_enclave_api::{
	direct_request::DirectRequest,
	enclave_base::EnclaveBase,
	remote_attestation::{RemoteAttestation, TlsRemoteAttestation},
	side_chain::SideChain,
	teerex_api::TeerexApi,
};
use substratee_node_primitives::SignedBlock;
use substratee_settings::files::{
	ENCRYPTED_STATE_FILE, SHARDS_PATH, SHIELDING_KEY_FILE, SIGNING_KEY_FILE,
};
use substratee_worker_api::direct_client::DirectClient;

use polkadex_sgx_primitives::types::SignedOrder;
use polkadex_sgx_primitives::{OpenFinexUri, PolkadexAccount};

use crate::enclave::openfinex_tcp_client::enclave_run_openfinex_client;
use crate::polkadex_db::{DiskStorageHandler, OrderbookMirror, PolkadexDBError};

use crate::enclave::api::{
    enclave_accept_pdex_accounts, enclave_init_chain_relay, enclave_load_orders_to_memory,
    enclave_sync_chain,
};

mod config;
mod direct_invocation;
mod enclave;
mod error;
mod globals;
mod node_api_factory;
mod ocall_bridge;
mod sync_block_gossiper;
mod tests;
mod utils;
mod worker;
mod polkadex;
mod polkadex_db;
mod db_handler;

/// how many blocks will be synced before storing the chain db to disk
const BLOCK_SYNC_BATCH_SIZE: u32 = 1000;
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
	// Setup logging
	env_logger::init();

	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();

	let mut config = Config::from(&matches);
	let finex_ip = matches.value_of("openfinex-server").unwrap_or("127.0.0.1");
    let finex_port_path = matches
        .value_of("openfinex-port")
        .unwrap_or("8001/api/v2/ws");
    let finex_uri = OpenFinexUri::new(finex_ip, finex_port_path);

	GlobalTokioHandle::initialize();

	// build the entire dependency tree
	let worker = Arc::new(GlobalWorker {});
	let tokio_handle = Arc::new(GlobalTokioHandle {});
	let sync_block_gossiper = Arc::new(SyncBlockGossiper::new(tokio_handle.clone(), worker));
	let node_api_factory = Arc::new(GlobalUrlNodeApiFactory::new(config.node_url()));
	let direct_invocation_watch_list = Arc::new(WatchListService::<WsWatchingClient>::new());
	let enclave = Arc::new(enclave_init().unwrap());


	// initialize o-call bridge with a concrete factory implementation
	OCallBridge::initialize(Arc::new(OCallBridgeComponentFactory::new(
		node_api_factory.clone(),
		sync_block_gossiper,
		direct_invocation_watch_list.clone(),
		enclave.clone(),
	)));

	if let Some(smatches) = matches.subcommand_matches("run") {
		println!("*** Starting substraTEE-worker");
		let shard = extract_shard(&smatches, enclave.as_ref());

		// Todo: Is this deprecated?? It is only used in remote attestation.
		config.set_ext_api_url(
			smatches
				.value_of("w-server")
				.map(ToString::to_string)
				.unwrap_or_else(|| format!("ws://127.0.0.1:{}", config.worker_rpc_port)),
		);

		println!("Worker Config: {:?}", config);
		let skip_ra = smatches.is_present("skip-ra");

		let node_api = node_api_factory.create_api().set_signer(AccountKeyring::Alice.pair());

		GlobalWorker::reset_worker(Worker::new(
			config.clone(),
			node_api.clone(),
			enclave.clone(),
			DirectClient::new(config.worker_url()),
		));

		start_worker(
			config,
			&shard,
			finex_uri,
			enclave,
			skip_ra,
			node_api,
			tokio_handle,
			direct_invocation_watch_list,
		);
	} else if let Some(smatches) = matches.subcommand_matches("request-keys") {
		let shard = extract_shard(&smatches, enclave.as_ref());
		let provider_url = smatches.value_of("provider").expect("provider must be specified");
		request_keys(provider_url, &shard, enclave.as_ref(), smatches.is_present("skip-ra"));
	} else if matches.is_present("shielding-key") {
		info!("*** Get the public key from the TEE\n");
		let pubkey = enclave.get_rsa_shielding_pubkey().unwrap();
		let file = File::create(SHIELDING_KEY_FILE).unwrap();
		match serde_json::to_writer(file, &pubkey) {
			Err(x) => {
				error!("[-] Failed to write '{}'. {}", SHIELDING_KEY_FILE, x);
			},
			_ => {
				println!("[+] File '{}' written successfully", SHIELDING_KEY_FILE);
			},
		}
	} else if matches.is_present("signing-key") {
		info!("*** Get the signing key from the TEE\n");
		let pubkey = enclave.get_ecc_signing_pubkey().unwrap();
		debug!("[+] Signing key raw: {:?}", pubkey);
		match fs::write(SIGNING_KEY_FILE, pubkey) {
			Err(x) => {
				error!("[-] Failed to write '{}'. {}", SIGNING_KEY_FILE, x);
			},
			_ => {
				println!("[+] File '{}' written successfully", SIGNING_KEY_FILE);
			},
		}
	} else if matches.is_present("dump-ra") {
		info!("*** Perform RA and dump cert to disk");
		enclave.dump_ra_to_disk().unwrap();
	} else if matches.is_present("mrenclave") {
		println!("{}", enclave.get_mrenclave().unwrap().encode().to_base58());
	} else if let Some(_matches) = matches.subcommand_matches("init-shard") {
		let shard = extract_shard(&_matches, enclave.as_ref());
		init_shard(&shard);
	} else if let Some(_matches) = matches.subcommand_matches("test") {
		if _matches.is_present("provisioning-server") {
			println!("*** Running Enclave MU-RA TLS server\n");
			enclave_run_key_provisioning_server(
				enclave.as_ref(),
				sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
				&format!("localhost:{}", config.worker_mu_ra_port),
				_matches.is_present("skip-ra"),
			);
			println!("[+] Done!");
		} else if _matches.is_present("provisioning-client") {
			println!("*** Running Enclave MU-RA TLS client\n");
			enclave_request_key_provisioning(
				enclave.as_ref(),
				sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
				&format!("localhost:{}", config.worker_mu_ra_port),
				_matches.is_present("skip-ra"),
			)
			.unwrap();
			println!("[+] Done!");
		} else {
			tests::run_enclave_tests(_matches, &config.node_port);
		}
	} else {
		println!("For options: use --help");
	}
}

fn start_worker<E, T, W>(
	config: Config,
	shard: &ShardIdentifier,
	finex_uri: OpenFinexUri,
	enclave: Arc<E>,
	skip_ra: bool,
	mut node_api: Api<sr25519::Pair, WsRpcClient>,
	tokio_handle: Arc<T>,
	watch_list: Arc<W>,
) where
	T: GetTokioHandle,
	W: WatchList<Client = WsWatchingClient>,
	E: EnclaveBase
		+ DirectRequest
		+ SideChain
		+ RemoteAttestation
		+ TlsRemoteAttestation
		+ TeerexApi
		+ Clone,
{
	println!("IntegriTEE Worker v{}", VERSION);
	info!("starting worker on shard {}", shard.encode().to_base58());
	// ------------------------------------------------------------------------
	// check for required files
	check_files();
	// ------------------------------------------------------------------------
	// initialize the enclave
	#[cfg(feature = "production")]
	println!("*** Starting enclave in production mode");
	#[cfg(not(feature = "production"))]
	println!("*** Starting enclave in development mode");

	let mrenclave = enclave.get_mrenclave().unwrap();
	println!("MRENCLAVE={}", mrenclave.to_base58());

	// ------------------------------------------------------------------------
	// let new workers call us for key provisioning
	println!("MU-RA server listening on ws://{}", config.mu_ra_url());
	let ra_url = config.mu_ra_url();
	let enclave_api_key_prov = enclave.clone();
	thread::spawn(move || {
		enclave_run_key_provisioning_server(
			enclave_api_key_prov.as_ref(),
			sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
			&ra_url,
			skip_ra,
		)
	});

	// ------------------------------------------------------------------------
    // start open finex client
    println!(
        "OpenFinex Client listening on ws://{}:{}:{}",
        finex_uri.ip(),
        finex_uri.port(),
        finex_uri.path()
    );
    thread::spawn(move || enclave_run_openfinex_client(eid, finex_uri));


	// ------------------------------------------------------------------------
	// start worker api direct invocation server
	println!("rpc worker server listening on ws://{}", config.worker_url());

	let ws_handler_factory = Arc::new(WsHandlerFactory::new(enclave.clone(), watch_list));
	let ws_direct_server = WsDirectServerRunner::new(ws_handler_factory, enclave.clone());
	ws_direct_server.run(config.worker_url());

	// listen for sidechain_block import request. Later the `start_worker_api_direct_server`
	// should be merged into this one.
	let url = worker_url_into_async_rpc_url(&config.worker_url()).unwrap();

	let handle = tokio_handle.get_handle();
	let enclave_rpc_server = enclave.clone();
	handle.spawn(async move {
		substratee_worker_rpc_server::run_server(&url, enclave_rpc_server)
			.await
			.unwrap()
	});

	// ------------------------------------------------------------------------
	// start the substrate-api-client to communicate with the node
	let genesis_hash = node_api.genesis_hash.as_bytes().to_vec();

	let tee_accountid = enclave_account(enclave.as_ref());
	ensure_account_has_funds(&mut node_api, &tee_accountid);

	// ------------------------------------------------------------------------
	// perform a remote attestation and get an unchecked extrinsic back

	// get enclaves's account nonce
	let nonce = node_api.get_nonce_of(&tee_accountid).unwrap();
	info!("Enclave nonce = {:?}", nonce);

	let uxt = if skip_ra {
		println!(
			"[!] skipping remote attestation. Registering enclave without attestation report."
		);
		enclave
			.mock_register_xt(node_api.genesis_hash, nonce, &config.ext_api_url.unwrap())
			.unwrap()
	} else {
		enclave
			.perform_ra(genesis_hash, nonce, config.ext_api_url.unwrap().as_bytes().to_vec())
			.unwrap()
	};

	let mut xthex = hex::encode(uxt);
	xthex.insert_str(0, "0x");

	// send the extrinsic and wait for confirmation
	println!("[>] Register the enclave (send the extrinsic)");
	let tx_hash = node_api.send_extrinsic(xthex, XtStatus::Finalized).unwrap();
	println!("[<] Extrinsic got finalized. Hash: {:?}\n", tx_hash);

	let latest_head = init_chain_relay(&node_api, enclave.as_ref());
	println!("*** [+] Finished syncing chain relay\n");

	// ------------------------------------------------------------------------
    // Start DB Handler Thread
    crate::db_handler::DBHandler::initialize(eid);

    let mut latest_head = init_chain_relay(eid, &api);
    println!("*** [+] Finished syncing chain relay\n");
    // start disk & ipfs snapshotting
    polkadex_db::start_snapshot_loop();

    // ------------------------------------------------------------------------
    // subscribe to events and react on firing
    println!("*** Subscribing to events");
    let (sender, receiver) = channel();
    let sender2 = sender.clone();
    let api2 = api.clone();
    let _eventsubscriber = thread::Builder::new()
        .name("eventsubscriber".to_owned())
        .spawn(move || {
            api2.subscribe_events(sender2).unwrap();
        })
        .unwrap();

    let api3 = api.clone();
    let sender3 = sender.clone();
    let _block_subscriber = thread::Builder::new()
        .name("block_subscriber".to_owned())
        .spawn(move || api3.subscribe_finalized_heads(sender3))
        .unwrap();

    println!("[+] Subscribed to events. waiting...");
    let timeout = Duration::from_millis(10);
    loop {
        if let Ok(msg) = receiver.recv_timeout(timeout) {
            if let Ok(events) = parse_events(msg.clone()) {
                print_events(events, sender.clone())
            } else if let Ok(_header) = parse_header(msg.clone()) {
                latest_head = sync_chain(eid, &api, latest_head);
            }
        }
    }
}


fn request_keys<E: TlsRemoteAttestation>(
	provider_url: &str,
	_shard: &ShardIdentifier,
	enclave_api: &E,
	skip_ra: bool,
) {
	// FIXME: we now assume that keys are equal for all shards

	// initialize the enclave
	#[cfg(feature = "production")]
	println!("*** Starting enclave in production mode");
	#[cfg(not(feature = "production"))]
	println!("*** Starting enclave in development mode");

	println!("Requesting key provisioning from worker at {}", provider_url);

	enclave_request_key_provisioning(
		enclave_api,
		sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
		&provider_url,
		skip_ra,
	)
	.unwrap();
	println!("key provisioning successfully performed");
}

type Events = Vec<frame_system::EventRecord<Event, Hash>>;

fn parse_events(event: String) -> Result<Events, String> {
	let _unhex = Vec::from_hex(event).map_err(|_| "Decoding Events Failed".to_string())?;
	let mut _er_enc = _unhex.as_slice();
	Events::decode(&mut _er_enc).map_err(|_| "Decoding Events Failed".to_string())
}

fn parse_header(header: String) -> Result<Header, String> {
    serde_json::from_str(&header).map_err(|_| "Decoding Header Failed".to_string())
}

fn print_events(events: Events, _sender: Sender<String>) {
	for evr in &events {
		debug!("Decoded: phase = {:?}, event = {:?}", evr.phase, evr.event);
		match &evr.event {
			Event::Balances(be) => {
				info!("[+] Received balances event");
				debug!("{:?}", be);
				match &be {
					pallet_balances::Event::Transfer(transactor, dest, value) => {
						debug!("    Transactor:  {:?}", transactor.to_ss58check());
						debug!("    Destination: {:?}", dest.to_ss58check());
						debug!("    Value:       {:?}", value);
					},
					_ => {
						trace!("Ignoring unsupported balances event");
					},
				}
			},
			Event::Teerex(re) => {
				debug!("{:?}", re);
				match &re {
					my_node_runtime::pallet_teerex::RawEvent::AddedEnclave(sender, worker_url) => {
						println!("[+] Received AddedEnclave event");
						println!("    Sender (Worker):  {:?}", sender);
						println!("    Registered URL: {:?}", str::from_utf8(&worker_url).unwrap());
					},
					my_node_runtime::pallet_teerex::RawEvent::Forwarded(shard) => {
						println!(
							"[+] Received trusted call for shard {}",
							shard.encode().to_base58()
						);
					},
					my_node_runtime::pallet_teerex::RawEvent::CallConfirmed(sender, payload) => {
						info!("[+] Received CallConfirmed event");
						debug!("    From:    {:?}", sender);
						debug!("    Payload: {:?}", hex::encode(payload));
					},
					my_node_runtime::pallet_teerex::RawEvent::BlockConfirmed(sender, payload) => {
						info!("[+] Received BlockConfirmed event");
						debug!("    From:    {:?}", sender);
						debug!("    Payload: {:?}", hex::encode(payload));
					},
					my_node_runtime::pallet_teerex::RawEvent::ShieldFunds(incognito_account) => {
						info!("[+] Received ShieldFunds event");
						debug!("    For:    {:?}", incognito_account);
					},
					my_node_runtime::pallet_teerex::RawEvent::UnshieldedFunds(
						incognito_account,
					) => {
						info!("[+] Received UnshieldedFunds event");
						debug!("    For:    {:?}", incognito_account);
					},
					_ => {
						trace!("Ignoring unsupported pallet_teerex event");
					},
				}
			},
			_ => {
				trace!("Ignoring event {:?}", evr);
			},
		}
	}
}

pub fn init_chain_relay<E: EnclaveBase + SideChain>(
	api: &Api<sr25519::Pair, WsRpcClient>,
	enclave_api: &E,
) -> Header {
	let genesis_hash = api.get_genesis_hash().unwrap();
	let genesis_header: Header = api.get_header(Some(genesis_hash)).unwrap().unwrap();
	info!("Got genesis Header: \n {:?} \n", genesis_header);
	let grandpas = api.grandpa_authorities(Some(genesis_hash)).unwrap();
	let grandpa_proof = api.grandpa_authorities_proof(Some(genesis_hash)).unwrap();

	debug!("Grandpa Authority List: \n {:?} \n ", grandpas);

	let authority_list = VersionedAuthorityList::from(grandpas);

	let latest = enclave_api
		.init_chain_relay(genesis_header, authority_list, grandpa_proof)
		.unwrap();

	info!("Finished initializing chain relay, syncing....");

	let polkadex_accounts: Vec<PolkadexAccount> = polkadex::get_main_accounts(latest.clone(), api);

    enclave_accept_pdex_accounts(eid, polkadex_accounts).unwrap();

    info!("Finishing retrieving Polkadex Accounts, ...");

    info!("Initializing Polkadex Orderbook Mirror");

    info!("Loading Orders from Orderbook Storage");
    let signed_orders = polkadex_db::orderbook::load_orderbook_mirror()
        .unwrap() // TODO: Replace unwrap
        .lock()
        .unwrap()
        .read_all()
        .ok()
        .unwrap();

    enclave_load_orders_to_memory(eid, signed_orders).unwrap();

    info!("Finished loading Orderbook Storage ...");
    sync_chain(eid, api, latest)
}

/// Starts block production
///
/// Returns the last synced header of layer one
pub fn sync_chain(
    eid: sgx_enclave_id_t,
    api: &Api<sr25519::Pair>,
    last_synced_head: Header,
) -> Header {
    // obtain latest finalized block from layer one
    debug!("Getting current head");
    let curr_head: SignedBlock = api
        .get_finalized_head()
        .unwrap()
        .map(|hash| api.get_signed_block(Some(hash)).unwrap())
        .unwrap()
        .unwrap();

    if curr_head.block.header.hash() == last_synced_head.hash() {
        // we are already up to date, do nothing
        return curr_head.block.header;
    }

    let mut blocks_to_sync = vec![curr_head.clone()];

    // Todo: Check, is this dangerous such that it could be an eternal or too big loop?
    let mut head = curr_head.clone();

    let no_blocks_to_sync = head.block.header.number - last_synced_head.number;
    if no_blocks_to_sync > 1 {
        println!(
            "Chain Relay is synced until block: {:?}",
            last_synced_head.number
        );
        println!(
            "Last finalized block number: {:?}\n",
            head.block.header.number
        );
    }

    while head.block.header.parent_hash != last_synced_head.hash() {
        head = api
            .get_signed_block(Some(head.block.header.parent_hash))
            .unwrap()
            .unwrap();
        blocks_to_sync.push(head.clone());

        if head.block.header.number % BLOCK_SYNC_BATCH_SIZE == 0 {
            println!(
                "Remaining blocks to fetch until last synced header: {:?}",
                head.block.header.number - last_synced_head.number
            )
        }
    }
    blocks_to_sync.reverse();

    let tee_accountid = enclave_account(eid);

    // only feed BLOCK_SYNC_BATCH_SIZE blocks at a time into the enclave to save enclave state regularly
    let mut i = blocks_to_sync[0].block.header.number as usize;
    for chunk in blocks_to_sync.chunks(BLOCK_SYNC_BATCH_SIZE as usize) {
        let tee_nonce = get_nonce(&api, &tee_accountid);

        // sync enclave with chain
        if let Err(e) = enclave_sync_chain(eid, chunk.to_vec(), tee_nonce) {
            error!("{}", e);
            // enclave might not have synced
            return last_synced_head;
        };

        i += chunk.len();
        println!(
            "Synced {} blocks out of {} finalized blocks",
            i,
            blocks_to_sync[0].block.header.number as usize + blocks_to_sync.len()
        )
    }

    curr_head.block.header
}

/// gets a list of blocks that need to be synced, ordered from oldest to most recent header
/// blocks that need to be synced are all blocks from the current header to the last synced header, iterating over parent
fn get_blocks_to_sync(
	api: &Api<sr25519::Pair, WsRpcClient>,
	last_synced_head: &Header,
	curr_head: &SignedBlock,
) -> Vec<SignedBlock> {
	let mut blocks_to_sync = Vec::<SignedBlock>::new();

	// add blocks to sync if not already up to date
	if curr_head.block.header.hash() != last_synced_head.hash() {
		blocks_to_sync.push((*curr_head).clone());

		// Todo: Check, is this dangerous such that it could be an eternal or too big loop?
		let mut head = (*curr_head).clone();
		let no_blocks_to_sync = head.block.header.number - last_synced_head.number;
		if no_blocks_to_sync > 1 {
			println!("Chain Relay is synced until block: {:?}", last_synced_head.number);
			println!("Last finalized block number: {:?}\n", head.block.header.number);
		}
		while head.block.header.parent_hash != last_synced_head.hash() {
			debug!("Getting head of hash: {:?}", head.block.header.parent_hash);
			head = api.signed_block(Some(head.block.header.parent_hash)).unwrap().unwrap();
			blocks_to_sync.push(head.clone());

			if head.block.header.number % BLOCK_SYNC_BATCH_SIZE == 0 {
				println!(
					"Remaining blocks to fetch until last synced header: {:?}",
					head.block.header.number - last_synced_head.number
				)
			}
		}
		blocks_to_sync.reverse();
	}
	blocks_to_sync
}

fn init_shard(shard: &ShardIdentifier) {
	let path = format!("{}/{}", SHARDS_PATH, shard.encode().to_base58());
	println!("initializing shard at {}", path);
	fs::create_dir_all(path.clone()).expect("could not create dir");

	let path = format!("{}/{}", path, ENCRYPTED_STATE_FILE);
	if Path::new(&path).exists() {
		println!("shard state exists. Overwrite? [y/N]");
		let buffer = &mut String::new();
		stdin().read_line(buffer).unwrap();
		match buffer.trim() {
			"y" | "Y" => (),
			_ => return,
		}
	}
	let mut file = fs::File::create(path).unwrap();
	file.write_all(b"").unwrap();
}

// get the public signing key of the TEE
fn enclave_account<E: EnclaveBase>(enclave_api: &E) -> AccountId32 {
	let tee_public = enclave_api.get_ecc_signing_pubkey().unwrap();
	trace!("[+] Got ed25519 account of TEE = {}", tee_public.to_ss58check());
	AccountId32::from(*tee_public.as_array_ref())
}

// Alice plays the faucet and sends some funds to the account if balance is low
fn ensure_account_has_funds(api: &mut Api<sr25519::Pair, WsRpcClient>, accountid: &AccountId32) {
	let alice = AccountKeyring::Alice.pair();
	info!("encoding Alice's public 	= {:?}", alice.public().0.encode());
	let alice_acc = AccountId32::from(*alice.public().as_array_ref());
	info!("encoding Alice's AccountId = {:?}", alice_acc.encode());

	let free = api.get_free_balance(&alice_acc);
	info!("    Alice's free balance = {:?}", free);
	let nonce = api.get_nonce_of(&alice_acc).unwrap();
	info!("    Alice's Account Nonce is {}", nonce);

	// check account balance
	let free = api.get_free_balance(&accountid).unwrap();
	info!("TEE's free balance = {:?}", free);

	if free < 1_000_000_000_000 {
		let signer_orig = api.signer.clone();
		api.signer = Some(alice);

		println!("[+] bootstrap funding Enclave form Alice's funds");
		let xt = api.balance_transfer(GenericAddress::Id(accountid.clone()), 1_000_000_000_000);
		let xt_hash = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();
		info!("[<] Extrinsic got finalized. Hash: {:?}\n", xt_hash);

		//verify funds have arrived
		let free = api.get_free_balance(&accountid);
		info!("TEE's NEW free balance = {:?}", free);

		api.signer = signer_orig;
	}
}

/// # Safety
///
/// FFI are always unsafe
#[no_mangle]
pub unsafe extern "C" fn ocall_write_order_to_db(
    order: *const u8,
    order_size: u32,
) -> sgx_status_t {
    debug!("    Entering ocall_write_order_to_db");
    let mut status = sgx_status_t::SGX_SUCCESS;
    let mut order_slice = slice::from_raw_parts(order, order_size as usize);

    let signed_order: SignedOrder = match Decode::decode(&mut order_slice) {
        Ok(order) => order,
        Err(_) => {
            error!("Could not decode SignedOrder");
            status = sgx_status_t::SGX_ERROR_UNEXPECTED;
            SignedOrder::default()
        }
    };
    if status == sgx_status_t::SGX_ERROR_UNEXPECTED {
        return status;
    }
    // TODO: Do we need error handling here?
    let order_id = signed_order.order_id.clone();
    thread::spawn(move || -> Result<(), PolkadexDBError> {
        let mutex = polkadex_db::orderbook::load_orderbook_mirror()?;
        let mut orderbook_mirror: MutexGuard<OrderbookMirror<DiskStorageHandler>> =
            mutex.lock().unwrap();
        orderbook_mirror.write(order_id, &signed_order);
        Ok(())
    });
    status
}

/// # Safety
///
/// FFI are always unsafe
#[no_mangle]
pub unsafe extern "C" fn ocall_send_release_extrinsic(
    extrinsic: *const u8,
    extrinsic_size: u32,
) -> sgx_status_t {
    debug!("Entering ocall_send_release_extrinsic");
    let mut status = sgx_status_t::SGX_SUCCESS;
    let mut extrinsic_slice = slice::from_raw_parts(extrinsic, extrinsic_size as usize);
    let api = Api::<sr25519::Pair>::new(NODE_URL.lock().unwrap().clone()).unwrap();
    let release_extrinsic_calls: Vec<u8> = match Decode::decode(&mut extrinsic_slice) {
        Ok(calls) => calls,
        Err(_) => {
            error!("Could not decode release calls");
            status = sgx_status_t::SGX_ERROR_UNEXPECTED;
            vec![]
        }
    };
    api.send_extrinsic(hex_encode(release_extrinsic_calls), XtStatus::Ready)
        .unwrap();
    status
}
