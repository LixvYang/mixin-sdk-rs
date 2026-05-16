use std::fmt;

use sha3::{Digest, Sha3_256};
use uuid::Uuid;

use crate::error::Error;

pub const MAINNET_ADDRESS_PREFIX: &str = "XIN";
pub const MIX_ADDRESS_PREFIX: &str = "MIX";
pub const MIX_ADDRESS_VERSION: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixAddress {
    pub version: u8,
    pub threshold: u8,
    pub uuid_members: Vec<String>,
    pub xin_members: Vec<String>,
}

impl MixAddress {
    pub fn new_uuid(members: Vec<String>, threshold: u8) -> Result<Self, Error> {
        validate_member_count(members.len(), threshold, true)?;
        for member in &members {
            Uuid::parse_str(member)
                .map_err(|e| Error::Input(format!("invalid uuid member {member}: {e}")))?;
        }
        Ok(Self {
            version: MIX_ADDRESS_VERSION,
            threshold,
            uuid_members: members,
            xin_members: Vec::new(),
        })
    }

    pub fn new_mainnet(members: Vec<String>, threshold: u8) -> Result<Self, Error> {
        validate_member_count(members.len(), threshold, false)?;
        for member in &members {
            get_public_from_mainnet_address(member)?;
        }
        Ok(Self {
            version: MIX_ADDRESS_VERSION,
            threshold,
            uuid_members: Vec::new(),
            xin_members: members,
        })
    }

    pub fn parse(encoded: &str) -> Result<Self, Error> {
        if !encoded.starts_with(MIX_ADDRESS_PREFIX) {
            return Err(Error::Input(format!(
                "invalid mix address prefix: {encoded}"
            )));
        }

        let data = bs58::decode(&encoded[MIX_ADDRESS_PREFIX.len()..])
            .into_vec()
            .map_err(|e| Error::Input(format!("invalid mix address base58: {e}")))?;
        if data.len() < 3 + 16 + 4 {
            return Err(Error::Input(format!(
                "invalid mix address length: {}",
                data.len()
            )));
        }

        let checksum_index = data.len() - 4;
        let payload = &data[..checksum_index];
        let expected = checksum(MIX_ADDRESS_PREFIX.as_bytes(), payload);
        if expected[..4] != data[checksum_index..] {
            return Err(Error::Input("invalid mix address checksum".to_string()));
        }

        let version = payload[0];
        let threshold = payload[1];
        let total = payload[2] as usize;
        if version != MIX_ADDRESS_VERSION {
            return Err(Error::Input(format!(
                "invalid mix address version: {version}"
            )));
        }
        if threshold == 0 || total == 0 || total > 64 {
            return Err(Error::Input(format!(
                "invalid mix address threshold/count: {threshold}/{total}"
            )));
        }

        let member_data = &payload[3..];
        if member_data.len() == total * 16 {
            let mut uuid_members = Vec::with_capacity(total);
            for chunk in member_data.chunks_exact(16) {
                let uuid = Uuid::from_slice(chunk)
                    .map_err(|e| Error::Input(format!("invalid uuid member bytes: {e}")))?;
                uuid_members.push(uuid.to_string());
            }
            return Ok(Self {
                version,
                threshold,
                uuid_members,
                xin_members: Vec::new(),
            });
        }

        if member_data.len() == total * 64 {
            let mut xin_members = Vec::with_capacity(total);
            for chunk in member_data.chunks_exact(64) {
                xin_members.push(mainnet_address_from_public(chunk)?);
            }
            return Ok(Self {
                version,
                threshold,
                uuid_members: Vec::new(),
                xin_members,
            });
        }

        Err(Error::Input(
            "invalid mix address member payload".to_string(),
        ))
    }

    pub fn members(&self) -> Vec<String> {
        let mut members = if self.uuid_members.is_empty() {
            self.xin_members.clone()
        } else {
            self.uuid_members.clone()
        };
        members.sort();
        members
    }

