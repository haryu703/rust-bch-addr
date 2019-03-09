#![warn(missing_docs)]

//! cash_addr format implementation inspired by bchaddrjs.

mod error;
mod cash_converter;
mod legacy_converter;

pub use cash_addr::AddressType as AddressType;
pub use error::{Error, Result};
use cash_converter::CashConverter;

/// Type of bitcoin netowrk
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Network {
    /// mainnet
    Mainnet,
    /// testnet
    Testnet,
    /// regtest
    Regtest,
}

/// Type of address format
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AddressFormat {
    /// Legacy format.
    /// Same as bitcoin core address.
    Legacy,
    /// cash_addr format
    /// spec: https://github.com/bitcoincashorg/bitcoincash.org/blob/master/spec/cashaddr.md
    CashAddr,
    /// other user-defiend format like cash_addr format
    /// e.g.) slp addr for simpleledger protocol
    ///     https://github.com/simpleledger/slp-specifications/blob/master/slp-token-type-1.md#slp-addr
    /// # Arguments
    /// * `String` - format name
    /// 
    /// # Exapmle
    /// ```
    /// # use bch_addr::AddressFormat;
    /// let format = AddressFormat::Other("SLPAddr".to_string());
    /// ```
    Other(String),
}

/// Address converter.
pub struct Converter {
    cash_converter: CashConverter,
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

impl Converter {
    /// Construct `Converter`.
    /// # Returns
    /// * Object for address conversion.
    /// # Example
    /// ```
    /// # use bch_addr::Converter;
    /// let converter = Converter::new();
    /// ```
    pub fn new() -> Converter {
        Converter {
            cash_converter: CashConverter::new()
        }
    }

    /// Add user-defined address prefix.
    /// By calling this function, you can use other address formats.
    /// # Arguments
    /// * `prefixes` - Slice of tuple of prefix and `Netorok`.
    /// * `format_name` - Format name you want to add.
    /// # Returns
    /// * Object for address conversion.
    /// # Example
    /// ```
    /// # use bch_addr::{Converter, Network};
    /// let converter = Converter::new().add_prefixes(
    ///     &[("simpleledger", Network::Mainnet), ("slptest", Network::Testnet)],
    ///     "SLPAddr",
    /// );
    /// ```
    pub fn add_prefixes(mut self, prefixes: &[(&str, Network)], format_name: &str) -> Converter {
        self.cash_converter = self.cash_converter.add_prefixes(prefixes, format_name);
        self
    }

    /// Convert to cash_addr format with some options.
    /// # Arguments
    /// * `legacy` - Address to be converted. Usually legacy format but cash_addr format is acceptable.
    /// * `format` - (option) Address format. `AddressFormat::CashAddr` or `AddressFormat::Other("other format")` is required.
    /// * `network` - (option) Address network.
    /// # Returns
    /// * Converted address.
    /// # Example
    /// ```
    /// # use bch_addr::{Converter, Network, AddressFormat};
    /// # let converter = Converter::new().add_prefixes(
    /// #     &[("simpleledger", Network::Mainnet), ("slptest", Network::Testnet)],
    /// #     "SLPAddr",
    /// # );
    /// let regtest_addr = converter.to_cash_addr_with_options(
    ///     "mqfRfwGeZnFwfFE7KWJjyg6Yx212iGi6Fi",
    ///     None,
    ///     Some(Network::Regtest)
    /// ).unwrap();
    /// assert_eq!(regtest_addr, "bchreg:qph5kuz78czq00e3t85ugpgd7xmer5kr7c28g5v92v");
    /// 
    /// let slp_addr = converter.to_cash_addr_with_options(
    ///     "1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR",
    ///     Some(AddressFormat::Other("SLPAddr".to_string())),
    ///     None
    /// ).unwrap();
    /// assert_eq!(slp_addr, "simpleledger:qph5kuz78czq00e3t85ugpgd7xmer5kr7ccj3fcpsg");
    /// ```
    pub fn to_cash_addr_with_options(&self, legacy: &str, format: Option<AddressFormat>, network: Option<Network>) -> Result<String> {
        let format = format.unwrap_or(AddressFormat::CashAddr);

        if let Ok((_, current_network, addr_type, hash)) = legacy_converter::parse(legacy) {
            let network = network.unwrap_or(current_network);
            return Ok(self.cash_converter.build(&format, network, addr_type, &hash)?);
        }

        // actually `legacy` may be cash_addr
        if let Ok(current_format) = self.detect_addr_format(legacy) {
            if format == current_format {
                return Ok(legacy.to_string());
            } else {
                let (_, current_network, addr_type, hash) = self.cash_converter.parse(legacy)?;
                let network = network.unwrap_or(current_network);
                return Ok(self.cash_converter.build(&format, network, addr_type, &hash)?);
            }
        }

        Err(Error::InvalidAddress(legacy.to_string()))
    }

    /// Convert to cash_addr format.
    /// # Arguments
    /// * `legacy` - Address to be converted. Usually legacy format but cash_addr format is acceptable.
    /// # Returns
    /// * Converted address.
    /// # Example
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let cash_addr = converter.to_cash_addr("1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR").unwrap();
    /// assert_eq!(cash_addr, "bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk");
    /// ```
    pub fn to_cash_addr(&self, legacy: &str) -> Result<String> {
        self.to_cash_addr_with_options(legacy, None, None)
    }

    /// Convert to legacy format.
    /// # Arguments
    /// * `cash` - Address to be converted. Usually cash_addr format but legacy format is acceptable.
    /// # Returns
    /// * Converted address.
    /// # Example
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let cash_addr = converter.to_legacy_addr("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk").unwrap();
    /// assert_eq!(cash_addr, "1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR");
    /// ```
    pub fn to_legacy_addr(&self, cash: &str) -> Result<String> {
        if let Ok((_, network, addr_type, hash)) = self.cash_converter.parse(cash) {
            return Ok(legacy_converter::build(network, addr_type, &hash)?);
        }

        if self.is_legacy_addr(cash) {
            // actually `cash` is legacy_addr
            return Ok(cash.to_string());
        }

        Err(Error::InvalidAddress(cash.to_string()))
    }

