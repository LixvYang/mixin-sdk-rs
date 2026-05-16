use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::sign_authentication_token,
    error::Error,
    mix_address::{MIX_ADDRESS_PREFIX, MixAddress, hash256},
    models::Invoice as ApiInvoice,
    request::{ApiResponse, request},
    safe::SafeUser,
};

pub const MIXIN_INVOICE_VERSION: u8 = 0;
pub const MIXIN_INVOICE_PREFIX: &str = "MIN";
pub const EXTRA_SIZE_GENERAL_LIMIT: usize = 256;
pub const EXTRA_SIZE_STORAGE_CAPACITY: usize = 1024 * 1024 * 4;
pub const REFERENCES_COUNT_LIMIT: usize = 16;
pub const XIN_ASSET_ID: &str = "c94ac88f-4671-3976-b60a-09064f1811e8";

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InvoiceRequest {
    pub amount: String,
    pub asset_id: String,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceEntry {
    pub trace_id: String,
    pub asset_id: String,
    pub amount: String,
    pub extra: Vec<u8>,
    pub index_references: Vec<u8>,
    pub hash_references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixinInvoice {
    pub version: u8,
    pub recipient: MixAddress,
    pub entries: Vec<InvoiceEntry>,
}

impl MixinInvoice {
    pub fn new(recipient: &str) -> Result<Self, Error> {
        Ok(Self {
            version: MIXIN_INVOICE_VERSION,
            recipient: MixAddress::parse(recipient)?,
            entries: Vec::new(),
        })
    }

    pub fn parse(encoded: &str) -> Result<Self, Error> {
        parse_mixin_invoice(encoded)
    }

    pub fn add_entry(
        &mut self,
        trace_id: &str,
        asset_id: &str,
        amount: &str,
        extra: Vec<u8>,
        index_references: Vec<u8>,
        hash_references: Vec<String>,
    ) -> Result<(), Error> {
        if extra.len() >= EXTRA_SIZE_GENERAL_LIMIT {
            return Err(Error::Input(format!(
                "invoice entry extra too large: {}",
                extra.len()
            )));
        }
        if index_references.len() + hash_references.len() > REFERENCES_COUNT_LIMIT {
            return Err(Error::Input("too many invoice references".to_string()));
        }
        for index in &index_references {
            if *index as usize >= self.entries.len() {
                return Err(Error::Input(format!(
                    "invalid invoice index reference: {index}"
                )));
            }
        }
        validate_uuid(trace_id, "trace_id")?;
        validate_uuid(asset_id, "asset_id")?;
        for reference in &hash_references {
            validate_hash(reference)?;
        }

        self.entries.push(InvoiceEntry {
            trace_id: trace_id.to_string(),
            asset_id: asset_id.to_string(),
            amount: amount.to_string(),
            extra,
            index_references,
            hash_references,
        });
        Ok(())
    }

    pub fn add_storage_entry(&mut self, trace_id: &str, extra: Vec<u8>) -> Result<(), Error> {
        if extra.len() >= EXTRA_SIZE_STORAGE_CAPACITY {
            return Err(Error::Input(format!(
                "invoice storage extra too large: {}",
                extra.len()
            )));
        }
        validate_uuid(trace_id, "trace_id")?;
        self.entries.push(InvoiceEntry {
            trace_id: trace_id.to_string(),
            asset_id: XIN_ASSET_ID.to_string(),
            amount: estimate_storage_cost(extra.len()),
            extra,
            index_references: Vec::new(),
            hash_references: Vec::new(),
        });
        Ok(())
    }

    pub fn bytes_unchecked(&self) -> Result<Vec<u8>, Error> {
        let mut out = Vec::new();
        out.push(self.version);

        let recipient = self.recipient.bytes_unchecked()?;
        write_u16(&mut out, recipient.len(), "recipient")?;
        out.extend_from_slice(&recipient);

        if self.entries.len() > 128 {
            return Err(Error::Input(format!(
                "too many invoice entries: {}",
                self.entries.len()
            )));
        }
        out.push(self.entries.len() as u8);

        for entry in &self.entries {
            out.extend_from_slice(validate_uuid(&entry.trace_id, "trace_id")?.as_bytes());
            out.extend_from_slice(validate_uuid(&entry.asset_id, "asset_id")?.as_bytes());

            let amount = entry.amount.as_bytes();
            if amount.len() > 128 {
                return Err(Error::Input(format!(
                    "invoice amount too long: {}",
                    entry.amount
                )));
            }
            out.push(amount.len() as u8);
            out.extend_from_slice(amount);

            if entry.extra.len() >= EXTRA_SIZE_STORAGE_CAPACITY {
                return Err(Error::Input(format!(
                    "invoice extra too large: {}",
                    entry.extra.len()
                )));
            }
            write_u16(&mut out, entry.extra.len(), "extra")?;
            out.extend_from_slice(&entry.extra);

            let reference_count = entry.index_references.len() + entry.hash_references.len();
            if reference_count > REFERENCES_COUNT_LIMIT {
                return Err(Error::Input(format!(
                    "too many invoice references: {reference_count}"
                )));
            }
            out.push(reference_count as u8);
            for index in &entry.index_references {
                out.push(1);
                out.push(*index);
            }
            for reference in &entry.hash_references {
                out.push(0);
                out.extend_from_slice(&validate_hash(reference)?);
            }
        }

        Ok(out)
    }

    pub fn encode(&self) -> Result<String, Error> {
        let payload = self.bytes_unchecked()?;
        let mut checksum_input = Vec::with_capacity(MIXIN_INVOICE_PREFIX.len() + payload.len());
        checksum_input.extend_from_slice(MIXIN_INVOICE_PREFIX.as_bytes());
        checksum_input.extend_from_slice(&payload);

        let checksum = hash256(&checksum_input);
        let mut data = Vec::with_capacity(payload.len() + 4);
        data.extend_from_slice(&payload);
        data.extend_from_slice(&checksum[..4]);

        Ok(format!(
            "{MIXIN_INVOICE_PREFIX}{}",
            URL_SAFE_NO_PAD.encode(data)
        ))
    }
}

pub fn new_mixin_invoice(recipient: &str) -> Result<MixinInvoice, Error> {
    MixinInvoice::new(recipient)
}

pub fn parse_mixin_invoice(encoded: &str) -> Result<MixinInvoice, Error> {
    if !encoded.starts_with(MIXIN_INVOICE_PREFIX) {
        return Err(Error::Input(format!("invalid invoice prefix: {encoded}")));
    }
    let data = URL_SAFE_NO_PAD
        .decode(&encoded[MIXIN_INVOICE_PREFIX.len()..])
        .map_err(|e| Error::Input(format!("invalid invoice base64: {e}")))?;
    if data.len() < 3 + 16 + 4 {
        return Err(Error::Input(format!(
            "invalid invoice length: {}",
            data.len()
        )));
    }

    let checksum_index = data.len() - 4;
    let payload = &data[..checksum_index];
    let mut checksum_input = Vec::with_capacity(MIXIN_INVOICE_PREFIX.len() + payload.len());
    checksum_input.extend_from_slice(MIXIN_INVOICE_PREFIX.as_bytes());
    checksum_input.extend_from_slice(payload);
    let checksum = hash256(&checksum_input);
    if checksum[..4] != data[checksum_index..] {
        return Err(Error::Input("invalid invoice checksum".to_string()));
    }

    let mut decoder = Decoder::new(payload);
    let version = decoder.read_u8()?;
    if version != MIXIN_INVOICE_VERSION {
        return Err(Error::Input(format!("invalid invoice version: {version}")));
    }

    let recipient_len = decoder.read_u16()? as usize;
    let recipient_payload = decoder.read_exact(recipient_len)?;
    let recipient = MixAddress::parse(&mix_address_from_payload(recipient_payload)?)?;

    let entry_count = decoder.read_u8()? as usize;
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let trace_id = uuid_from_bytes(decoder.read_exact(16)?)?;
        let asset_id = uuid_from_bytes(decoder.read_exact(16)?)?;
        let amount_len = decoder.read_u8()? as usize;
        let amount = String::from_utf8(decoder.read_exact(amount_len)?.to_vec())
            .map_err(|e| Error::Input(format!("invalid invoice amount utf8: {e}")))?;
        let extra_len = decoder.read_u16()? as usize;
        let extra = decoder.read_exact(extra_len)?.to_vec();

        let reference_count = decoder.read_u8()? as usize;
        if reference_count > REFERENCES_COUNT_LIMIT {
            return Err(Error::Input(format!(
                "too many invoice references: {reference_count}"
            )));
        }
        let mut index_references = Vec::new();
        let mut hash_references = Vec::new();
        for _ in 0..reference_count {
            match decoder.read_u8()? {
                1 => index_references.push(decoder.read_u8()?),
                0 => hash_references.push(hex::encode(decoder.read_exact(32)?)),
                flag => {
                    return Err(Error::Input(format!(
                        "invalid invoice reference flag: {flag}"
                    )));
                }
            }
        }

        entries.push(InvoiceEntry {
            trace_id,
            asset_id,
            amount,
            extra,
            index_references,
            hash_references,
        });
    }

    if !decoder.is_finished() {
        return Err(Error::Input("invalid trailing invoice bytes".to_string()));
    }

    Ok(MixinInvoice {
        version,
        recipient,
        entries,
    })
}

#[deprecated(note = "Mixin invoices are offline MIN strings; use MixinInvoice::new/encode instead")]
pub async fn create_invoice(
    amount: &str,
    asset_id: &str,
    memo: Option<&str>,
    trace_id: Option<&str>,
    safe_user: &SafeUser,
) -> Result<ApiInvoice, Error> {
    let data = InvoiceRequest {
        amount: amount.to_string(),
        asset_id: asset_id.to_string(),
        memo: memo.map(|m| m.to_string()),
        trace_id: trace_id.map(|t| t.to_string()),
    };
    let data_str = serde_json::to_string(&data)?;
    let path = "/invoices";
    let token = sign_authentication_token("POST", path, &data_str, safe_user)?;
    let body = request("POST", path, data_str.as_bytes(), &token).await?;

    let parsed: ApiResponse<ApiInvoice> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain invoice data".to_string()))
}

#[deprecated(note = "Mixin invoices are offline MIN strings; use parse_mixin_invoice instead")]
pub async fn read_invoice(invoice_id: &str, safe_user: &SafeUser) -> Result<ApiInvoice, Error> {
    let path = format!("/invoices/{invoice_id}");
    let token = sign_authentication_token("GET", &path, "", safe_user)?;
    let body = request("GET", &path, &[], &token).await?;

    let parsed: ApiResponse<ApiInvoice> = serde_json::from_slice(&body)?;
    parsed
        .data
        .ok_or_else(|| Error::DataNotFound("API response did not contain invoice data".to_string()))
}

fn mix_address_from_payload(payload: &[u8]) -> Result<String, Error> {
    let mut checksum_input = Vec::with_capacity(MIX_ADDRESS_PREFIX.len() + payload.len());
    checksum_input.extend_from_slice(MIX_ADDRESS_PREFIX.as_bytes());
    checksum_input.extend_from_slice(payload);
    let checksum = hash256(&checksum_input);

    let mut data = Vec::with_capacity(payload.len() + 4);
    data.extend_from_slice(payload);
    data.extend_from_slice(&checksum[..4]);
    Ok(format!(
        "{MIX_ADDRESS_PREFIX}{}",
        bs58::encode(data).into_string()
    ))
}

fn validate_uuid(value: &str, name: &str) -> Result<Uuid, Error> {
    Uuid::parse_str(value).map_err(|e| Error::Input(format!("invalid invoice {name}: {e}")))
}

fn uuid_from_bytes(bytes: &[u8]) -> Result<String, Error> {
    Ok(Uuid::from_slice(bytes)
        .map_err(|e| Error::Input(format!("invalid invoice uuid bytes: {e}")))?
        .to_string())
}

fn validate_hash(value: &str) -> Result<Vec<u8>, Error> {
    let bytes = hex::decode(value)?;
    if bytes.len() != 32 {
        return Err(Error::Input(format!(
            "invalid invoice hash length: {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn write_u16(out: &mut Vec<u8>, value: usize, label: &str) -> Result<(), Error> {
    let value = u16::try_from(value)
        .map_err(|_| Error::Input(format!("invoice {label} length too large: {value}")))?;
    out.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn estimate_storage_cost(extra_len: usize) -> String {
    let steps = extra_len / 1024 + 1;
    format!("0.{:08}", steps * 10_000)
}

struct Decoder<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        let bytes = self.read_exact(1)?;
        Ok(bytes[0])
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| Error::Input("invoice cursor overflow".to_string()))?;
        if end > self.data.len() {
            return Err(Error::Input("truncated invoice payload".to_string()));
        }
        let bytes = &self.data[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn is_finished(&self) -> bool {
        self.offset == self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_request_serialization() {
        let request = InvoiceRequest {
            amount: "1".to_string(),
            asset_id: "asset-id".to_string(),
            memo: Some("memo".to_string()),
            trace_id: Some("trace".to_string()),
        };
        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
        assert_eq!(value["amount"], "1");
        assert_eq!(value["asset_id"], "asset-id");
        assert_eq!(value["memo"], "memo");
        assert_eq!(value["trace_id"], "trace");
    }

    #[test]
    fn test_mixin_invoice_encode_decode() {
        let btc = "c6d0c728-2624-429b-8e0d-d9d19b6592fa";
        let eth = "43d61dcd-e413-450d-80b8-101d5e903357";
        let recipient = "MIX4fwusRK88p5GexHWddUQuYJbKMJTAuBvhudgahRXKndvaM8FdPHS2Hgeo7DQxNVoSkKSEDyZeD8TYBhiwiea9PvCzay1A9Vx1C2nugc4iAmhwLGGv4h3GnABeCXHTwWEto9wEe1MWB49jLzy3nuoM81tqE2XnLvUWv";

        let mut invoice = MixinInvoice::new(recipient).unwrap();
        invoice
            .add_entry(
                "772e6bef-3bff-4fcc-987d-29bafca74d63",
                btc,
                "0.12345678",
                b"extra one".to_vec(),
                Vec::new(),
                vec![
                    "7ecf9fc49ff4d2e36424b8e53e67aed8cc4e9d08d7cbdca7d8bdb153ed2fcdde".to_string(),
                ],
            )
            .unwrap();
        invoice
            .add_entry(
                "3552d116-b29d-4d72-9b24-3ca3b2e0f9c2",
                eth,
                "0.23345678",
                b"extra two".to_vec(),
                vec![0],
                vec![
                    "4a5f79c76872524c6a4a81b174338584e790f09fb059c39cf2a894de1b3c31c6".to_string(),
                ],
            )
            .unwrap();

        let encoded = invoice.encode().unwrap();
        assert_eq!(
            encoded,
            "MINAABzAgQHZ6h4KBj1RqG2zMcql6d8Q8lKyI9GcTl2tgoJBk8YEejG0McoJiRCm44N2dGbZZL6Z6h4KBj1RqG2zMcql6d8Q8lKyI9GcTl2tgoJBk8YEejG0McoJiRCm44N2dGbZZL6Z6h4KBj1RqG2zMcql6d8QwJ3LmvvO_9PzJh9Kbr8p01jxtDHKCYkQpuODdnRm2WS-gowLjEyMzQ1Njc4AAlleHRyYSBvbmUBAH7Pn8Sf9NLjZCS45T5nrtjMTp0I18vcp9i9sVPtL83eNVLRFrKdTXKbJDyjsuD5wkPWHc3kE0UNgLgQHV6QM1cKMC4yMzM0NTY3OAAJZXh0cmEgdHdvAgEAAEpfecdoclJMakqBsXQzhYTnkPCfsFnDnPKolN4bPDHGTTpvYA"
        );

        let decoded = MixinInvoice::parse(&encoded).unwrap();
        assert_eq!(decoded.version, MIXIN_INVOICE_VERSION);
        assert_eq!(decoded.recipient.encode().unwrap(), recipient);
        assert_eq!(decoded.entries.len(), 2);
        assert_eq!(decoded.entries[0].asset_id, btc);
        assert_eq!(decoded.entries[0].amount, "0.12345678");
        assert_eq!(decoded.entries[0].extra, b"extra one");
        assert_eq!(
            decoded.entries[0].hash_references[0],
            "7ecf9fc49ff4d2e36424b8e53e67aed8cc4e9d08d7cbdca7d8bdb153ed2fcdde"
        );
        assert_eq!(decoded.entries[1].index_references, vec![0]);
        assert_eq!(
            decoded.entries[1].hash_references[0],
            "4a5f79c76872524c6a4a81b174338584e790f09fb059c39cf2a894de1b3c31c6"
        );
    }

    #[test]
    fn test_storage_cost() {
        assert_eq!(estimate_storage_cost(255), "0.00010000");
        assert_eq!(estimate_storage_cost(256), "0.00010000");
        assert_eq!(estimate_storage_cost(1024), "0.00020000");
        assert_eq!(estimate_storage_cost(1025), "0.00020000");
    }
}
