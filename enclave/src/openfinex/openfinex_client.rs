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

use crate::openfinex::client_utils;
use crate::openfinex::jwt;
use crate::openfinex::market_repo::{MarketRepository, MarketsRequestSender};
use crate::openfinex::response_handler::{PolkadexResponseHandler, TcpResponseHandler};
use crate::openfinex::response_object_mapper::ResponseObjectMapper;
use crate::openfinex::response_parser::TcpResponseParser;
use crate::openfinex::string_serialization::ResponseDeserializerImpl;
use crate::polkadex_cache::market_cache::LocalMarketCacheFactory;
use crate::polkadex_gateway::PolkaDexGatewayCallbackFactory;
use client_utils::{Message, Opcode, Payload};
use codec::Decode;
use lazy_static::lazy_static;
use log::*;
use polkadex_sgx_primitives::OpenFinexUri;
use sgx_types::*;
use std::borrow::ToOwned;
use std::boxed::Box;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::slice;
use std::string::String;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, SgxRwLock};
use std::vec::Vec;

static GLOBAL_CONTEXT_COUNT: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref GLOBAL_CONTEXTS: SgxRwLock<HashMap<usize, AtomicPtr<TcpClient>>> =
        SgxRwLock::new(HashMap::new());
}

/// This encapsulates the TCP-level connection, some connection
/// state, and the underlying TLS-level session.
struct TcpClient {
    socket: TcpStream,
    uri: OpenFinexUri,
    //closing: bool,
    //clean_closure: bool,
    received_plaintext: Vec<u8>,
    sendable_plaintext: Vec<u8>,
    response_handler: Box<dyn TcpResponseHandler>,
    markets_request_sender: Arc<dyn MarketsRequestSender>,
    payload_string_buffer: String,
}

impl TcpClient {
    fn new(
        socket_address: c_int,
        uri: OpenFinexUri,
        response_handler: Box<dyn TcpResponseHandler>,
        markets_request_sender: Arc<dyn MarketsRequestSender>,
    ) -> TcpClient {
        TcpClient {
            socket: TcpStream::new(socket_address).unwrap(),
            uri,
            //closing: false,
            //clean_closure: false,
            received_plaintext: Vec::new(),
            sendable_plaintext: Vec::new(),
            response_handler,
            markets_request_sender,
            payload_string_buffer: String::new(),
        }
    }

    fn jwt_handshake(&mut self) {
        //FIXME: this should probably some proper hostname
        let ip = if self.uri.ip() == "127.0.0.1" {
            "localhost".to_owned()
        } else {
            self.uri.ip()
        };
        let host = format!("Host: {}:{}\r\n", ip, self.uri.port());
        let authorization = format!(
            "Authorization: Bearer {}\r\n",
            jwt::create_jwt("U487205863".to_owned())
        );
        let ws_version =
            "Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n";
        let connection = "Connection: Upgrade\r\nUpgrade: websocket\r\n";
        let data = format!(
            "GET {} HTTP/1.1\r\n{}{}{}{}\r\n",
            self.uri.path(),
            ws_version,
            connection,
            authorization,
            host
        );
        debug!("Sending http request: {}", data);
        if let Err(e) = self.socket.write(data.as_bytes()) {
            error!("Could not handshake with openfinex server: {}", e);
        };
    }

    fn wants_write(&self) -> bool {
        !self.sendable_plaintext.is_empty()
    }

    // we always want to read except when we already received some text
    // we want to process
    fn wants_read(&self) -> bool {
        self.received_plaintext.is_empty()
    }

    /* fn parse_http_response(&mut self, buffer: &[u8]) -> c_int {
        // split header and body
        let mut index = 0;
        let mut _prev_byte = 0;
        let new_line = 0;
        // Header - Body seperator: [13, 10, 13, 10] = \r\n\r\n
        // start with header
        for byte in buffer.iter() {
            if *byte == 129 {
                break;
            } else if *byte == 0 {
                // EOF
                break;
            }
            index += 1;
            _prev_byte = *byte;
        }
        let header = String::from_utf8_lossy(&buffer[0..index]);
        debug!("Header: {}", header);
        // get body
        let body = String::new();
        // TODO if necessary
        debug!("Body: {}", body);
        1
    } */