    /// Parse address.
    /// # Arguments
    /// * `addr` - Address to be parsed.
    /// # Returns
    /// * Address format.
    /// * Address network.
    /// * Address type.
    /// * hashed pubilckey.
    /// # Example
    /// ```
    /// # use bch_addr::{Converter, AddressFormat, Network, AddressType};
    /// # let converter = Converter::new();
    /// let (format, network, addr_type, hash) = converter.parse("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk").unwrap();
    /// assert_eq!(format, AddressFormat::CashAddr);
    /// assert_eq!(network, Network::Mainnet);
    /// assert_eq!(addr_type, AddressType::P2PKH);
    /// assert_eq!(hash.len(), 20);
    /// ```
    pub fn parse(&self, addr: &str) -> Result<(AddressFormat, Network, AddressType, Vec<u8>)> {
        legacy_converter::parse(addr)
        .or_else(|_| self.cash_converter.parse(addr))
        .or_else(|_| Err(Error::InvalidAddress(addr.to_string())))
    }

    /// Detect address format.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * Address format.
    /// ```
    /// # use bch_addr::{Converter, AddressFormat};
    /// # let converter = Converter::new();
    /// let format = converter.detect_addr_format("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk").unwrap();
    /// assert_eq!(format, AddressFormat::CashAddr);
    /// ```
    pub fn detect_addr_format(&self, addr: &str) -> Result<AddressFormat> {
        let (format, _, _, _) = self.parse(addr)?;
        Ok(format)
    }

    /// Return `true` if the given address is in cash_addr format.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in cash_addr format, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_cash = converter.is_cash_addr("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk");
    /// assert_eq!(is_cash, true);
    /// ```
    pub fn is_cash_addr(&self, addr: &str) -> bool {
        self.cash_converter.parse(addr).is_ok()
    }

    /// Return `true` if the given address is in legacy format.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in legacy format, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_legacy = converter.is_legacy_addr("1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR");
    /// assert_eq!(is_legacy, true);
    /// ```
    pub fn is_legacy_addr(&self, addr: &str) -> bool {
        legacy_converter::parse(addr).is_ok()
    }

    /// Detect address network.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * Address network.
    /// ```
    /// # use bch_addr::{Converter, Network};
    /// # let converter = Converter::new();
    /// let network = converter.detect_addr_network("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk").unwrap();
    /// assert_eq!(network, Network::Mainnet);
    /// ```
    pub fn detect_addr_network(&self, addr: &str) -> Result<Network> {
        let (_, network, _, _) = self.parse(addr)?;
        Ok(network)
    }

    /// Return `true` if the given address is in mainnet address.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in mainnet address, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_mainnet = converter.is_mainnet_addr("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk");
    /// assert_eq!(is_mainnet, true);
    /// ```
    pub fn is_mainnet_addr(&self, addr: &str) -> bool {
        match self.detect_addr_network(addr) {
            Ok(network) => network == Network::Mainnet,
            Err(_)      => false,
        }
    }

    /// Return `true` if the given address is in testnet address.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in testnet address, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_testnet = converter.is_testnet_addr("mqfRfwGeZnFwfFE7KWJjyg6Yx212iGi6Fi");
    /// assert_eq!(is_testnet, true);
    /// ```
    pub fn is_testnet_addr(&self, addr: &str) -> bool {
        match self.detect_addr_network(addr) {
            Ok(network) => network == Network::Testnet,
            Err(_)      => false,
        }
    }

    /// Return `true` if the given address is in regtest address.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in regtest address, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_regtest = converter.is_regtest_addr("bchreg:qph5kuz78czq00e3t85ugpgd7xmer5kr7c28g5v92v");
    /// assert_eq!(is_regtest, true);
    /// ```
    pub fn is_regtest_addr(&self, addr: &str) -> bool {
        match self.detect_addr_network(addr) {
            Ok(network) => network == Network::Regtest,
            Err(_)      => false,
        }
    }

    /// Detect address type.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * Address type.
    /// ```
    /// # use bch_addr::{Converter, AddressType};
    /// # let converter = Converter::new();
    /// let addr_type = converter.detect_addr_type("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk").unwrap();
    /// assert_eq!(addr_type, AddressType::P2PKH);
    /// ```
    pub fn detect_addr_type(&self, addr: &str) -> Result<AddressType> {
        let (_, _, addr_type, _) = self.parse(addr)?;
        Ok(addr_type)
    }

    /// Return `true` if the given address is in P2PKH address.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in P2PKH address, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_p2pkh = converter.is_p2pkh_addr("bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk");
    /// assert_eq!(is_p2pkh, true);
    /// ```
    pub fn is_p2pkh_addr(&self, addr: &str) -> bool {
        match self.detect_addr_type(addr) {
            Ok(format) => format == AddressType::P2PKH,
            Err(_)     => false,
        }
    }

