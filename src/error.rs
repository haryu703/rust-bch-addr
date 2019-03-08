use std::result;

use bs58;
use cash_addr;
use failure::Fail;

use super::{Network, AddressFormat, AddressType};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "unknow legacy prefix: {}", 0)]
    UnknownLegacyPrefix(u8),

    #[fail(display = "unknow cash prefix: {:?}, {:?}", 0, 1)]
    UnknownLegacyFormat(AddressType, Network),

    #[fail(display = "unknow cash prefix: {}", 0)]
    UnknownCashPrefix(String),

    #[fail(display = "unknow cash prefix: {:?}, {:?}", 0, 1)]
    UnknownCashFormat(AddressFormat, Network),

    #[fail(display = "invalid address: {}", 0)]
    InvalidAddress(String),

    #[fail(display = "bs58 error: {}", 0)]
    Bs58(bs58::decode::DecodeError),

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