    /// We're ready to do a read.
    fn do_read(&mut self) -> io::Result<usize> {
        //FIXME: maybe we can use up buffer read here?
        //let mut buffer = [0 as u8; 1028]; // Dummy buffer. will not be necessary with tls client
        let mut start_bytes = [0u8; 2];
        let rc = self.socket.read(&mut start_bytes);
        if rc.is_err() {
            error!("TLS read error: {:?}", rc);
            //self.closing = true;
            return rc;
        }

        // If we're ready but there's no data: EOF.
        if rc.unwrap() == 0 {
            //self.closing = true;
            //self.clean_closure = true;
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF",
            ));
        }
        if start_bytes[0] == 72 {
            // parse http response
            let mut buffer = [0u8; 512];
            self.socket.read(&mut buffer).unwrap(); //TODO: Proper Error handling

            debug!(
                "Received Http response: HT{}",
                String::from_utf8_lossy(&buffer)
            );
            // handshake successful, send a subscription:
            self.subscribe_matches();
            self.send_markets_request();
        } else {
            // direct tcp message
            let fin = client_utils::last_frame(start_bytes[0]);
            if let Some(message) = self.read_tcp_buffer(start_bytes.to_vec()) {
                match message.opcode {
                    Opcode::PingOp => {
                        self.send_pong();
                    }
                    Opcode::TextOp => {
                        if fin {
                            debug!("Sending to handler : {:?}", message.payload);
                            self.response_handler.handle_text_op(message.payload);
                        } else {
                            self.append_string_payload(message.payload)
                        }
                    }
                    Opcode::ContinuationOp => {
                        if fin {
                            self.append_string_payload(message.payload);
                            debug!(
                                "Sending to handler : {:?}",
                                self.payload_string_buffer.clone()
                            );
                            self.response_handler
                                .handle_text_op(Payload::Text(self.payload_string_buffer.clone()));
                            self.payload_string_buffer = String::new();
                        } else {
                            self.append_string_payload(message.payload)
                        }
                    }
                    _ => error!("received unexpected op: {:?}", message.opcode),
                }
            }
        }
        Ok(start_bytes.len())
    }

    fn read_tcp_buffer(&mut self, start_bytes: Vec<u8>) -> Option<Message> {
        // https://stackoverflow.com/questions/41115870/is-binary-opcode-encoding-and-decoding-implementation-specific-in-websockets
        //let fin = buf1[0] >> 7; // TODO check this, required for handling fragmented messages
        let rsv = (start_bytes[0] >> 4) & 0b0111;
        if rsv != 0 {
            return None;
        }
        let opcode: Opcode = (start_bytes[0] & 0b0000_1111).into();

        // Take the 2nd byte and read every bit except the Most significant bit
        let pay_len = start_bytes[1] & 0b0111_1111;

        let payload_length: u64 = match pay_len {
            127 => {
                // Your length is a uint64 of byte 3 to 8
                let mut length_buffer = [0u8; 8];
                if let Err(rc) = self.socket.read(&mut length_buffer) {
                    error!("TLS read error: {:?}", rc);
                    return None;
                };
                u64::from_be_bytes(length_buffer)
            }
            126 => {
                // Your length is an uint16 of byte 3 and 4
                let mut length_buffer = [0u8; 2];
                if let Err(rc) = self.socket.read(&mut length_buffer) {
                    error!("TLS read error: {:?}", rc);
                    return None;
                };
                u16::from_be_bytes(length_buffer) as u64
            }
            // Byte is 125 or less thats your length
            _ => pay_len as u64,
        };

        /* let (payload_length, payload_buf) = match pay_len {
            127 => {
                // Your length is a uint64 of byte 3 to 8
                error!("buffer is too small for this long message..")
                let slice: [u8; 8] = buffer[2 .. 10].to_vec().try_into()
                    .unwrap_or_else( |_| {error!("Invalid payload length"); [0; 8]});
                let length = u64::from_be_bytes(slice);
                (length, buffer[10 .. (length+10) as usize].to_vec())
            },
            126 => {
                // Your length is an uint16 of byte 3 and 4
                let slice: [u8; 2] = buffer[2 .. 4].to_vec().try_into()
                    .unwrap_or_else( |_| {error!("Invalid payload length"); [0; 2]});
                let length = u16::from_be_bytes(slice);
                (length as u64, buffer[4 .. (length+4) as usize].to_vec())
            }
            // Byte is 125 or less thats your length
            _   => (pay_len as u64, buffer[2 .. (pay_len+2) as usize].to_vec())
        }; */
        debug!("payload_length: {}", payload_length);
        let mut payload_buf = vec![0; payload_length as usize];
        if payload_length > 0 {
            if let Err(rc) = self.socket.read(&mut payload_buf) {
                error!("TLS read error: {:?}", rc);
                return None;
            };
        }

        // payloads larger than 125 bytes are not allowed for control frames
        match opcode {
            Opcode::CloseOp | Opcode::PingOp if payload_length > 125 => panic!(),
            _ => (),
        }

        // No mask from server
        /* let masking_key = try!(stream.read_exact(4));
        let mut masked_payload_buf = try!(stream.read_exact(payload_length as uint));
        // unmask the payload in-place
        for (i, octet) in masked_payload_buf.iter_mut().enumerate() {
            *octet = *octet ^ masking_key[i % 4];
        }
        let payload_buf = masked_payload_buf; */

        let payload: Payload = match opcode {
            Opcode::TextOp => Payload::Text(String::from_utf8(payload_buf.to_vec()).unwrap()),
            Opcode::BinaryOp => Payload::Binary(payload_buf.to_vec()),
            Opcode::CloseOp => Payload::Empty,
            Opcode::PingOp => {
                debug!("Ping");
                Payload::Binary(payload_buf.to_vec())
            }
            Opcode::PongOp => {
                debug!("Pong");
                Payload::Binary(payload_buf.to_vec())
            }
            _ => Payload::Text(String::from_utf8(payload_buf.to_vec()).unwrap()), // ContinuationOp
        };

        // for now only take text option
        Some(Message::new(payload, opcode))
    }

    fn append_string_payload(&mut self, payload: Payload) {
        debug!("Appending to payload: {:?}", payload);
        if let Payload::Text(new_text) = payload {
            self.payload_string_buffer.push_str(&new_text);
        }
    }
    /// write intern buffer to Tcpstream
    fn do_write(&mut self) {
        let request: &[u8] = &self.sendable_plaintext;
        self.socket.write_all(request).unwrap();
        self.flush_buffer();
    }

    /// write to intern buffer
    fn write_buffer(&mut self, request: &[u8]) {
        self.sendable_plaintext = request.to_owned();
        self.do_write()
    }

    fn flush_buffer(&mut self) {
        self.sendable_plaintext = vec![];
    }

    fn write_masked_text(&mut self, plaintext: &[u8]) {
        let masked_request = client_utils::mask(plaintext, Opcode::TextOp);
        self.write_buffer(&masked_request)
    }

    fn send_pong(&mut self) {
        let masked_request = client_utils::mask(&[0u8], Opcode::PongOp);
        self.write_buffer(&masked_request)
    }

    fn subscribe_matches(&mut self) {
        let plaintext = r#"[1,51,"subscribe",["admin",["events.order","events.trade"]]]"#;
        let masked_request = client_utils::mask(plaintext.as_bytes(), Opcode::TextOp);
        self.sendable_plaintext = masked_request;
        self.do_write()
    }

    fn send_markets_request(&mut self) {
        let request = match self.markets_request_sender.get_markets_ws_request() {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to get markets request string: {}, will not send any request",
                    e
                );
                return;
            }
        };
        let masked_request = client_utils::mask(request.as_bytes(), Opcode::TextOp);
        self.sendable_plaintext = masked_request;
        self.do_write()
    }
}

