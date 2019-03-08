use std::collections::HashMap;

use cash_addr;

use super::AddressType;
use super::AddressFormat;
use super::Network;
use super::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PrefixDetails {
    format: AddressFormat,
    network: Network,
}

pub struct CashConverter {
    prefix_list: HashMap<String, PrefixDetails>,
    prefix_inv_list: HashMap<PrefixDetails, String>,
}

const SEPARATOR: char = ':';

impl CashConverter {
    pub fn new() -> CashConverter {
        let prefix_list = [
            ("bitcoincash".to_string(), PrefixDetails {
                format: AddressFormat::CashAddr,
                network: Network::Mainnet,
            }),
            ("bchtest".to_string(), PrefixDetails {
                format: AddressFormat::CashAddr,
                network: Network::Testnet,
            }),
            ("bchreg".to_string(), PrefixDetails {
                format: AddressFormat::CashAddr,
                network: Network::Regtest,
            }),
        ].iter().cloned().collect::<HashMap<String, PrefixDetails>>();

        CashConverter {
            prefix_inv_list: prefix_list.iter().map(|el| (el.1.clone(), el.0.clone())).collect(),
            prefix_list,
        }
    }

    pub fn add_prefixes(mut self, prefixes: &[(&str, Network)], format_name: &str) -> CashConverter {
        self.prefix_list.extend(prefixes.iter().map(|p| {
            (p.0.to_string(), PrefixDetails {
                format: AddressFormat::Other(format_name.to_string()),
                network: p.1,
            })
        }));
        self.prefix_inv_list.extend(prefixes.iter().map(|p| {
            (PrefixDetails {
                format: AddressFormat::Other(format_name.to_string()),
                network: p.1,
            }, p.0.to_string())
        }));
        self
    }

    pub fn parse(&self, addr: &str) -> Result<(AddressFormat, Network, AddressType, Vec<u8>)> {
        if addr.contains(SEPARATOR) {
            return Ok(self.parse_with_prefix(addr)?)
        }

        for prefix in self.prefix_list.keys() {
            let addr = format!("{}{}{}", prefix, SEPARATOR, addr);
            match self.parse_with_prefix(&addr) {
                Ok(ret) => return Ok(ret),
                Err(_)  => continue,
            }
        }

        Err(Error::InvalidAddress(addr.to_string()))
    }

    fn parse_with_prefix(&self, addr: &str) -> Result<(AddressFormat, Network, AddressType, Vec<u8>)> {
        let (prefix, addr_type, hash) = cash_addr::decode(addr)?;
        let prefix_details = self.prefix_list.get(&prefix).ok_or_else(|| Error::UnknownCashPrefix(prefix))?;

        Ok((prefix_details.format.clone(), prefix_details.network, addr_type, hash))
    }

    pub fn build(&self, format: &AddressFormat, network: Network, addr_type: AddressType, hash: &[u8]) -> Result<String> {
        let prefix = self.prefix_inv_list.get(&PrefixDetails{format: format.clone(), network})
            .ok_or_else(|| Error::UnknownCashFormat(format.clone(), network))?;
        Ok(cash_addr::encode(prefix, addr_type, hash)?)
    }
}