    /// Return `true` if the given address is in P2SH address.
    /// # Arguments
    /// * `addr` - Address in any format.
    /// # Returns
    /// * `true` if the given address is in P2SH address, `false` otherwise.
    /// ```
    /// # use bch_addr::Converter;
    /// # let converter = Converter::new();
    /// let is_p2sh = converter.is_p2sh_addr("3BqVJRg7Jf94yJSvj2zxaPFAEYh3MAyyw9");
    /// assert_eq!(is_p2sh, true);
    /// ```
    pub fn is_p2sh_addr(&self, addr: &str) -> bool {
        match self.detect_addr_type(addr) {
            Ok(format) => format == AddressType::P2SH,
            Err(_)     => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://github.com/bitcoincashjs/bchaddrjs/blob/master/test/bchaddr.js
    static LEGACY_MAINNET_P2PKH_ADDRESSES: [&str; 20] = [
        "1B9UNtBfkkpgt8kVbwLN9ktE62QKnMbDzR",
        "185K5yAfcrARrHjNVt4iAUHtkYqcogF4km",
        "1EUrmffDt4SQQkGVfmDTyFcp57PuByeadW",
        "1H6YWsFBxvDx6Ce9dyUFZvjG29npxQpBpR",
        "15z9kQvBaZmTGRTRbP3K1VBM3BQvRsj4U4",
        "1P238gziZdeS5Wj9nqLhQHSBK2Lz6zPSke",
        "13WamBttqMB9AHNovKBCeLFGC5sbN4iZkh",
        "17Sa1fdVXh2NVgcn5xoWzTLGNivg9gUDQ7",
        "1tQ2P2q5cVERY8AkGD4K8RGc6NmZQVTKN",
        "1FJSGaq7Wip2ADSJboxMXniPhnYM8ym5Ri",
        "1GxjvJnjF6t29gDnX4jF3u25u5JRqANYPV",
        "1N7gqB2GtgJG8ap3uwRoKyrcrrSTa4qfXu",
        "1JG6fXqEiu9H2fktGxqpFfGGLdy6ie7QgY",
        "14ipzRgYAbSZUnmeRNhhrPMQ8XQrzGg4wo",
        "185FScTRCtVXRoy5gSDbuLnnQaQWqCK4A1",
        "1NPRQpCNaeVvZLYw6Z3Y1XkKxLt9BrFTn5",
        "1Pa8bRApFwCZ8rkgCJh9mfUmj4XJMUYdom",
        "13HmTnwyKacGJCt2WseTReCeEAtG5ZAyci",
        "1Mdob5JY1yuwoj6y76Vf3AQpoqUH5Aft8z",
        "1D8zGeRj3Vkns6VwKxwNoW2mDsxF25w2Zy",
    ];

    static LEGACY_MAINNET_P2SH_ADDRESSES: [&str; 20] = [
        "3BqVJRg7Jf94yJSvj2zxaPFAEYh3MAyyw9",
        "38mL1Wf7AkUowTRocyjJb6epu58LSafEYf",
        "3FAshD9fRxknVuxvnrt4PsykDdgckmK7xD",
        "3HnZSQjdWpYLBNLam58qzZ6CAg5YXBddBW",
        "36gAfxQd8U5qMb9riUhuS7YHBhhdvjr8u1",
        "3Pi44EVA7XxpAgRauw1Hpuo7TYdhd7WMon",
        "34CbgjPLPFVXFT5F3Qqo4xcCLcAJwvkM85",
        "388awD7w5bLkarKDD4U7R5hCXFDPmHuWW7",
        "32aQwvXGdWocWhpbsMsejknCkcfVB4ivTM",
        "3FzTC8KZ4d8QFP8jiucwxR5KrJq4bcevn7",
        "3HekqrHAo1CQEqvDeAPqUXP23bb9Sf9WoA",
        "3NohkiWiSaceDkWV336PkcDZ1NjBBWBewT",
        "3Jx7b5KgGoTf7qTKQ4WQgHdCVAFpCKiqsB",
        "35QquyAyiVkwZxU5YUNJH1iLH3haZ5TEfC",
        "38mGN9wrknouWyfWoXtCKy9iZ6hEMRGsyp",
        "3P5SLMgp8YpJeWFNDei8SA7G6sArkNKQKL",
        "3QG9WxfFoqWwE2T7KQMkCHqhsap1waSfDu",
        "33ynPLSQsUvePNaTdyK3rGZaNhAyfeAmbT",
        "3NKpWcnyZtEKttoQECAFTnmkxMkzgbT4WX",
        "3Dq1CBvAbQ5AxGCNT4byE8PhNQExZcR6Q2",
    ];

    static LEGACY_TESTNET_P2PKH_ADDRESSES: [&str; 20] = [
        "mqfRfwGeZnFwfFE7KWJjyg6Yx212iGi6Fi",
        "mnbGP2FeRsbgdQCzDT35zPWDcYSKm4wrcg",
        "mtzp4ikCh5sfBrk7PLBqoAq8w6zc48PsGn",
        "mwcVovLAmwfCsK7mMYSdPqwat9PXqcMiFt",
        "mkW73U1APbCi3Xw3Jx1gqQPfuB1dHFDiEU",
        "n3XzRk5hNf5grdCmWQK5ECeWB1wgzzYzZd",
        "mi2Y4EyseNcPwPrRdt9aUFTb45UJHNgtbL",
        "mmxXJiiULiTdGo6PoXmtpNYbEiXP2v746S",
        "mgQMKS7otdvVCebnTqBS93dbU5yUZPsANB",
        "mupPZdv6KkFGwKuvKNvjMhviZn93yznq73",
        "mwUhDMsi48KGvnhQEdhcspEQm4u8o754bx",
        "n2de8E7FhhjWuhHfdWQB9u4wir3AXqspCt",
        "mxn3xavDXvaXonEVzXpC5aUbCdZoaTEB2g",
        "mjEnHUmWycspFuFG8wg5gJZizX1ZtEF1XN",
        "mnbCjfYQ1uvnCvShQ1ByjG17Ga1Dk3RTXN",
        "n2uNhsHMPfwBLT2Yp81uqSxepLUr6zCnCz",
        "n465tUFo4xdouyEHusfXbah6b481K5Nivk",
        "mhoikr2x8c3X5KMeEScqFZQy6AUy4GeR4M",
        "n29kt8PWq1MCaqaapfU2s5d9fq4yytS1xJ",
        "msewZhWhrXC3eCyZ3XukdRF65sYwtbmARy",
    ];

    static LEGACY_TESTNET_P2SH_ADDRESSES: [&str; 20] = [
        "2N3PhNAc8v7eRB65UQAcqCLERStuD93JXLD",
        "2MzKY5Fb8nCzA9F4MJ7MBD3e67RLWFE1ciP",
        "2N6j5kx5h3RG8hhbUTzVw1py1RytnZNYoXo",
        "2N9LmW9ff8H3gP9y8SCkicW5TP2HiFpeK4z",
        "2MxENjhLejvbBZNnQPcKn44XYQ3uoiBT3fF",
        "2NFGG7yRBizUANU48b4dASrnNftqsNwzSM1",
        "2MukokUKMzhzsTEhniYTfgubTYxNUi6PtTX",
        "2Mygnzx3xh3r6ndwktC5z32gTjbRZXkJpFr",
        "2Mt8d1fTJEyJxiVT9YVVXMhmTxxsexLdJiE",
        "2N7YfFsFag5dkTAmHQ3EpaN4b4f3EPkwQkk",
        "2N9CxubDCQThkSdYmKJ1i6UNHFwoKBxp2Hj",
        "2NEMupTSk437zRY92iAiGNZCpDiwLvwnZEL",
        "2NAWKepFhtFy1Kd5s5C8HJEcThWTyzKiNGA",
        "2Mvy3yi71KxGHmk6dDbzAtxhbVPukK6MD5u",
        "2MzKURtstNFKFimJ4UfW4wv8ymSuQCcZPN2",
        "2NEdeQ6cqk1KerHsutnL1476XKDP2agcCh5",
        "2NFpMahbHRJ2HRp5ezXycpEpy5w2BmnVM9W",
        "2MuXzT5NSUwRzbAD1K6vvUDYqb3P9RUvPgK",
        "2NDt2aMj1BLjg6gRwuKn85jm2AhyAV8e2VF",
        "2N5PDFvrCCraXA3pv8CDqr5NxakT8KJb3Gg",
    ];

    static CASHADDR_MAINNET_P2PKH_ADDRESSES: [&str; 20] = [
        "bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk",
        "bitcoincash:qpxenfpcf975gxdjmq9pk3xm6hjmfj6re56t60smsm",
        "bitcoincash:qzfau6vrq980qntgp5e7l6cpfsf7jw88c5u7y85qx6",
        "bitcoincash:qzcguejjfxld867ck4zudc9a6y8mf6ftgqqrxzfmlh",
        "bitcoincash:qqm2lpqdfjsg8kkhwk0a3e3gypyswkd69urny99j70",
        "bitcoincash:qrccfa4qm3xfcrta78v7du75jjaww0ylnss5nxsy9s",
        "bitcoincash:qqdcsl6c879esyxyacmz7g6vtzwjjwtznsv65x6znz",
        "bitcoincash:qpr2ddwe8qnnh8h20mmn4zgrharmy0vuy5y4gr8gl2",
        "bitcoincash:qqymsmh0nhfhs9k5whhnjwfxyaumvtxm8g2z0s4f9y",
        "bitcoincash:qzwdmm83qjx7372wxgszaukan73ffn8ct54v6hs3dl",
        "bitcoincash:qzh3f9me5z5sn2w8euap2gyrp6kr7gf6my5mhjey6s",
        "bitcoincash:qrneuckcx69clprn4nnr82tf8sycqrs3ac4tr8m86f",
        "bitcoincash:qz742xef07g9w8q52mx0q6m9hp05hnzm657wqd0ce2",
        "bitcoincash:qq5dzl0drx8v0layyyuh5aupvxfs80ydmsp5444280",
        "bitcoincash:qpxedxtug7kpwd6tgf5vx08gjamel7sldsc40mxew8",
        "bitcoincash:qr4fs2m8tjmw54r2aqmadggzuagttkujgyrjs5d769",
        "bitcoincash:qrmed4fxlhkgay9nxw7zn9muew5ktkyjnuuawvycze",
        "bitcoincash:qqv3cpvmu4h0vqa6aly0urec7kwtuhe49yz6e7922v",
        "bitcoincash:qr39scfteeu5l573lzerchh6wc4cqkxeturafzfkk9",
        "bitcoincash:qzzjgw37vwls805c9fw6g9vqyupadst6wgmane0s4l",
    ];

    static CASHADDR_MAINNET_P2SH_ADDRESSES: [&str; 20] = [
        "bitcoincash:pph5kuz78czq00e3t85ugpgd7xmer5kr7crv8a2z4t",
        "bitcoincash:ppxenfpcf975gxdjmq9pk3xm6hjmfj6re5dw8qhctx",
        "bitcoincash:pzfau6vrq980qntgp5e7l6cpfsf7jw88c5tmegnra8",
        "bitcoincash:pzcguejjfxld867ck4zudc9a6y8mf6ftgqhxmdwcy2",
        "bitcoincash:pqm2lpqdfjsg8kkhwk0a3e3gypyswkd69u5ke2z39j",
        "bitcoincash:prccfa4qm3xfcrta78v7du75jjaww0ylns83wfh87d",
        "bitcoincash:pqdcsl6c879esyxyacmz7g6vtzwjjwtznsmlffapgl",
        "bitcoincash:ppr2ddwe8qnnh8h20mmn4zgrharmy0vuy5ns4vqtyh",
        "bitcoincash:pqymsmh0nhfhs9k5whhnjwfxyaumvtxm8ga8jlj27e",
        "bitcoincash:pzwdmm83qjx7372wxgszaukan73ffn8ct5zf8chjkz",
        "bitcoincash:pzh3f9me5z5sn2w8euap2gyrp6kr7gf6myr72a78pd",
        "bitcoincash:prneuckcx69clprn4nnr82tf8sycqrs3aczw7guyp5",
        "bitcoincash:pz742xef07g9w8q52mx0q6m9hp05hnzm65ftazgmzh",
        "bitcoincash:pq5dzl0drx8v0layyyuh5aupvxfs80ydmsk3g6jfuj",
        "bitcoincash:ppxedxtug7kpwd6tgf5vx08gjamel7slds0sj5p646",
        "bitcoincash:pr4fs2m8tjmw54r2aqmadggzuagttkujgy5hdm2apc",
        "bitcoincash:prmed4fxlhkgay9nxw7zn9muew5ktkyjnutcnrrmey",
        "bitcoincash:pqv3cpvmu4h0vqa6aly0urec7kwtuhe49y4ly3zf33",
        "bitcoincash:pr39scfteeu5l573lzerchh6wc4cqkxetu5c5dw4dc",
        "bitcoincash:pzzjgw37vwls805c9fw6g9vqyupadst6wgvcwkgnwz",
    ];

    static CASHADDR_TESTNET_P2PKH_ADDRESSES: [&str; 20] = [
        "bchtest:qph5kuz78czq00e3t85ugpgd7xmer5kr7csm740kf2",
        "bchtest:qpxenfpcf975gxdjmq9pk3xm6hjmfj6re57e7gjvh8",
        "bchtest:qzfau6vrq980qntgp5e7l6cpfsf7jw88c5cvqqkhpx",
        "bchtest:qzcguejjfxld867ck4zudc9a6y8mf6ftgqy3z9tvct",
        "bchtest:qqm2lpqdfjsg8kkhwk0a3e3gypyswkd69u8pqz89en",
        "bchtest:qrccfa4qm3xfcrta78v7du75jjaww0ylns5xhpjnzv",
        "bchtest:qqdcsl6c879esyxyacmz7g6vtzwjjwtznsggspc457",
        "bchtest:qpr2ddwe8qnnh8h20mmn4zgrharmy0vuy5q8vy9lck",
        "bchtest:qqymsmh0nhfhs9k5whhnjwfxyaumvtxm8gwsthh7zc",
        "bchtest:qzwdmm83qjx7372wxgszaukan73ffn8ct5377sjx2r",
        "bchtest:qzh3f9me5z5sn2w8euap2gyrp6kr7gf6mysfn4mnav",
        "bchtest:qrneuckcx69clprn4nnr82tf8sycqrs3ac3e8qesa4",
        "bchtest:qz742xef07g9w8q52mx0q6m9hp05hnzm656uy2d07k",
        "bchtest:qq5dzl0drx8v0layyyuh5aupvxfs80ydms9x3jhaqn",
        "bchtest:qpxedxtug7kpwd6tgf5vx08gjamel7sldsu8tuywfm",
        "bchtest:qr4fs2m8tjmw54r2aqmadggzuagttkujgy8q5n0fae",
        "bchtest:qrmed4fxlhkgay9nxw7zn9muew5ktkyjnuc02tx099",
        "bchtest:qqv3cpvmu4h0vqa6aly0urec7kwtuhe49yxgae8ads",
        "bchtest:qr39scfteeu5l573lzerchh6wc4cqkxetu80d9tp3e",
        "bchtest:qzzjgw37vwls805c9fw6g9vqyupadst6wgl0h7d8jr",
    ];

    static CASHADDR_TESTNET_P2SH_ADDRESSES: [&str; 20] = [
        "bchtest:pph5kuz78czq00e3t85ugpgd7xmer5kr7c87r6g4jh",
        "bchtest:ppxenfpcf975gxdjmq9pk3xm6hjmfj6re5fur840v6",
        "bchtest:pzfau6vrq980qntgp5e7l6cpfsf7jw88c50fa0356m",
        "bchtest:pzcguejjfxld867ck4zudc9a6y8mf6ftgqn5l2v0rk",
        "bchtest:pqm2lpqdfjsg8kkhwk0a3e3gypyswkd69usyadqxzw",
        "bchtest:prccfa4qm3xfcrta78v7du75jjaww0ylnsrr2w4se3",
        "bchtest:pqdcsl6c879esyxyacmz7g6vtzwjjwtznslddwlk0r",
        "bchtest:ppr2ddwe8qnnh8h20mmn4zgrharmy0vuy5hz3tzurt",
        "bchtest:pqymsmh0nhfhs9k5whhnjwfxyaumvtxm8ge4kcsae9",
        "bchtest:pzwdmm83qjx7372wxgszaukan73ffn8ct5xmrl4937",
        "bchtest:pzh3f9me5z5sn2w8euap2gyrp6kr7gf6my8vw6usx3",
        "bchtest:prneuckcx69clprn4nnr82tf8sycqrs3acxu607nxg",
        "bchtest:pz742xef07g9w8q52mx0q6m9hp05hnzm65dee92v9t",
        "bchtest:pq5dzl0drx8v0layyyuh5aupvxfs80ydmsjrvas7mw",
        "bchtest:ppxedxtug7kpwd6tgf5vx08gjamel7sldstzknrdjx",
        "bchtest:pr4fs2m8tjmw54r2aqmadggzuagttkujgys9fug2xy",
        "bchtest:prmed4fxlhkgay9nxw7zn9muew5ktkyjnu02hypv7c",
        "bchtest:pqv3cpvmu4h0vqa6aly0urec7kwtuhe49y3dqkq7kd",
        "bchtest:pr39scfteeu5l573lzerchh6wc4cqkxetus2s2vz2y",
        "bchtest:pzzjgw37vwls805c9fw6g9vqyupadst6wgg2232yf7",
    ];

    static CASHADDR_REGTEST_P2PKH_ADDRESSES: [&str; 20] = [
        "bchreg:qph5kuz78czq00e3t85ugpgd7xmer5kr7c28g5v92v",
        "bchreg:qpxenfpcf975gxdjmq9pk3xm6hjmfj6re5y9gf3l5p",
        "bchreg:qzfau6vrq980qntgp5e7l6cpfsf7jw88c5zskp4yzq",
        "bchreg:qzcguejjfxld867ck4zudc9a6y8mf6ftgq7d5yglmd",
        "bchreg:qqm2lpqdfjsg8kkhwk0a3e3gypyswkd69uaakryk64",
        "bchreg:qrccfa4qm3xfcrta78v7du75jjaww0ylnsw6pq3qp2",
        "bchreg:qqdcsl6c879esyxyacmz7g6vtzwjjwtznsj5xqmxhc",
        "bchreg:qpr2ddwe8qnnh8h20mmn4zgrharmy0vuy56m69xvms",
        "bchreg:qqymsmh0nhfhs9k5whhnjwfxyaumvtxm8g5vak5dp7",
        "bchreg:qzwdmm83qjx7372wxgszaukan73ffn8ct5tzg334f9",
        "bchreg:qzh3f9me5z5sn2w8euap2gyrp6kr7gf6my2495cq72",
        "bchreg:qrneuckcx69clprn4nnr82tf8sycqrs3act93p6r7n",
        "bchreg:qz742xef07g9w8q52mx0q6m9hp05hnzm65qqjtwuas",
        "bchreg:qq5dzl0drx8v0layyyuh5aupvxfs80ydmsl68n5wr4",
        "bchreg:qpxedxtug7kpwd6tgf5vx08gjamel7sldsxmaa8a2a",
        "bchreg:qr4fs2m8tjmw54r2aqmadggzuagttkujgyauzjv67l",
        "bchreg:qrmed4fxlhkgay9nxw7zn9muew5ktkyjnuznu29uxr",
        "bchreg:qqv3cpvmu4h0vqa6aly0urec7kwtuhe49yu5tcywwk",
        "bchreg:qr39scfteeu5l573lzerchh6wc4cqkxetuanmygjjl",
        "bchreg:qzzjgw37vwls805c9fw6g9vqyupadst6wg9nplw539"
    ];

    static CASHADDR_REGTEST_P2SH_ADDRESSES: [&str; 20] = [
        "bchreg:pph5kuz78czq00e3t85ugpgd7xmer5kr7caz4mtx33",
        "bchreg:ppxenfpcf975gxdjmq9pk3xm6hjmfj6re5nq4xku0u",
        "bchreg:pzfau6vrq980qntgp5e7l6cpfsf7jw88c544twj8ea",
        "bchreg:pzcguejjfxld867ck4zudc9a6y8mf6ftgqfgft0uqs",
        "bchreg:pqm2lpqdfjsg8kkhwk0a3e3gypyswkd69u2ctvr4pg",
        "bchreg:prccfa4qm3xfcrta78v7du75jjaww0ylnselu0kr6h",
        "bchreg:pqdcsl6c879esyxyacmz7g6vtzwjjwtzns93m0u9v9",
        "bchreg:ppr2ddwe8qnnh8h20mmn4zgrharmy0vuy5d782p0qd",
        "bchreg:pqymsmh0nhfhs9k5whhnjwfxyaumvtxm8grfqenw6r",
        "bchreg:pzwdmm83qjx7372wxgszaukan73ffn8ct5u847kkjc",
        "bchreg:pzh3f9me5z5sn2w8euap2gyrp6kr7gf6myascmlr9h",
        "bchreg:prneuckcx69clprn4nnr82tf8sycqrs3acuqvwaq9w",
        "bchreg:pz742xef07g9w8q52mx0q6m9hp05hnzm65h90yflxd",
        "bchreg:pq5dzl0drx8v0layyyuh5aupvxfs80ydmsgl6undcg",
        "bchreg:ppxedxtug7kpwd6tgf5vx08gjamel7slds37qjq73q",
        "bchreg:pr4fs2m8tjmw54r2aqmadggzuagttkujgy2elate9z",
        "bchreg:prmed4fxlhkgay9nxw7zn9muew5ktkyjnu4kp9zla7",
        "bchreg:pqv3cpvmu4h0vqa6aly0urec7kwtuhe49yt3khrd4t",
        "bchreg:pr39scfteeu5l573lzerchh6wc4cqkxetu2kxt03fz",
        "bchreg:pzzjgw37vwls805c9fw6g9vqyupadst6wgjkusfh2c"
    ];

    static SLPADDR_MAINNET_P2PKH_ADDRESSES: [&str; 20] = [
        "simpleledger:qph5kuz78czq00e3t85ugpgd7xmer5kr7ccj3fcpsg",
        "simpleledger:qpxenfpcf975gxdjmq9pk3xm6hjmfj6re5ks359mw9",
        "simpleledger:qzfau6vrq980qntgp5e7l6cpfsf7jw88c5s90upqcy",
        "simpleledger:qzcguejjfxld867ck4zudc9a6y8mf6ftgqvcdeumpf",
        "simpleledger:qqm2lpqdfjsg8kkhwk0a3e3gypyswkd69u0g07sjq3",
        "simpleledger:qrccfa4qm3xfcrta78v7du75jjaww0ylnsu0ca9ymw",
        "simpleledger:qqdcsl6c879esyxyacmz7g6vtzwjjwtznsqpla0zdu",
        "simpleledger:qpr2ddwe8qnnh8h20mmn4zgrharmy0vuy5gwrcjgp5",
        "simpleledger:qqymsmh0nhfhs9k5whhnjwfxyaumvtxm8gxeytqfm6",
        "simpleledger:qzwdmm83qjx7372wxgszaukan73ffn8ct5eh3v93np",
        "simpleledger:qzh3f9me5z5sn2w8euap2gyrp6kr7gf6mycqufvyyw",
        "simpleledger:qrneuckcx69clprn4nnr82tf8sycqrs3acesguw8yh",
        "simpleledger:qz742xef07g9w8q52mx0q6m9hp05hnzm65j4tk6c85",
        "simpleledger:qq5dzl0drx8v0layyyuh5aupvxfs80ydmsd07wq2e3",
        "simpleledger:qpxedxtug7kpwd6tgf5vx08gjamel7slds5wyqnese",
        "simpleledger:qr4fs2m8tjmw54r2aqmadggzuagttkujgy0fm0c7ym",
        "simpleledger:qrmed4fxlhkgay9nxw7zn9muew5ktkyjnusx9h3cu8",
        "simpleledger:qqv3cpvmu4h0vqa6aly0urec7kwtuhe49ywpj9s25j",
        "simpleledger:qr39scfteeu5l573lzerchh6wc4cqkxetu0xzeukgm",
        "simpleledger:qzzjgw37vwls805c9fw6g9vqyupadst6wghxcz6stp"
    ];

    static SLPADDR_MAINNET_P2SH_ADDRESSES: [&str; 20] = [
        "simpleledger:pph5kuz78czq00e3t85ugpgd7xmer5kr7c0hvxlzt4",
        "simpleledger:ppxenfpcf975gxdjmq9pk3xm6hjmfj6re5p4vmzc4c",
        "simpleledger:pzfau6vrq980qntgp5e7l6cpfsf7jw88c58qjnxrre",
        "simpleledger:pzcguejjfxld867ck4zudc9a6y8mf6ftgqmaskmc65",
        "simpleledger:pqm2lpqdfjsg8kkhwk0a3e3gypyswkd69ucdj3h3mv",
        "simpleledger:prccfa4qm3xfcrta78v7du75jjaww0ylnst29jz8qn",
        "simpleledger:pqdcsl6c879esyxyacmz7g6vtzwjjwtznshyzjgpkp",
        "simpleledger:ppr2ddwe8qnnh8h20mmn4zgrharmy0vuy5lt7h4t6f",
        "simpleledger:pqymsmh0nhfhs9k5whhnjwfxyaumvtxm8g3uey82q8",
        "simpleledger:pzwdmm83qjx7372wxgszaukan73ffn8ct5wjvrzjgu",
        "simpleledger:pzh3f9me5z5sn2w8euap2gyrp6kr7gf6my09pxt8ln",
        "simpleledger:prneuckcx69clprn4nnr82tf8sycqrs3acw44nfyl2",
        "simpleledger:pz742xef07g9w8q52mx0q6m9hp05hnzm659skeamuf",
        "simpleledger:pq5dzl0drx8v0layyyuh5aupvxfs80ydms62rp8fzv",
        "simpleledger:ppxedxtug7kpwd6tgf5vx08gjamel7sldsrte056ty",
        "simpleledger:pr4fs2m8tjmw54r2aqmadggzuagttkujgycvxqlalx",
        "simpleledger:prmed4fxlhkgay9nxw7zn9muew5ktkyjnu8rcckm86",
        "simpleledger:pqv3cpvmu4h0vqa6aly0urec7kwtuhe49yey02hf00",
        "simpleledger:pr39scfteeu5l573lzerchh6wc4cqkxetucrlkm4nx",
        "simpleledger:pzzjgw37vwls805c9fw6g9vqyupadst6wgqr9dansu"
    ];

    static SLPADDR_TESTNET_P2PKH_ADDRESSES: [&str; 20] = [
        "slptest:qph5kuz78czq00e3t85ugpgd7xmer5kr7ct0ew4pmh",
        "slptest:qpxenfpcf975gxdjmq9pk3xm6hjmfj6re59dengm96",
        "slptest:qzfau6vrq980qntgp5e7l6cpfsf7jw88c5rc8mvqnm",
        "slptest:qzcguejjfxld867ck4zudc9a6y8mf6ftgql9973m2k",
        "slptest:qqm2lpqdfjsg8kkhwk0a3e3gypyswkd69uu48eajtw",
        "slptest:qrccfa4qm3xfcrta78v7du75jjaww0ylns0js6gys3",
        "slptest:qqdcsl6c879esyxyacmz7g6vtzwjjwtznsnuh6zzxr",
        "slptest:qpr2ddwe8qnnh8h20mmn4zgrharmy0vuy5mntllg2t",
        "slptest:qqymsmh0nhfhs9k5whhnjwfxyaumvtxm8g4yvvdfs9",
        "slptest:qzwdmm83qjx7372wxgszaukan73ffn8ct522etg3c7",
        "slptest:qzh3f9me5z5sn2w8euap2gyrp6kr7gf6myta5wpy03",
        "slptest:qrneuckcx69clprn4nnr82tf8sycqrs3ac2dqmr80g",
        "slptest:qz742xef07g9w8q52mx0q6m9hp05hnzm65pgr3hcvt",
        "slptest:qq5dzl0drx8v0layyyuh5aupvxfs80ydms7jkfd2jw",
        "slptest:qpxedxtug7kpwd6tgf5vx08gjamel7slds8nv87emx",
        "slptest:qr4fs2m8tjmw54r2aqmadggzuagttkujgyu5ng470y",
        "slptest:qrmed4fxlhkgay9nxw7zn9muew5ktkyjnurmdsuchc",
        "slptest:qqv3cpvmu4h0vqa6aly0urec7kwtuhe49yau6za2ld",
        "slptest:qr39scfteeu5l573lzerchh6wc4cqkxetuum273kry",
        "slptest:qzzjgw37vwls805c9fw6g9vqyupadst6wgyms9hsq7"
    ];

    static SLPADDR_TESTNET_P2SH_ADDRESSES: [&str; 20] = [
        "slptest:pph5kuz78czq00e3t85ugpgd7xmer5kr7cu2ypjzq2",
        "slptest:ppxenfpcf975gxdjmq9pk3xm6hjmfj6re5jgyu0c78",
        "slptest:pzfau6vrq980qntgp5e7l6cpfsf7jw88c55a65trgx",
        "slptest:pzcguejjfxld867ck4zudc9a6y8mf6ftgqgqc3kc3t",
        "slptest:pqm2lpqdfjsg8kkhwk0a3e3gypyswkd69uts6k63sn",
        "slptest:prccfa4qm3xfcrta78v7du75jjaww0ylnschd408tv",
        "slptest:pqdcsl6c879esyxyacmz7g6vtzwjjwtznsye249pa7",
        "slptest:ppr2ddwe8qnnh8h20mmn4zgrharmy0vuy5vkksct3k",
        "slptest:pqymsmh0nhfhs9k5whhnjwfxyaumvtxm8gzp3r22tc",
        "slptest:pzwdmm83qjx7372wxgszaukan73ffn8ct5a0yy0jrr",
        "slptest:pzh3f9me5z5sn2w8euap2gyrp6kr7gf6myucfpx85v",
        "slptest:prneuckcx69clprn4nnr82tf8sycqrs3acaga5yy54",
        "slptest:pz742xef07g9w8q52mx0q6m9hp05hnzm65kd77smhk",
        "slptest:pq5dzl0drx8v0layyyuh5aupvxfs80ydmsfhtx2ffn",
        "slptest:ppxedxtug7kpwd6tgf5vx08gjamel7sldssk3ge6qm",
        "slptest:pr4fs2m8tjmw54r2aqmadggzuagttkujgyt3w8ja5e",
        "slptest:prmed4fxlhkgay9nxw7zn9muew5ktkyjnu57slmmv9",
        "slptest:pqv3cpvmu4h0vqa6aly0urec7kwtuhe49y2e8d6fys",
        "slptest:pr39scfteeu5l573lzerchh6wc4cqkxetut7h3k4ce",
        "slptest:pzzjgw37vwls805c9fw6g9vqyupadst6wgn7d2snmr"
    ];

    fn legacy_addresses() -> Vec<&'static str> {
        [
            LEGACY_MAINNET_P2PKH_ADDRESSES,
            LEGACY_MAINNET_P2SH_ADDRESSES,
            LEGACY_TESTNET_P2PKH_ADDRESSES,
            LEGACY_TESTNET_P2SH_ADDRESSES,
        ].concat()
    }

    fn legacy_testnet_addresses() -> Vec<&'static str> {
        [
            LEGACY_TESTNET_P2PKH_ADDRESSES,
            LEGACY_TESTNET_P2SH_ADDRESSES,
        ].concat()
    }

