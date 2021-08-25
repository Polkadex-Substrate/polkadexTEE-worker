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

pub extern crate alloc;

use crate::constants::UNIT;
use crate::openfinex::openfinex_api::{OpenFinexApiError, OpenFinexApiResult};
use crate::openfinex::openfinex_types::ResponseInteger;
use alloc::{string::String, string::ToString, vec::Vec};
use core::iter::Peekable;

/// Provides conversion from and to a string containing a fixed point decimal with 18-digit precision
/// to an u128 integer, scaled by UNIT (=1e18).
pub struct FixedPointNumberConverter {}

impl FixedPointNumberConverter {
    const PRECISION: usize = 4;

    /// parses a fixed point number (base10) from a string and converts it to
    /// an integer value, shifted by UNIT orders of magnitude. Any digits that are beyond this
    /// precision, will cause the parsing to fail (in order to prevent loss of digits/information)
    ///
    /// Limitations:
    /// - no negative numbers can be parsed
    /// - no scientific notation
    /// - no fractions below 1/UNIT
    pub fn parse_from_string(str: &str) -> OpenFinexApiResult<ResponseInteger> {
        let mut chars_iter = str.chars().peekable();

        // parse all digits before the decimal (integer part)
        let mut integer_digits = parse_number_sequence(&mut chars_iter);
        integer_digits.reverse();

        if let Some(c) = chars_iter.peek() {
            if *c == '.' {
                chars_iter.next();
            }
        }

        let fraction_digits = parse_number_sequence(&mut chars_iter);

        if chars_iter.peek().is_some() {
            return Err(OpenFinexApiError::FixedPointConversionError(format!(
                "string ({}) is not a valid fixed point number",
                str
            )));
        }

        if integer_digits.is_empty() && fraction_digits.is_empty() {
            return Err(OpenFinexApiError::FixedPointConversionError(format!(
                "string ({}) does not contain any numbers",
                str
            )));
        }

        if fraction_digits.len() > FixedPointNumberConverter::PRECISION {
            return Err(OpenFinexApiError::FixedPointConversionError(format!(
                "string ({}) contains more fraction digits ({}) than are supported ({}) (guaranteed lossless)",
                str, fraction_digits.len(), FixedPointNumberConverter::PRECISION
            )));
        }

        let integer_part = combine_integer_digits(&integer_digits)?;
        let fraction_part = combine_fraction_digits(&fraction_digits)?;

        Ok(integer_part + fraction_part)
    }

    /// convert an integer to a fixed point number string, shifted by UNIT orders of magnitude
    pub fn _to_string(integer: ResponseInteger) -> String {
        //TODO: Find a function name that doesn't trigger clippy
        let fraction = integer % UNIT;
        let integer_part = integer / UNIT;

        if fraction == 0 {
            return format!("{}.0", integer_part);
        }

        format!("{}.{:0>4}", integer_part, fraction)
    }
}

fn parse_number_sequence<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Vec<u8> {
    let mut digits: Vec<u8> = Vec::with_capacity(60);
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<u8>()) {
        digits.push(digit);
        iter.next();
    }
    digits
}

fn combine_integer_digits(digits: &[u8]) -> OpenFinexApiResult<ResponseInteger> {
    combine_digits(digits, &|order| UNIT.checked_mul(order))
}

fn combine_fraction_digits(digits: &[u8]) -> OpenFinexApiResult<ResponseInteger> {
    combine_digits(digits, &|order| (UNIT / 10u128).checked_div(order))
}

fn combine_digits(
    digits: &[u8],
    scale_fn: &dyn Fn(u128) -> Option<u128>,
) -> OpenFinexApiResult<ResponseInteger> {
    let mut number: u128 = 0;
    let mut order: u128 = 1;

    for digit in digits {
        // all operations are performed in 'checked' mode to prevent overflow errors

        let scale = (scale_fn)(order).ok_or_else(|| {
            OpenFinexApiError::FixedPointConversionError("Value overflow".to_string())
        })?;

        let digit_value = scale.checked_mul(*digit as u128).ok_or_else(|| {
            OpenFinexApiError::FixedPointConversionError("Value overflow".to_string())
        })?;

        number = number.checked_add(digit_value).ok_or_else(|| {
            OpenFinexApiError::FixedPointConversionError("Value overflow".to_string())
        })?;

        order = order.checked_mul(10u128).ok_or_else(|| {
            OpenFinexApiError::FixedPointConversionError("Value overflow".to_string())
        })?;
    }

    Ok(number)
}

pub mod tests {

    use super::*;

    pub fn fail_to_parse_invalid_strings() {
        assert!(FixedPointNumberConverter::parse_from_string(&"ar4".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"34.5g".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"not_a_number".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"12.f".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"a.56".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"a0.21542".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"_".to_string()).is_err());
        assert!(
            FixedPointNumberConverter::parse_from_string(&"-1547566.2894".to_string()).is_err()
        );
        assert!(FixedPointNumberConverter::parse_from_string(&"NaN".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"Inf".to_string()).is_err());
    }

    pub fn fail_to_parse_scientific_notation() {
        assert!(FixedPointNumberConverter::parse_from_string(&"1.4e-3".to_string()).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&"9e8".to_string()).is_err());
    }

    pub fn fail_to_parse_if_too_large() {
        // number in string is larger than what can be converted to u128 (with 18 digit shift)
        // u128 max 340_282_366_920_938_463_463,,_374_607_431_768_211_455
        assert!(
            FixedPointNumberConverter::parse_from_string(&"340282366920938463464".to_string())
                .is_err()
        );
    }

    pub fn successfully_parse_numbers() {
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&"42".to_string()).unwrap(),
            42_000
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&"0.1234".to_string()).unwrap(),
            123_40
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&".005".to_string()).unwrap(),
            5_000_0
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(
                &"6548731654.123456789012345678".to_string()
            )
            .unwrap(),
            6_548_731_654_123_456_789_012_345_678
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&"4785182996.201809734".to_string())
                .unwrap(),
            4_785_182_996_201_809_734_000_000_000u128
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&".1".to_string()).unwrap(),
            100_000_000_000_000_000u128
        );
    }

    pub fn fail_to_parse_if_number_exceeds_precision() {
        // have limited precision of 18 digits
        assert!(
            FixedPointNumberConverter::parse_from_string(&"0.0000000000000000001".to_string())
                .is_err()
        );
    }

    pub fn convert_to_string() {
        assert_eq!(
            FixedPointNumberConverter::_to_string(10_000_000_000_000_000_000u128),
            "10.0".to_string()
        );

        assert_eq!(
            FixedPointNumberConverter::_to_string(42u128),
            "0.0042".to_string()
        );

        assert_eq!(
            FixedPointNumberConverter::_to_string(487_190_845_002_441_456_031_034_845_001u128),
            "487190845002.441456031034845001".to_string()
        );

        assert_eq!(
            FixedPointNumberConverter::_to_string(1u128),
            "0.000000000000000001".to_string()
        );

        assert_eq!(
            FixedPointNumberConverter::_to_string(0u128),
            "0.0".to_string()
        );
    }

    pub fn convert_to_string_and_back() {
        let numbers = vec![
            5648944u128,
            482u128,
            0u128,
            98714587614u128,
            7_000_000_000_000_000_000_000_000_000_000u128,
            u128::MAX,
        ];

        for number in numbers {
            let number_str = FixedPointNumberConverter::_to_string(number);
            let converted_number =
                FixedPointNumberConverter::parse_from_string(&number_str).unwrap();
            assert_eq!(number, converted_number);
        }
    }
}
