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
    const PRECISION: usize = 18;

    /// parses a fixed point number (base10) from a string and converts it to
    /// an integer value, shifted by UNIT orders of magnitude. Any digits that are beyond this
    /// precision, will cause the parsing to fail (in order to prevent loss of digits/information)
    ///
    /// Limitations:
    /// - no negative numbers can be parsed
    /// - no scientific notation
    /// - no fractions below 1/UNIT
    pub fn parse_from_string(str: &String) -> OpenFinexApiResult<ResponseInteger> {
        let mut chars_iter = str.chars().peekable();

        // parse all digits before the decimal (integer part)
        let mut integer_digits = parse_number_sequence(&mut chars_iter);
        integer_digits.reverse();

        match chars_iter.peek() {
            Some(c) => {
                if *c == '.' {
                    chars_iter.next();
                }
            }
            None => {}
        }

        let fraction_digits = parse_number_sequence(&mut chars_iter);

        if let Some(_) = chars_iter.peek() {
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
    pub fn to_string(integer: ResponseInteger) -> String {
        let fraction = integer % UNIT;
        let integer_part = integer / UNIT;

        if fraction == 0 {
            return format!("{}.0", integer_part);
        }

        format!("{}.{:0>18}", integer_part, fraction)
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

fn combine_integer_digits(digits: &Vec<u8>) -> OpenFinexApiResult<ResponseInteger> {
    combine_digits(digits, &|order| UNIT.checked_mul(order))
}

fn combine_fraction_digits(digits: &Vec<u8>) -> OpenFinexApiResult<ResponseInteger> {
    combine_digits(digits, &|order| (UNIT / 10u128).checked_div(order))
}

fn combine_digits(
    digits: &Vec<u8>,
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
        assert!(FixedPointNumberConverter::parse_from_string(&format!("ar4")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("34.5g")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("not_a_number")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("12.f")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("a.56")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("a0.21542")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("_")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("-1547566.2894")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("NaN")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("Inf")).is_err());
    }

    pub fn fail_to_parse_scientific_notation() {
        assert!(FixedPointNumberConverter::parse_from_string(&format!("1.4e-3")).is_err());
        assert!(FixedPointNumberConverter::parse_from_string(&format!("9e8")).is_err());
    }

    pub fn fail_to_parse_if_too_large() {
        // number in string is larger than what can be converted to u128 (with 18 digit shift)
        // u128 max 340_282_366_920_938_463_463,,_374_607_431_768_211_455
        assert!(
            FixedPointNumberConverter::parse_from_string(&format!("340282366920938463464"))
                .is_err()
        );
    }

    pub fn successfully_parse_numbers() {
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&format!("42")).unwrap(),
            42_000_000_000_000_000_000
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&format!("0.1234")).unwrap(),
            123_400_000_000_000_000
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&format!(".005")).unwrap(),
            5_000_000_000_000_000
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&format!("6548731654.123456789012345678"))
                .unwrap(),
            6_548_731_654_123_456_789_012_345_678
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&format!("4785182996.201809734")).unwrap(),
            4_785_182_996_201_809_734_000_000_000u128
        );
        assert_eq!(
            FixedPointNumberConverter::parse_from_string(&format!(".1")).unwrap(),
            100_000_000_000_000_000u128
        );
    }

    pub fn fail_to_parse_if_number_exceeds_precision() {
        // have limited precision of 18 digits
        assert!(
            FixedPointNumberConverter::parse_from_string(&format!("0.0000000000000000001"))
                .is_err()
        );
    }

    pub fn convert_to_string() {
        assert_eq!(
            FixedPointNumberConverter::to_string(10_000_000_000_000_000_000u128),
            format!("10.0")
        );

        assert_eq!(
            FixedPointNumberConverter::to_string(42u128),
            format!("0.000000000000000042")
        );

        assert_eq!(
            FixedPointNumberConverter::to_string(487_190_845_002_441_456_031_034_845_001u128),
            format!("487190845002.441456031034845001")
        );

        assert_eq!(
            FixedPointNumberConverter::to_string(1u128),
            format!("0.000000000000000001")
        );

        assert_eq!(FixedPointNumberConverter::to_string(0u128), format!("0.0"));
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
            let number_str = FixedPointNumberConverter::to_string(number);
            let converted_number =
                FixedPointNumberConverter::parse_from_string(&number_str).unwrap();
            assert_eq!(number, converted_number);
        }
    }
}
