#![cfg_attr(all(not(target_env = "sgx"), not(feature = "std")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

mod types;
use embedded_websocket::{
    framer::{Framer, FramerError},
    WebSocketClient, WebSocketCloseStatusCode, WebSocketOptions, WebSocketSendMessageType,
};

use sgx_tstd::{error::Error, net::TcpStream};
pub fn subscribe_to_openfinex_api() {}
