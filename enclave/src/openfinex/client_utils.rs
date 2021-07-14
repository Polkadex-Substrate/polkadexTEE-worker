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

use log::*;
use sgx_rand;
use std::string::String;
use std::vec::Vec;

pub use self::Opcode::{BinaryOp, CloseOp, ContinuationOp, PingOp, PongOp, TextOp};
pub use self::Payload::{Binary, Empty, Text};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Payload {
    Text(String),
    Binary(Vec<u8>),
    Empty,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub payload: Payload,
    pub opcode: Opcode,
}

impl Message {
    pub fn new(payload: Payload, opcode: Opcode) -> Self {
        Message { payload, opcode }
    }
}

#[derive(Clone, Debug)]
pub enum Opcode {
    ContinuationOp = 0x0,
    TextOp = 0x1,
    BinaryOp = 0x2,
    CloseOp = 0x8,
    PingOp = 0x9,
    PongOp = 0xA,
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
    byte >> 7 != 0
}

// https://datatracker.ietf.org/doc/html/rfc6455#section-5
pub fn mask(payload: &[u8], opcode: Opcode) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![0b10000000 | opcode as u8]; // fin: 1, rsv: 000

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
        return vec![];
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
