/* Copyright (c) 2014 Ehsanul Hoque
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:
The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE. */

use std::vec::Vec;
use std::string::String;
use log::*;
use core::convert::TryInto;
use sgx_rand;

pub use self::Payload::{Text, Binary, Empty};
pub use self::Opcode::{ContinuationOp, TextOp, BinaryOp, CloseOp, PingOp, PongOp};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Payload {
    Text(String),
    Binary(Vec<u8>),
    Empty
}

#[derive(Clone, Debug)]
pub struct Message {
    pub payload: Payload,
    pub opcode: Opcode,
}

impl Message {
    pub fn new(payload: Payload, opcode: Opcode) -> Self {
        Message {payload, opcode}
    }
}


#[derive(Clone, Debug)]
pub enum Opcode {
    ContinuationOp = 0x0,
    TextOp         = 0x1,
    BinaryOp       = 0x2,
    CloseOp        = 0x8,
    PingOp         = 0x9,
    PongOp         = 0xA,
}

impl From<u8> for Opcode {
    fn from(item: u8) -> Self {
        match item {
            0 => ContinuationOp,
            1 => TextOp,
            2 => BinaryOp,
            8 => CloseOp,
            9 => PingOp,
            10 => PongOp,
            _ => unimplemented!(),
        }
    }
}

/// returns true if last msg, false is more msgs follow
/// = %x0 ; more frames of this message follow
/// %x1 ; final frame of this message
pub fn last_frame(byte: u8) -> bool {
    if byte >> 7 == 0 {
        false
    } else {
        true
    }
}

pub fn read_tcp_buffer(buffer: Vec<u8>) -> Option<Message> {
    // https://stackoverflow.com/questions/41115870/is-binary-opcode-encoding-and-decoding-implementation-specific-in-websockets
    //let fin = buf1[0] >> 7; // TODO check this, required for handling fragmented messages
    let rsv = (buffer[0] >> 4) & 0b0111;
    if rsv != 0 {
        return None;
    }

    //let opcode = buffer[0] & 0b0000_1111;
    let opcode: Opcode = (buffer[0] & 0b0000_1111).into();

    //let mask    = buf1[1] & 0b1000_0000; TODO use this to determine whether to unmask or not
    // Take the 2nd byte and read every bit except the Most significant bit
    let pay_len = buffer[1] & 0b0111_1111;

    let (payload_length, payload_buf) = match pay_len {
        127 => {
            // Your length is a uint64 of byte 3 to 8
            error!("buffer is too small for this long message..");
            let slice: [u8; 8] = buffer[2 .. 8].to_vec().try_into().unwrap();
            let length = u64::from_be_bytes(slice);
            (length, buffer[8 .. (length+8) as usize].to_vec())
        },
        126 => {
            // Your length is an uint16 of byte 3 and 4
            let slice: [u8; 2] = buffer[2 .. 4].to_vec().try_into().unwrap();
            let length = u16::from_be_bytes(slice);
            (length as u64, buffer[4 .. (length+4) as usize].to_vec())
        }
        // Byte is 125 or less thats your length
        _   => (pay_len as u64, buffer[2 .. (pay_len+2) as usize].to_vec())
    };
    debug!("payload_length: {}", payload_length);

    // payloads larger than 125 bytes are not allowed for control frames
    match opcode {
        CloseOp | PingOp if payload_length > 125 => panic!(),
        _ => ()
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
        TextOp   => Payload::Text(String::from_utf8(payload_buf.to_vec()).unwrap()),
        BinaryOp => Payload::Binary(payload_buf.to_vec()),
        CloseOp  => Payload::Empty,
        PingOp   => {
            debug!("Ping");
            Payload::Binary(payload_buf.to_vec())

        },
        PongOp   => {
            debug!("Pong");
            Payload::Binary(payload_buf.to_vec())
        },
        _        => {
            Payload::Text(String::from_utf8(payload_buf.to_vec()).unwrap())
        }, // ContinuationOp
    };

    // for now only take text option
    return Some(Message::new(payload, opcode))
}


// https://datatracker.ietf.org/doc/html/rfc6455#section-5
pub fn mask(payload: &[u8], opcode: Opcode) -> Vec<u8> {
    let mut vec = Vec::<u8>::new();
    vec.push(0b10000000 | opcode as u8); // fin: 1, rsv: 000

    let masking_bit = 0b10000000;
    let payload_len = payload.len();
    if payload_len <= 125 {
        vec.push((masking_bit | payload_len) as u8);
    } else if payload_len <= 65535 {
        vec.push((masking_bit | 0b01111110) as u8);
        let mut length = (payload_len as u16).to_be_bytes().to_vec();
        vec.append(&mut length);
    } else if payload_len as u64 <= u64::MAX {
        vec.push((masking_bit | 0b01111111) as u8);
        let mut length = (payload_len as u64).to_be_bytes().to_vec();
        vec.append(&mut length);
    } else {
        error!("too long payload");
        return vec![]
    }
    let mask_key = gen_mask();
    let mut masked_payload = mask_data(mask_key, payload);
    vec.append(&mut mask_key.to_vec());
    debug!("bytes: {:?}", vec);
    vec.append(&mut masked_payload);
    debug!("length: {:?}", vec.len());
    vec

}



/// Generates a random masking key
pub fn gen_mask() -> [u8; 4] {
	sgx_rand::random()
}

/// Masks data to send to a server and writes
pub fn mask_data(mask: [u8; 4], data: &[u8]) -> Vec<u8> {
	let mut out = Vec::with_capacity(data.len());
	let zip_iter = data.iter().zip(mask.iter().cycle());
	for (&buf_item, &key_item) in zip_iter {
		out.push(buf_item ^ key_item);
	}
	out
}

/*# [cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use test;
	#[test]
	fn test_mask_data() {
		let key = [1u8, 2u8, 3u8, 4u8];
		let original = vec![10u8, 11u8, 12u8, 13u8, 14u8, 15u8, 16u8, 17u8];
		let expected = vec![11u8, 9u8, 15u8, 9u8, 15u8, 13u8, 19u8, 21u8];
		let obtained = mask_data(key, &original[..]);
		let reversed = mask_data(key, &obtained[..]);
		assert_eq!(original, reversed);
		assert_eq!(obtained, expected);
	}
	#[bench]
	fn bench_mask_data(b: &mut test::Bencher) {
		let buffer = b"The quick brown fox jumps over the lazy dog";
		let key = gen_mask();
		b.iter(|| {
			let mut output = mask_data(key, buffer);
			test::black_box(&mut output);
		});
	}
	#[bench]
	fn bench_gen_mask(b: &mut test::Bencher) {
		b.iter(|| {
			let mut key = gen_mask();
			test::black_box(&mut key);
		});
	}
} */