struct Sessions;

impl Sessions {
    /// store current TLS session (contained in TlsClient) to Global pointer,
    /// such that the connection will not be disrupted as soon as the
    /// enclave returns.
    /// Every session is stored in Hashmap, with global context count acting as key
    /// Returns current session id (=hash map key)
    fn new_session(svr_ptr: *mut TcpClient) -> Option<usize> {
        match GLOBAL_CONTEXTS.write() {
            Ok(mut gctxts) => {
                let curr_id = GLOBAL_CONTEXT_COUNT.fetch_add(1, Ordering::SeqCst);
                gctxts.insert(curr_id, AtomicPtr::new(svr_ptr));
                Some(curr_id)
            }
            Err(x) => {
                println!("Locking global context SgxRwLock failed! {:?}", x);
                None
            }
        }
    }

    /// load stored session from global pointer
    fn get_session(sess_id: size_t) -> Option<*mut TcpClient> {
        match GLOBAL_CONTEXTS.read() {
            Ok(gctxts) => match gctxts.get(&sess_id) {
                Some(s) => Some(s.load(Ordering::SeqCst)),
                None => {
                    println!("Global contexts cannot find session id = {}", sess_id);
                    None
                }
            },
            Err(x) => {
                println!(
                    "Locking global context SgxRwLock failed on get_session! {:?}",
                    x
                );
                None
            }
        }
    }

