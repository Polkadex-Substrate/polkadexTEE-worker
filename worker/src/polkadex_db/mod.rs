pub mod general_db;
pub use general_db::*;
pub mod orderbook;
pub use orderbook::*;

#[derive(Debug)]
pub enum PolkadexDBError {
    UnableToLoadPointer,
    UnableToDeseralizeValue,
}