    pub fn bytes_unchecked(&self) -> Result<Vec<u8>, Error> {
        let members = if self.uuid_members.is_empty() {
            &self.xin_members
        } else {
            &self.uuid_members
        };
        if members.len() > u8::MAX as usize {
            return Err(Error::Input(format!(
                "too many mix address members: {}",
                members.len()
            )));
        }

        let mut payload = Vec::with_capacity(3 + members.len() * 64);
        payload.push(self.version);
        payload.push(self.threshold);
        payload.push(members.len() as u8);

        if !self.uuid_members.is_empty() {
            for member in &self.uuid_members {
                let uuid = Uuid::parse_str(member)
                    .map_err(|e| Error::Input(format!("invalid uuid member {member}: {e}")))?;
                payload.extend_from_slice(uuid.as_bytes());
            }
        } else {
            for member in &self.xin_members {
                payload.extend_from_slice(&get_public_from_mainnet_address(member)?);
            }
        }

        Ok(payload)
    }

    pub fn encode(&self) -> Result<String, Error> {
        let payload = self.bytes_unchecked()?;
        mix_address_from_payload(&payload)
    }
}

impl fmt::Display for MixAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.encode() {
            Ok(value) => f.write_str(&value),
            Err(_) => Err(fmt::Error),
        }
    }
}

pub fn new_uuid_mix_address(members: Vec<String>, threshold: u8) -> Result<MixAddress, Error> {
    MixAddress::new_uuid(members, threshold)
}

pub fn new_mainnet_mix_address(members: Vec<String>, threshold: u8) -> Result<MixAddress, Error> {
    MixAddress::new_mainnet(members, threshold)
}

pub fn parse_mix_address(encoded: &str) -> Result<MixAddress, Error> {
    MixAddress::parse(encoded)
}

pub fn mix_address_from_payload(payload: &[u8]) -> Result<String, Error> {
    let sum = checksum(MIX_ADDRESS_PREFIX.as_bytes(), payload);
    let mut data = Vec::with_capacity(payload.len() + 4);
    data.extend_from_slice(payload);
    data.extend_from_slice(&sum[..4]);
    Ok(format!(
        "{MIX_ADDRESS_PREFIX}{}",
        bs58::encode(data).into_string()
    ))
}

pub fn get_public_from_mainnet_address(address: &str) -> Result<Vec<u8>, Error> {
    if !address.starts_with(MAINNET_ADDRESS_PREFIX) {
        return Err(Error::Input(format!(
            "invalid mainnet address prefix: {address}"
        )));
    }
    let data = bs58::decode(&address[MAINNET_ADDRESS_PREFIX.len()..])
        .into_vec()
        .map_err(|e| Error::Input(format!("invalid mainnet address base58: {e}")))?;
    if data.len() != 68 {
        return Err(Error::Input(format!(
            "invalid mainnet address length: {}",
            data.len()
        )));
    }

    let checksum_index = data.len() - 4;
    let payload = &data[..checksum_index];
    let expected = checksum(MAINNET_ADDRESS_PREFIX.as_bytes(), payload);
    if expected[..4] != data[checksum_index..] {
        return Err(Error::Input("invalid mainnet address checksum".to_string()));
    }
    Ok(payload.to_vec())
}

pub fn mainnet_address_from_public(public_key: &[u8]) -> Result<String, Error> {
    if public_key.len() != 64 {
        return Err(Error::Input(format!(
            "invalid mainnet public key length: {}",
            public_key.len()
        )));
    }
    let sum = checksum(MAINNET_ADDRESS_PREFIX.as_bytes(), public_key);
    let mut data = Vec::with_capacity(public_key.len() + 4);
    data.extend_from_slice(public_key);
    data.extend_from_slice(&sum[..4]);
    Ok(format!(
        "{MAINNET_ADDRESS_PREFIX}{}",
        bs58::encode(data).into_string()
    ))
}