    /// remove stored sessions from hashmap
    fn remove_session(sess_id: size_t) {
        if let Ok(mut gctxts) = GLOBAL_CONTEXTS.write() {
            if let Some(session_ptr) = gctxts.get(&sess_id) {
                let session_ptr = session_ptr.load(Ordering::SeqCst);
                let session = unsafe { &mut *session_ptr };
                let _ = unsafe { Box::<TcpClient>::from_raw(session as *mut _) };
                let _ = gctxts.remove(&sess_id);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tcp_client_new(
    fd: c_int,
    finex_uri: *const u8,
    finex_uri_size: usize,
) -> usize {
    let mut uri_vec = slice::from_raw_parts(finex_uri, finex_uri_size as usize);
    let finex_uri = match OpenFinexUri::decode(&mut uri_vec) {
        Ok(uri) => uri,
        Err(e) => {
            error!("Could not decode finex uri: {:?}", e);
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
    };

    let market_cache_provider = Arc::new(LocalMarketCacheFactory::create());
    let market_repository = Arc::new(MarketRepository::new(market_cache_provider.clone()));

    let response_handler = Box::new(PolkadexResponseHandler::new(
        PolkaDexGatewayCallbackFactory::create(),
        market_repository.clone(),
        Arc::new(TcpResponseParser {}),
        Arc::new(ResponseObjectMapper::new(Arc::new(
            ResponseDeserializerImpl::new(market_cache_provider),
        ))),
    ));

    let mut tcp_client = TcpClient::new(fd, finex_uri, response_handler, market_repository);
    tcp_client.jwt_handshake();
    let client_pointer: *mut TcpClient = Box::into_raw(Box::new(tcp_client));

    // create session and return current session id
    Sessions::new_session(client_pointer).unwrap_or(0xFFFF_FFFF_FFFF_FFFF)
}

#[no_mangle]
pub extern "C" fn tcp_client_read(session_id: usize) -> c_int {
    // load session
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *session_ptr };
        if let Err(e) = session.do_read() {
            error!("Could not read from TCP stream: {}", e);
            return -1;
        };
        1
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn tcp_client_write(session_id: usize) -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *session_ptr };

        session.do_write();
        1
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn tcp_client_close(session_id: usize) {
    Sessions::remove_session(session_id)
}

#[no_mangle]
pub extern "C" fn tcp_client_wants_read(session_id: usize) -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &*session_ptr };
        session.wants_read() as c_int
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn tcp_client_wants_write(session_id: usize) -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &*session_ptr };
        session.wants_write() as c_int
    } else {
        -1
    }
}

/// Interface to client
#[derive(Debug, Clone, PartialEq)]
pub struct OpenFinexClientInterface {
    session_id: usize,
}

impl OpenFinexClientInterface {
    //TODO: is read write lock enough?
    pub fn new(session_id: usize) -> Self {
        OpenFinexClientInterface { session_id }
    }

    pub fn send_request(self, plaintext: &[u8]) -> Result<(), String> {
        if let Some(session_ptr) = Sessions::get_session(self.session_id) {
            let session = unsafe { &mut *session_ptr };
            session.write_masked_text(plaintext);
            Ok(())
        } else {
            Err(String::from("Failed to send request"))
        }
    }
}
