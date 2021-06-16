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

use crate::openfinex::client_utils;
use crate::openfinex::jwt;
use crate::openfinex::response_handler::{PolkadexResponseHandler, TcpResponseHandler};
use crate::openfinex::response_object_mapper::ResponseObjectMapper;
use crate::openfinex::response_parser::TcpResponseParser;
use crate::polkadex_gateway::PolkaDexGatewayCallbackFactory;
use client_utils::{Opcode};
use codec::{Decode};
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
}

impl TcpClient {
    fn new(
        socket_address: c_int,
        uri: OpenFinexUri,
        response_handler: Box<dyn TcpResponseHandler>,
    ) -> TcpClient {
        TcpClient {
            socket: TcpStream::new(socket_address).unwrap(),
            uri,
            //closing: false,
            //clean_closure: false,
            received_plaintext: Vec::new(),
            sendable_plaintext: Vec::new(),
            response_handler,
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
        if let Err(e) = self.socket.write(data.as_bytes()){
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
        let mut buffer = [0 as u8; 512]; // Dummy buffer. will not be necessary with tls client
        let rc = self.socket.read(&mut buffer);
        if rc.is_err() {
            error!("TLS read error: {:?}", rc);
            //self.closing = true;
            return rc;
        }

        // If we're ready but there's no data: EOF.
        if rc.unwrap() == 0 {
            //self.closing = true;
            //self.clean_closure = true;
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "EOF"));
        }

        if buffer[0] == 72 {
            // parse http response
            debug!(
                "Received Http response: {}",
                String::from_utf8_lossy(&buffer)
            ); //FIXME: replace with debug
               // handshake successful, send a subscription:
            self.subscribe_matches();
        } else {
            // direct tcp message
            if let Some(message) = client_utils::read_tcp_buffer(buffer.to_vec()) {
                match message.opcode {
                    Opcode::PingOp => {
                        self.send_pong();
                    }
                    Opcode::TextOp => {
                        debug!("received plaintext : {:?}", message.payload);
                        self.response_handler.handle_text_op(message.payload);
                    }
                    _ => error!("received unexpected op: {:?}", message.opcode),
                }
            }
        }
        Ok(buffer.len())
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
        let masked_request = client_utils::mask(&[0 as u8], Opcode::PongOp);
        self.write_buffer(&masked_request)
    }

    fn subscribe_matches(&mut self) {
        let plaintext = r#"[1,51,"subscribe",["admin",["events.order","events.trade"]]]"#;
        let masked_request = client_utils::mask(plaintext.as_bytes(), Opcode::TextOp);
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

    let response_handler = Box::new(PolkadexResponseHandler::new(
        PolkaDexGatewayCallbackFactory::create(),
        Arc::new(TcpResponseParser {}),
        Arc::new(ResponseObjectMapper {}),
    ));

    let mut tcp_client = TcpClient::new(fd, finex_uri, response_handler);
    tcp_client.jwt_handshake();
    let client_pointer: *mut TcpClient = Box::into_raw(Box::new(tcp_client));

    // create session and return current session id
    let session_id = match Sessions::new_session(client_pointer) {
        Some(s) => s,
        None => 0xFFFF_FFFF_FFFF_FFFF,
    };
    session_id
}

#[no_mangle]
pub extern "C" fn tcp_client_read(session_id: usize) -> c_int {
    // load session
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &mut *session_ptr };
        if let Err(e) = session.do_read() {
            error!("Could not read from TCP stream: {}", e);
            return -1
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
        let result = session.wants_read() as c_int;
        result
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn tcp_client_wants_write(session_id: usize) -> c_int {
    if let Some(session_ptr) = Sessions::get_session(session_id) {
        let session = unsafe { &*session_ptr };
        let result = session.wants_write() as c_int;
        result
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

    pub fn send_request(self, plaintext: &[u8]) -> Result<(), ()> {
        if let Some(session_ptr) = Sessions::get_session(self.session_id) {
            let session = unsafe { &mut *session_ptr };
            session.write_masked_text(plaintext);
            Ok(())
        } else {
            Err(())
        }
    }
}
