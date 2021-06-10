// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü.
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
use crate::openfinex::openfinex_api::OpenFinexApiError;
use crate::openfinex::openfinex_types::OpenFinexDecimal;
use alloc::string::String;
use polkadex_sgx_primitives::types::PriceAndQuantityType;

const CONVERSION_FACTOR: f64 = UNIT as f64;

/// converts an OpenFinex floating point value into a block chain price/amount value (integer)
pub fn convert_to_integer(
    decimal: OpenFinexDecimal,
) -> Result<PriceAndQuantityType, OpenFinexApiError> {
    // block chains don't support floating point numbers
    // (this is required to ensure the deterministic nature of the runtime)
    // so we have to convert block chain quantities to OpenFinex decimals by dividing by a factor of 10^18
    if decimal < 0.0f64 {
        return Err(OpenFinexApiError::FloatingPointConversionError(format!(
            "attempting to convert a negative floating point number ({}) into an unsigned integer",
            decimal
        )));
    }

    Ok(OpenFinexDecimal::trunc(decimal * CONVERSION_FACTOR) as PriceAndQuantityType)
}

/// converts a block chain price/amount value (integer) into an OpenFinex floating point value
pub fn convert_to_decimal(integer: PriceAndQuantityType) -> OpenFinexDecimal {
    let value_as_float = integer as f64;
    value_as_float / CONVERSION_FACTOR
}

pub mod tests {

    use super::*;

    pub fn given_negative_floating_point_number_then_conversion_fails() {
        assert!(convert_to_integer(-23.9f64).is_err());
        assert!(convert_to_integer(-1e-92f64).is_err());
        assert!(convert_to_integer(-1.0f64).is_err());
        assert!(convert_to_integer(-OpenFinexDecimal::INFINITY).is_err());
    }

    pub fn given_positive_floating_point_number_then_convert_successfully() {
        assert_eq!(
            convert_to_integer(23.719348f64).unwrap(),
            23_719_348_000_000_000_000
        );
        assert_eq!(
            convert_to_integer(1.0f64).unwrap(),
            1_000_000_000_000_000_000
        );
    }

    pub fn given_integer_value_then_convert_with_factor() {
        assert!(floats_are_approximately_equal(
            convert_to_decimal(1),
            1e-18f64
        ));

        assert!(floats_are_approximately_equal(
            convert_to_decimal(9_851_674_235),
            0.000_000_009_851_674_235f64
        ));
    }

    fn floats_are_approximately_equal(val1: f64, val2: f64) -> bool {
        (val1 - val2).abs() < f64::EPSILON
    }
}
