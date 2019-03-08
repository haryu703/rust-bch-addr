use super::AddressType;
use super::AddressFormat;
use super::Network;
use super::error::{Error, Result};

use bs58;

pub fn parse(addr: &str) -> Result<(AddressFormat, Network, AddressType, Vec<u8>)> {
    let data = bs58::decode(addr).with_check(None).into_vec()?;
    let (network, addr_type) = match data[0] {
        0x00 => Ok((Network::Mainnet, AddressType::P2PKH)),
        0x05 => Ok((Network::Mainnet, AddressType::P2SH)),
        0x6f => Ok((Network::Testnet, AddressType::P2PKH)),
        0xc4 => Ok((Network::Testnet, AddressType::P2SH)),
        e    => Err(Error::UnknownLegacyPrefix(e)),
    }?;
    let data = &data[1..];

    Ok((AddressFormat::Legacy, network, addr_type, data.to_vec()))
}

pub fn build(network: Network, addr_type: AddressType, hash: &[u8]) -> Result<String> {
    let prefix = match (network, addr_type) {
        (Network::Mainnet, AddressType::P2PKH) => 0x00,
        (Network::Mainnet, AddressType::P2SH)  => 0x05,
        (Network::Testnet, AddressType::P2PKH) => 0x6f,
        (Network::Testnet, AddressType::P2SH)  => 0xc4,
        (Network::Regtest, AddressType::P2PKH) => 0x6f,
        (Network::Regtest, AddressType::P2SH)  => 0xc4,
    };
    let hash = [&[prefix], &hash[..]].concat();
    Ok(bs58::encode(hash).with_check().into_string())
}
