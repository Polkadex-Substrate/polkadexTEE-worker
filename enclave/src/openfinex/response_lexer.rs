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
use crate::openfinex::openfinex_types::ResponseInteger;
use alloc::{
    fmt::Result as FormatResult, string::String, string::ToString,
    vec::Vec,
};
use core::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum LexItem {
    Paren(char),
    Number(ResponseInteger),
    String(String),
}

impl alloc::fmt::Display for LexItem {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> FormatResult {
        write!(f, "{:?}", self)
    }
}

pub struct ResponseLexer {}

impl ResponseLexer {
    pub fn lex(&self, input: &String) -> Result<Vec<LexItem>, String> {
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
    match c {
        '"' | '\'' => true,
        _ => false,
    }
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

    pub fn test_given_valid_delimited_string_then_return_result() {
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

    fn verify_last_char_is_not_quote(str: &String) {
        match str.chars().last() {
            Some(c) => {
                assert_ne!(c, '"');
                assert_ne!(c, '\'');
            }
            None => {}
        }
    }

    pub fn test_given_string_with_missing_delimiter_then_return_error() {
        let test_strings = vec!["test_string", "hello world ", "jike@«»12`_=-0", ""];

        for test_string in test_strings {
            let mut iter = test_string.chars().peekable();
            let get_string_result = get_string(&mut iter);
            assert!(get_string_result.is_err());
        }
    }

    pub fn test_given_valid_number_str_then_lex_correctly() {
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
