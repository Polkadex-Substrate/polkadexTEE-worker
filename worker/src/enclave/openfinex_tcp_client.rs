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

extern crate sgx_types;
use sgx_types::*;

use mio::tcp::TcpStream;

use log::*;
use codec::Encode;
use std::os::unix::io::AsRawFd;
use std::net::SocketAddr;
use std::str;
use std::io::{self};
use polkadex_sgx_primitives::OpenFinexUri;

extern {
    fn tcp_client_new(eid: sgx_enclave_id_t, retval: *mut usize,
            fd: c_int, finex_uri: *const u8, finex_uri_size: usize) -> sgx_status_t;
    fn tcp_client_read(eid: sgx_enclave_id_t, retval: *mut c_int,
                        session_id: usize) -> sgx_status_t;
    fn tcp_client_write(eid: sgx_enclave_id_t, retval: *mut c_int,
                        session_id: usize) -> sgx_status_t;
    fn tcp_client_wants_read(eid: sgx_enclave_id_t, retval: *mut c_int,
                            session_id: usize) -> sgx_status_t;
    fn tcp_client_wants_write(eid: sgx_enclave_id_t, retval: *mut c_int,
                            session_id: usize) -> sgx_status_t;
    fn tcp_client_close(eid: sgx_enclave_id_t,
                     session_id: usize) -> sgx_status_t;
}

const CLIENT: mio::Token = mio::Token(0);

/// This encapsulates the TCP-level connection, some connection
/// state, and the underlying TLS-level session.
struct TcpClient {
    enclave_id: sgx_enclave_id_t,
    // server
    socket: TcpStream,
    closing: bool,
    // id pointing (=key hash) to the acutal Client within the enclave
    client_id: usize,
}

impl TcpClient {
    fn ready(&mut self,
             poll: &mut mio::Poll,
             //Events is passed as an argument to Poll::poll and will be
             // used to receive any new readiness events received since the last poll.
             ev: &mio::Event) -> bool {

        assert_eq!(ev.token(), CLIENT);

        if ev.readiness().is_readable() {
            self.do_read();
        }

        if ev.readiness().is_writable() {
            self.do_write();
        }

        if self.is_closed() {
            println!("Connection closed");
            return false;
        }

        self.reregister(poll);

        true
    }
}

impl TcpClient {
    /// Creates a new TLSClient within the enclave
    fn new(enclave_id: sgx_enclave_id_t, sock: TcpStream, finex_uri: OpenFinexUri) -> Option<TcpClient> {
        let mut client_id: usize = 0xFFFF_FFFF_FFFF_FFFF;

        let result = unsafe {
            tcp_client_new(
                enclave_id,
                &mut client_id,
                sock.as_raw_fd(),
                finex_uri.encode().as_ptr(),
                finex_uri.encode().len(),
            )
        };

        if result != sgx_status_t::SGX_SUCCESS {
            println!("[-] ECALL Enclave [http_client_new] Failed {}!", result);
            return Option::None;
        }

        if client_id == 0xFFFF_FFFF_FFFF_FFFF {
            println!("[-] Could not create a new HTTP Client within the enclave");
            return Option::None;
        }

        Option::Some(
            TcpClient {
            enclave_id,
            socket: sock,
            closing: false,
            client_id,
        })
    }

    fn close(&self) {

        let retval = unsafe {
            tcp_client_close(self.enclave_id, self.client_id)
        };

        if retval != sgx_status_t::SGX_SUCCESS {
            println!("[-] ECALL Enclave [tcp_client_close] Failed {}!", retval);
        }
    }

    /// read from server
    fn read_tcp(&self) -> isize {
        let mut retval = -1;
        let result = unsafe {
            tcp_client_read(self.enclave_id,
                            &mut retval,
                            self.client_id
                        )
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { retval as isize }
            _ => {
                println!("[-] ECALL Enclave [tcp_client_read] Failed {}!", result);
                -1
            }
        }
    }
    /// write from client to server
    fn write_tcp(&self) -> isize {
        let mut retval = -1;
        let result = unsafe {
            tcp_client_write(self.enclave_id,
                             &mut retval,
                             self.client_id)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { retval as isize }
            _ => {
                println!("[-] ECALL Enclave [tcp_client_write] Failed {}!", result);
                -1
            }
        }
    }

    fn wants_read(&self) -> bool {
        let mut retval = -1;
        let result = unsafe {
            tcp_client_wants_read(self.enclave_id,
                                  &mut retval,
                                  self.client_id)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { },
            _ => {
                println!("[-] ECALL Enclave [tcp_client_wants_read] Failed {}!", result);
                return false;
            }
        }
        !matches!(retval, 0)
    }

