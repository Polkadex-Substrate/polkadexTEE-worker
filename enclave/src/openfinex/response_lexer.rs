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
use crate::openfinex::openfinex_types::ResponseInteger;
use alloc::{fmt::Result as FormatResult, string::String, string::ToString, vec::Vec};
use core::iter::Peekable;
use log::*;

#[derive(Debug, Clone, PartialEq)]
pub enum LexItem {
    Paren(char),
    Number(ResponseInteger),
    String(String),
    Json(String),
}

impl alloc::fmt::Display for LexItem {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

pub struct ResponseLexer {}

impl ResponseLexer {
    pub fn lex(&self, input: &str) -> Result<Vec<LexItem>, String> {
        let mut result = Vec::new();
        let mut it = input.chars().peekable();
        while let Some(&c) = it.peek() {
            match c {
                '0'..='9' => {
                    it.next();
                    let n = get_number(c, &mut it)?;
                    result.push(LexItem::Number(n));
                }
                '[' | ']' => {
                    result.push(LexItem::Paren(c));
                    it.next();
                }
                '{' => {
                    let json_string = get_json(&mut it)?;
                    result.push(LexItem::Json(json_string));
                }
                '"' => {
                    it.next();
                    let str = get_string(&mut it)?;
                    result.push(LexItem::String(str));
                }
                ',' => {
                    it.next();
                }
                ' ' => {
                    it.next();
                }
                _ => {
                    return Err(format!("unexpected character {}", c));
                }
            }
        }
        Ok(result)
    }
}

/// lex json content as string
fn get_json<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Result<String, String> {
    let mut json_string = String::new();
    let mut indentation_level = 0u128;

    // match first curly braces
    match iter.next() {
        Some(c) => match c {
            '{' => {
                indentation_level = 1;
                json_string.push(c);
                Ok(())
            }
            _ => Err(format!("Expected '{{' but found {}", c)),
        },
        None => Err("Expected '{' but found end of input string".to_string()),
    }?;

    while let Some(&c) = iter.peek() {
        if indentation_level < 1 {
            break;
        }

        match c {
            '{' => {
                indentation_level += 1;
            }
            '}' => {
                if indentation_level > 0 {
                    indentation_level -= 1;
                } else {
                    return Err("Unexpected closing braces '}'".to_string());
                }
            }
            _ => {}
        }

        json_string.push(c);
        iter.next();
    }

    if indentation_level != 0 {
        return Err("Missing closing braces for JSON string".to_string());
    }

    Ok(json_string)
}

fn get_number<T: Iterator<Item = char>>(
    c: char,
    iter: &mut Peekable<T>,
) -> Result<ResponseInteger, String> {
    let mut number = c
        .to_string()
        .parse::<ResponseInteger>()
        .map_err(|_| format!("The caller should have passed a digit (found {})", c))?;

    while let Some(Ok(digit)) = iter
        .peek()
        .map(|c| c.to_string().parse::<ResponseInteger>())
    {
        number = number * 10 + digit;
        iter.next();
    }
    Ok(number)
}

fn get_string<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> Result<String, String> {
    let mut string = String::new();

    while let Some(c) = iter.peek() {
        if is_string_delimiter(c) {
            iter.next(); // consume string delimiter
            return Ok(string);
        }

        string.push(*c);
        iter.next();
    }

    Err(format!("String ended unexpectedly: {}", string))
}

fn is_string_delimiter(c: &char) -> bool {
    matches!(c, '"' | '\'')
}

pub mod tests {

    use super::*;

    pub fn given_valid_response_string_then_return_lexed_items() {
        let response_string =
            "[2,42,\"admin_create_order\",[\"1245-2345-6798-123123\"]]".to_string();

        let lexer = ResponseLexer {};
        let lexed_items = lexer.lex(&response_string).unwrap();

        assert_eq!(lexed_items.len(), 8);
        assert_eq!(
            lexed_items,
            vec![
                LexItem::Paren('['),
                LexItem::Number(2),
                LexItem::Number(42),
                LexItem::String("admin_create_order".to_string()),
                LexItem::Paren('['),
                LexItem::String("1245-2345-6798-123123".to_string()),
                LexItem::Paren(']'),
                LexItem::Paren(']')
            ]
        );
    }

