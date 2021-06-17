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

mod client_utils;
mod jwt;
mod market_repo;
mod response_object_mapper;
mod response_parser;

// most of these could be private, but because the tests are inside the same modules
// and are not discovered using 'cargo test', they have to be public
// in some cases we decided to have the tests in a separate file, those modules can be kept private
pub mod fixed_point_number_converter;
pub mod market;
pub mod openfinex_api;
pub mod openfinex_api_impl;
pub mod openfinex_client;
pub mod openfinex_types;
pub mod request_builder;
pub mod request_id_generator;
pub mod response_handler;
pub mod response_lexer;
pub mod string_serialization;

pub mod tests;
