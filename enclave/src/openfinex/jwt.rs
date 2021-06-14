// more info: https://github.com/mesalock-linux/jsonwebtoken-sgx

use std::borrow::ToOwned;
use std::string::String;
use log::*;


use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode as jwtencode, Header, Algorithm, EncodingKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    uid: String,
    email: String,
    role: String,
    level: u8,
    /* aud: String,         // Optional. Audience
    exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize,          // Optional. Issued at (as UTC timestamp)
    iss: String,         // Optional. Issuer
    nbf: usize,          // Optional. Not Before (as UTC timestamp)
    sub: String,         // Optional. Subject (whom token refers to)
    */
}

impl Claims {
    /// create dummy claims for now..
    pub fn new(uid: String) -> Self {
        Claims {
            uid,
            email: "admin@barong.io".to_owned(),
            role: "admin".to_owned(),
            level: 3,
        }
    }
}

pub fn create_jwt(uid: String) -> String {
    let claims = Claims::new(uid);
    match jwtencode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(include_bytes!("../../../bin/jwt/rsa-key")).unwrap() //FIXME: eek, hardcoded
    ) {
        Ok(token) => {
            debug!("successfully created jwt: {}", token);
            token
        },
        Err(e) => {
            error!("Could not create jwt: {:?}", e);
            "".to_owned()
        }
    }
}