    pub fn given_valid_delimited_string_then_return_result() {
        verify_valid_string("test_string\"\"");
        verify_valid_string("hello world \" ");
        verify_valid_string("jike@«»12`_=-0\'");
        verify_valid_string("\""); // empty string case
    }

    fn verify_valid_string(input_str: &str) {
        let mut iter = input_str.chars().peekable();
        let get_string_result = get_string(&mut iter);

        assert!(get_string_result.is_ok());
        verify_last_char_is_not_quote(&get_string_result.unwrap())
    }

    fn verify_last_char_is_not_quote(str: &str) {
        if let Some(c) = str.chars().last() {
            assert_ne!(c, '"');
            assert_ne!(c, '\'');
        }
    }

    pub fn given_string_with_missing_delimiter_then_return_error() {
        let test_strings = vec!["test_string", "hello world ", "jike@«»12`_=-0", ""];

        for test_string in test_strings {
            let mut iter = test_string.chars().peekable();
            let get_string_result = get_string(&mut iter);
            assert!(get_string_result.is_err());
        }
    }

    pub fn parse_openfinex_example_json_parameter_correctly() {
        let test_string = (r#"[2,1,"get_markets",[{"id":"btcusd","name":"BTC/USD","base_unit":"btc","quote_unit":"usd","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":100,"filters":[]},{"id":"trsteth","name":"TRST/ETH","base_unit":"trst","quote_unit":"eth","state":"enabled","amount_precision":4,"price_precision":4,"min_price":"0.0001","max_price":"0","min_amount":"0.0001","position":105,"filters":[]}]]"#).to_string();

        let lexer = ResponseLexer {};
        let lexed_items = lexer.lex(&test_string).unwrap();

        assert_eq!(lexed_items.len(), 9);
        verify_json_string(&lexed_items, 5, 215);
        verify_json_string(&lexed_items, 6, 218);
    }

    pub fn parse_json_parameter_mixed_with_regular_parameters() {
        let test_string =
            (r#"[2,1,"get_markets",[{"id":"btcusd","name":"BTC/USD"},"param1","param2",{"id":"jsonid"},{}]]"#).to_string();

        let lexer = ResponseLexer {};
        let lexed_items = lexer.lex(&test_string).unwrap();

        assert_eq!(lexed_items.len(), 12);
        verify_json_string(&lexed_items, 5, 32);
        verify_json_string(&lexed_items, 8, 15);
        verify_json_string(&lexed_items, 9, 2);
    }

    fn verify_json_string(lex_items: &[LexItem], index: usize, expected_length: usize) {
        let json_string = get_json_string(&lex_items, index).unwrap();
        assert_eq!(json_string.len(), expected_length);
        assert_eq!(json_string.chars().next().unwrap(), '{');
        assert_eq!(json_string.chars().last().unwrap(), '}');
    }

    fn get_json_string(lex_items: &[LexItem], index: usize) -> Result<String, ()> {
        match lex_items.get(index) {
            None => Err(()),
            Some(LexItem::Json(s)) => Ok(s.clone()),
            Some(_) => Err(()),
        }
    }

    pub fn given_json_parameter_with_too_many_closing_braces_then_return_error() {
        let test_string =
            (r#"[2,1,"get_markets",[{"id":"btcusd","name":"BTC/USD"}}]]"#).to_string();
        let lexer = ResponseLexer {};
        assert!(lexer.lex(&test_string).is_err());
    }

    pub fn given_json_parameter_with_missing_closing_braces_then_return_error() {
        let test_string =
            (r#"[2,1,"get_markets",[{{"id":"btcusd","name":"BTC/USD"}]]"#).to_string();
        let lexer = ResponseLexer {};
        assert!(lexer.lex(&test_string).is_err());
    }

    pub fn given_valid_number_str_then_lex_correctly() {
        verify_number_parsed("1254", 1254);
        verify_number_parsed("08475", 8475);
        verify_number_parsed("4875]", 4875);
        verify_number_parsed("8475\"]", 8475);
        verify_number_parsed("0", 0);
    }

    fn verify_number_parsed(number_str: &str, number: ResponseInteger) {
        let mut iter = number_str.chars().peekable();
        let first_char = iter.next().unwrap();
        let number_result = get_number(first_char, &mut iter);
        assert!(number_result.is_ok());
        assert_eq!(number_result.unwrap(), number);
    }
}
