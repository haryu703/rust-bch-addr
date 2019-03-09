use std::result;

use bs58;
use cash_addr;
use failure::Fail;

use super::{Network, AddressFormat};

/// Alias of `Result` used by bch_addr.
pub type Result<T> = result::Result<T, Error>;

/// Errors
#[derive(Debug, Fail)]
pub enum Error {
    /// Unknow legacy address's prefix (first byte).
    /// # Arguments
    /// * Prefix (1 byte).
    #[fail(display = "unknow legacy prefix: {}", 0)]
    UnknownLegacyPrefix(u8),

    /// Unknow cash_addr address's prefix.
    /// # Arguments
    /// * Prefix.
    #[fail(display = "unknow cash prefix: {}", 0)]
    UnknownCashPrefix(String),

    /// Unknow cash_addr address's format and network.
    /// # Arguments
    /// * address format.
    /// * network.
    #[fail(display = "unknow cash prefix: {:?}, {:?}", 0, 1)]
    UnknownCashFormat(AddressFormat, Network),

    /// Address that can not be converted.
    /// # Arguments
    /// * Address.
    #[fail(display = "invalid address: {}", 0)]
    InvalidAddress(String),

    /// bs58 library's error.
    /// # Arguments
    /// * Error.
    #[fail(display = "bs58 error: {}", 0)]
    Bs58(bs58::decode::DecodeError),

    /// cash_addr library's error.
    /// # Arguments
    /// * Error.
    #[fail(display = "cash addr error: {}", 0)]
    CashAddr(cash_addr::Error),
}

impl From<bs58::decode::DecodeError> for Error {
    fn from(err: bs58::decode::DecodeError) -> Error {
        Error::Bs58(err)
    }
}

impl From<cash_addr::Error> for Error {
    fn from(err: cash_addr::Error) -> Error {
        Error::CashAddr(err)
    }
}