    fn cash_addresses() -> Vec<&'static str> {
        [
            CASHADDR_MAINNET_P2PKH_ADDRESSES,
            CASHADDR_MAINNET_P2SH_ADDRESSES,
            CASHADDR_TESTNET_P2PKH_ADDRESSES,
            CASHADDR_TESTNET_P2SH_ADDRESSES,
        ].concat()
    }

    fn cash_addresses_no_prefix() -> Vec<&'static str> {
        no_prefix(cash_addresses())
    }

    fn regtest_addresses() -> Vec<&'static str> {
        [
            CASHADDR_REGTEST_P2PKH_ADDRESSES,
            CASHADDR_REGTEST_P2SH_ADDRESSES,
        ].concat()
    }

    fn regtest_addresses_no_prefix() -> Vec<&'static str> {
        no_prefix(regtest_addresses())
    }

    fn slp_addresses() -> Vec<&'static str> {
        [
            SLPADDR_MAINNET_P2PKH_ADDRESSES,
            SLPADDR_MAINNET_P2SH_ADDRESSES,
            SLPADDR_TESTNET_P2PKH_ADDRESSES,
            SLPADDR_TESTNET_P2SH_ADDRESSES,
        ].concat()
    }

    fn slp_addresses_no_prefix() -> Vec<&'static str> {
        no_prefix(slp_addresses())
    }

    fn no_prefix(data: Vec<&'static str>) -> Vec<&'static str> {
        data.into_iter().map(|el| {
            el.splitn(2, ':').nth(1).unwrap()
        }).collect()
    }

    fn convert_test_base(converter: &Converter) {
        for (i, addr) in legacy_addresses().iter().enumerate() {
            let conv_cash = converter.to_cash_addr(addr).unwrap();
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();

            assert_eq!(conv_cash, cash_addresses()[i]);
            assert_eq!(conv_legacy, addr.to_string());
        }

        for (i, addr) in cash_addresses().iter().enumerate() {
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();
            let conv_cash = converter.to_cash_addr(addr).unwrap();

            assert_eq!(conv_legacy, legacy_addresses()[i]);
            assert_eq!(conv_cash, addr.to_string());
        }

        for (i, addr) in cash_addresses_no_prefix().iter().enumerate() {
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();
            let conv_cash = converter.to_cash_addr(addr).unwrap();

            assert_eq!(conv_legacy, legacy_addresses()[i]);
            assert_eq!(conv_cash, cash_addresses_no_prefix()[i]);
        }
    }

    #[test]
    fn convert_test() {
        let converter = Converter::new();

        convert_test_base(&converter);
    }

    #[test]
    fn regtest_addr() {
        let converter = Converter::new();

        for (i, addr) in legacy_testnet_addresses().iter().enumerate() {
            let conv_cash = converter.to_cash_addr_with_options(addr, None, Some(Network::Regtest)).unwrap();

            assert_eq!(conv_cash, regtest_addresses()[i]);
        }

        for (i, addr) in regtest_addresses().iter().enumerate() {
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();
            let conv_cash = converter.to_cash_addr_with_options(addr, None, Some(Network::Regtest)).unwrap();

            assert_eq!(conv_legacy, legacy_testnet_addresses()[i]);
            assert_eq!(conv_cash, addr.to_string());
        }

        for (i, addr) in regtest_addresses_no_prefix().iter().enumerate() {
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();
            let conv_cash = converter.to_cash_addr_with_options(addr, None, Some(Network::Regtest)).unwrap();

            assert_eq!(conv_legacy, legacy_testnet_addresses()[i]);
            assert_eq!(conv_cash, regtest_addresses_no_prefix()[i]);
        }
    }

    #[test]
    fn slp_convert() {
        let converter = Converter::new().add_prefixes(
            &[
                ("simpleledger", Network::Mainnet),
                ("slptest", Network::Testnet),
            ],
            "SLP"
        );

        convert_test_base(&converter);

        for (i, addr) in legacy_addresses().iter().enumerate() {
            let conv_cash = converter.to_cash_addr_with_options(addr, Some(AddressFormat::Other("SLP".to_string())), None).unwrap();

            assert_eq!(conv_cash, slp_addresses()[i]);
        }

        for (i, addr) in slp_addresses().iter().enumerate() {
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();
            let conv_cash = converter.to_cash_addr_with_options(addr, Some(AddressFormat::Other("SLP".to_string())), None).unwrap();

            assert_eq!(conv_legacy, legacy_addresses()[i]);
            assert_eq!(conv_cash, addr.to_string());
        }

        for (i, addr) in slp_addresses_no_prefix().iter().enumerate() {
            let conv_legacy = converter.to_legacy_addr(addr).unwrap();
            let conv_cash = converter.to_cash_addr_with_options(addr, Some(AddressFormat::Other("SLP".to_string())), None).unwrap();

            assert_eq!(conv_legacy, legacy_addresses()[i]);
            assert_eq!(conv_cash, slp_addresses_no_prefix()[i]);
        }
    }
}
