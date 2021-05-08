mod types;
use embedded_websocket::{
    framer::{Framer, FramerError},
    WebSocketClient, WebSocketCloseStatusCode, WebSocketOptions, WebSocketSendMessageType,
};
use sgx_tstd as std;
use std::{error::Error, net::TcpStream};
pub fn subscribe_to_openfinex_api() {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