    fn wants_write(&self) -> bool {
        let mut retval = -1;
        let result = unsafe {
            tcp_client_wants_write(self.enclave_id,
                                   &mut retval,
                                   self.client_id)
        };

        match result {
            sgx_status_t::SGX_SUCCESS => { },
            _ => {
                println!("[-] ECALL Enclave [http_client_wants_write] Failed {}!", result);
                return false;
            }
        }
        !matches!(retval, 0)

    }

    /// We're ready to do a read.
    fn do_read(&mut self) {
        // BUFFER_SIZE = 1024, just for test.
        // Do read all plaintext, you need to do more ecalls to get buffer size and buffer.
        let rc = self.read_tcp();
        if rc == -1 {
            println!("TLS read error: {:?}", rc);
            self.closing = true;
        }
    }

    fn do_write(&mut self) {
        self.write_tcp();
    }

    /// The connect is not guaranteed to have started until it is registered at
    /// this point
    /// (Polls for readiness events on all registered values)
    fn register(&self, poll: &mut mio::Poll) {
        let interest = self.ready_interest();
        poll.register(&self.socket,
                      CLIENT,
                      interest,
                      mio::PollOpt::level() | mio::PollOpt::oneshot())
            .unwrap();
    }

    fn ready_interest(&self) -> mio::Ready {
        let rd = self.wants_read();
        let wr = self.wants_write();
        if rd && wr {
            mio::Ready::readable() | mio::Ready::writable()
        } else if wr {
            mio::Ready::writable()
        } else {
            mio::Ready::readable()
        }
    }

    fn reregister(&self, poll: &mut mio::Poll) {
        let interest = self.ready_interest();
        poll.reregister(&self.socket,
                        CLIENT,
                        interest,
                        mio::PollOpt::level() | mio::PollOpt::oneshot())
            .unwrap();
    }

    fn is_closed(&self) -> bool {
        self.closing
    }
}

/// We implement `io::Write` and pass through to the TLS session
impl io::Write for TcpClient {
    fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
        Ok(self.write_tcp() as usize)
    }
    // unused
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for TcpClient {
    fn read(&mut self, _bytes: &mut [u8]) -> io::Result<usize> {
        Ok(self.read_tcp() as usize)
    }
}

// IPV4.. or?
fn get_socket_addr(uri: OpenFinexUri) -> SocketAddr {
    use std::net::ToSocketAddrs;

    // FIXME: We should ensure a liable port input when starting the worker actually
    // can not start server wo port, panic i.o.
    let ip: &str = &uri.ip();
    let port = uri.port_u16();
    debug!("Connecting to: {}:{:?}", ip, port);
    let addrs = (ip, port).to_socket_addrs().unwrap();
    for addr in addrs {
        if let SocketAddr::V4(_) = addr {
            return addr;
        }
    }
    unreachable!("Cannot lookup address");
}



pub fn enclave_run_openfinex_client(
    eid: sgx_enclave_id_t,
    finex_uri: OpenFinexUri,
) -> SgxResult<()> {
    let addr = get_socket_addr(finex_uri.clone());
    // etablish TCP
    let sock = TcpStream::connect(&addr).expect("[-] Connect to websocket failed!");
    // create new HttpClient within enclave
    if let Some(mut client) = TcpClient::new(eid, sock, finex_uri) {
        println!("[+] Httpclient successfully created within enclave");
        // write_all from io::Write (https://doc.rust-lang.org/std/io/trait.Write.html)
        // Attempts to write an entire buffer into this writer.
        //client.write_all(httpreq.as_bytes()).unwrap();

        // Mio is a fast, low-level I/O library for Rust focusing on non-blocking APIs
        // and event notification for building high performance I/O apps with as
        // little overhead as possible over the OS abstractions.
        // Using Mio starts by creating a Poll, which reads events from the OS and
        // puts them into Events. You can handle I/O events from the OS with it.
        let mut poll = mio::Poll::new()
            .unwrap();
        let mut events = mio::Events::with_capacity(1024);
        // register to mio::Poll
        client.register(&mut poll);

        // Wait for the socket to become ready. This has to happens in a loop to
        // handle spurious wakeups.
        'outer: loop {
            // Poll allows a program to monitor a large number of event::Sources,
            // waiting until one or more become “ready” for some class of operations;
            // e.g. reading and writing
            poll.poll(&mut events, None).unwrap();
            for ev in events.iter() {
                if !client.ready(&mut poll, &ev) {
                    client.close();
                    break 'outer ;
                }
            }

        }
    } else {
        println!("[-] TcpClient could not be created within enclave");
    }

    println!("[+] Tcp Client closed");

    Ok(())
}