pub fn hash256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn checksum(prefix: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut data = Vec::with_capacity(prefix.len() + payload.len());
    data.extend_from_slice(prefix);
    data.extend_from_slice(payload);
    hash256(&data)
}

fn validate_member_count(
    count: usize,
    threshold: u8,
    enforce_threshold_upper: bool,
) -> Result<(), Error> {
    if count == 0 || count > u8::MAX as usize {
        return Err(Error::Input(format!(
            "invalid mix address member count: {count}"
        )));
    }
    if threshold == 0 || (enforce_threshold_upper && threshold as usize > count) {
        return Err(Error::Input(format!(
            "invalid mix address threshold: {threshold}/{count}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_mix_address() {
        let members = vec!["67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string()];
        let address = MixAddress::new_uuid(members.clone(), 1).unwrap();
        assert_eq!(
            address.encode().unwrap(),
            "MIX3QEeg1WkLrjvjxyMQf6Xc8dxs81tpPc"
        );

        let parsed = MixAddress::parse("MIX3QEeg1WkLrjvjxyMQf6Xc8dxs81tpPc").unwrap();
        assert_eq!(parsed.members(), members);
        assert_eq!(parsed.threshold, 1);
        assert_eq!(parsed.version, MIX_ADDRESS_VERSION);

        let members = vec![
            "67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string(),
            "c94ac88f-4671-3976-b60a-09064f1811e8".to_string(),
            "c6d0c728-2624-429b-8e0d-d9d19b6592fa".to_string(),
            "67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string(),
            "c94ac88f-4671-3976-b60a-09064f1811e8".to_string(),
            "c6d0c728-2624-429b-8e0d-d9d19b6592fa".to_string(),
            "67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string(),
        ];
        let address = MixAddress::new_uuid(members.clone(), 4).unwrap();
        assert_eq!(
            address.encode().unwrap(),
            "MIX4fwusRK88p5GexHWddUQuYJbKMJTAuBvhudgahRXKndvaM8FdPHS2Hgeo7DQxNVoSkKSEDyZeD8TYBhiwiea9PvCzay1A9Vx1C2nugc4iAmhwLGGv4h3GnABeCXHTwWEto9wEe1MWB49jLzy3nuoM81tqE2XnLvUWv"
        );
        let parsed = MixAddress::parse(&address.encode().unwrap()).unwrap();
        let mut sorted = members;
        sorted.sort();
        assert_eq!(parsed.members(), sorted);
    }

    #[test]
    fn test_mainnet_mix_address() {
        let members = vec![
            "XIN3BMNy9pQyj5XWDJtTbaBVE2zQ66zBo2weyc43iL286asdqwApWswAzQC5qba26fh3fzHK9iMoxyx1q3Lgj45KJftzGD9q".to_string(),
        ];
        let address = MixAddress::new_mainnet(members.clone(), 1).unwrap();
        assert_eq!(
            address.encode().unwrap(),
            "MIXPYWwhjxKsbFRzAP2Dcb2mMjj7sQQo4MpCSv3NYaYCdQ2kEcbcimpPT81gaxtuNhunLWPx7Sv7fawjZ8DhRmEj8E2hrQM4Z6e"
        );
        let parsed = MixAddress::parse(&address.encode().unwrap()).unwrap();
        assert_eq!(parsed.members(), members);
        assert_eq!(parsed.threshold, 1);
    }

    #[test]
    fn test_storage_threshold_mix_address_parses() {
        let parsed = MixAddress::parse(
            "MIXSK624cFT3CXbbjYxU17CeYWCwj6CZgkp2VsfiRsDMXw4MzpfYKPKKYwLmfDby2z85MLAbSWZbAB1dfPetCxUf7vwwJnToaG8",
        )
        .unwrap();
        assert_eq!(parsed.threshold, 64);
        assert_eq!(parsed.version, MIX_ADDRESS_VERSION);
        assert_eq!(parsed.members().len(), 1);
    }
}
