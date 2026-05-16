use std::collections::BTreeMap;

use curve25519_dalek::{
    edwards::{CompressedEdwardsY, EdwardsPoint},
    scalar::Scalar,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use uuid::Uuid;

use crate::{
    error::Error,
    mix_address::{MixAddress, get_public_from_mainnet_address, hash256},
    models::Output as UtxoOutput,
    output::list_unspent_outputs,
    safe::{GhostKeyRequest, GhostKeys, SafeUser, request_safe_ghost_keys},
    transaction::{TransactionRequest, TransactionView, send_transactions, verify_transactions},
    utils::{hash_members, unique_conversation_id},
};

pub const TX_VERSION_HASH_SIGNATURE: u8 = 0x05;
pub const OUTPUT_TYPE_SCRIPT: u8 = 0x00;
pub const OUTPUT_TYPE_WITHDRAWAL_SUBMIT: u8 = 0xa1;
pub const REFERENCES_COUNT_LIMIT: usize = 16;
pub const EXTRA_SIZE_GENERAL_LIMIT: usize = 256;
pub const EXTRA_SIZE_STORAGE_CAPACITY: usize = 1024 * 1024 * 4;
pub const EXTRA_SIZE_STORAGE_STEP: usize = 1024;
pub const EXTRA_STORAGE_PRICE_STEP_FIXED: u128 = 10_000;

const MAGIC: [u8; 2] = [0x77, 0x77];
const EMPTY: [u8; 2] = [0x00, 0x00];
const DECIMALS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SafeTransaction {
    pub version: u8,
    pub asset: String,
    pub inputs: Vec<SafeTransactionInput>,
    pub outputs: Vec<SafeTransactionOutput>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub extra: Vec<u8>,
    #[serde(default, rename = "signatureMap")]
    pub signature_map: Vec<BTreeMap<u16, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SafeTransactionInput {
    pub hash: String,
    pub index: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genesis: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SafeTransactionOutput {
    #[serde(rename = "type")]
    pub output_type: u8,
    pub amount: String,
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub script: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withdrawal: Option<SafeWithdrawalData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SafeWithdrawalData {
    pub address: String,
    #[serde(default)]
    pub tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafeTransactionRecipient {
    MixAddress {
        mix_address: MixAddress,
        amount: String,
    },
    Withdrawal {
        destination: String,
        tag: String,
        amount: String,
    },
}

impl SafeTransaction {
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            version: TX_VERSION_HASH_SIGNATURE,
            asset: asset.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            references: Vec::new(),
            extra: Vec::new(),
            signature_map: Vec::new(),
        }
    }
}

impl SafeTransactionInput {
    pub fn new(hash: impl Into<String>, index: u16) -> Self {
        Self {
            hash: hash.into(),
            index,
            genesis: Vec::new(),
        }
    }
}

impl SafeTransactionOutput {
    pub fn script(
        amount: impl Into<String>,
        keys: Vec<String>,
        mask: impl Into<String>,
        threshold: u8,
    ) -> Result<Self, Error> {
        Ok(Self {
            output_type: OUTPUT_TYPE_SCRIPT,
            amount: amount.into(),
            keys,
            mask: Some(mask.into()),
            script: encode_script(threshold)?,
            withdrawal: None,
        })
    }

    pub fn withdrawal(
        amount: impl Into<String>,
        destination: impl Into<String>,
        tag: impl Into<String>,
    ) -> Self {
        Self {
            output_type: OUTPUT_TYPE_WITHDRAWAL_SUBMIT,
            amount: amount.into(),
            keys: Vec::new(),
            mask: None,
            script: String::new(),
            withdrawal: Some(SafeWithdrawalData {
                address: destination.into(),
                tag: tag.into(),
            }),
        }
    }
}

impl SafeTransactionRecipient {
    pub fn mix_address(mix_address: MixAddress, amount: impl Into<String>) -> Self {
        Self::MixAddress {
            mix_address,
            amount: amount.into(),
        }
    }

    pub fn withdrawal(
        destination: impl Into<String>,
        tag: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        Self::Withdrawal {
            destination: destination.into(),
            tag: tag.into(),
            amount: amount.into(),
        }
    }

    fn amount(&self) -> &str {
        match self {
            Self::MixAddress { amount, .. } | Self::Withdrawal { amount, .. } => amount,
        }
    }
}

pub fn encode_script(threshold: u8) -> Result<String, Error> {
    if threshold > 64 {
        return Err(Error::Input(format!(
            "invalid script threshold: {threshold}"
        )));
    }
    Ok(format!("fffe{threshold:02x}"))
}

pub fn encode_safe_transaction(tx: &SafeTransaction) -> Result<String, Error> {
    encode_safe_transaction_with_signatures(tx, &tx.signature_map)
}

pub fn encode_unsigned_safe_transaction(tx: &SafeTransaction) -> Result<String, Error> {
    encode_safe_transaction_with_signatures(tx, &[])
}

pub fn encode_safe_transaction_with_signatures(
    tx: &SafeTransaction,
    signature_map: &[BTreeMap<u16, String>],
) -> Result<String, Error> {
    if tx.version != TX_VERSION_HASH_SIGNATURE {
        return Err(Error::Input(format!(
            "invalid safe transaction version: {}",
            tx.version
        )));
    }
    let mut enc = Encoder::new();
    enc.write(&MAGIC);
    enc.write(&[0x00, tx.version]);
    enc.write(&decode_fixed_hex(&tx.asset, 32, "asset")?);

    enc.write_int(tx.inputs.len())?;
    for input in &tx.inputs {
        enc.encode_input(input)?;
    }

    enc.write_int(tx.outputs.len())?;
    for output in &tx.outputs {
        enc.encode_output(output)?;
    }

    enc.write_int(tx.references.len())?;
    for reference in &tx.references {
        enc.write(&decode_fixed_hex(reference, 32, "reference")?);
    }

    enc.write_u32(tx.extra.len())?;
    enc.write(&tx.extra);

    enc.write_int(signature_map.len())?;
    for signatures in signature_map {
        enc.encode_signature(signatures)?;
    }

    Ok(hex::encode(enc.into_inner()))
}

pub fn decode_safe_transaction(raw: &str) -> Result<SafeTransaction, Error> {
    let bytes = hex::decode(raw)?;
    let mut dec = Decoder::new(&bytes);

    let magic = dec.read_exact(2)?;
    if magic != MAGIC {
        return Err(Error::Input("invalid safe transaction magic".to_string()));
    }
    let marker = dec.read_u8()?;
    if marker != 0 {
        return Err(Error::Input(format!(
            "invalid safe transaction marker: {marker}"
        )));
    }
    let version = dec.read_u8()?;
    if version != TX_VERSION_HASH_SIGNATURE {
        return Err(Error::Input(format!(
            "invalid safe transaction version: {version}"
        )));
    }

    let asset = hex::encode(dec.read_exact(32)?);

    let input_count = dec.read_u16()? as usize;
    let mut inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        inputs.push(dec.decode_input()?);
    }

    let output_count = dec.read_u16()? as usize;
    let mut outputs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
        outputs.push(dec.decode_output()?);
    }

    let reference_count = dec.read_u16()? as usize;
    let mut references = Vec::with_capacity(reference_count);
    for _ in 0..reference_count {
        references.push(hex::encode(dec.read_exact(32)?));
    }

    let extra_len = dec.read_u32()? as usize;
    let extra = dec.read_exact(extra_len)?.to_vec();

    let signature_count = dec.read_u16()? as usize;
    let mut signature_map = Vec::with_capacity(signature_count);
    for _ in 0..signature_count {
        signature_map.push(dec.decode_signature()?);
    }
    if !dec.is_empty() {
        return Err(Error::Input("trailing safe transaction bytes".to_string()));
    }

    Ok(SafeTransaction {
        version,
        asset,
        inputs,
        outputs,
        references,
        extra,
        signature_map,
    })
}

pub fn build_safe_transaction(
    utxos: &[UtxoOutput],
    recipients: &[SafeTransactionRecipient],
    ghosts: &[Option<GhostKeys>],
    extra: Vec<u8>,
    references: Vec<String>,
) -> Result<SafeTransaction, Error> {
    if utxos.is_empty() {
        return Err(Error::Input("empty safe transaction inputs".to_string()));
    }
    if references.len() > REFERENCES_COUNT_LIMIT {
        return Err(Error::Input(format!(
            "too many references: {}",
            references.len()
        )));
    }
    validate_extra_for_recipients(&extra, recipients)?;

    let mut asset = String::new();
    let mut inputs = Vec::with_capacity(utxos.len());
    for utxo in utxos {
        let utxo_asset = output_asset(utxo)?;
        if asset.is_empty() {
            asset = utxo_asset;
        } else if asset != utxo_asset {
            return Err(Error::Input("inconsistent asset in outputs".to_string()));
        }

        let hash = utxo
            .transaction_hash
            .as_ref()
            .ok_or_else(|| Error::Input("output is missing transaction_hash".to_string()))?;
        decode_fixed_hex(hash, 32, "transaction_hash")?;
        let index = utxo
            .output_index
            .ok_or_else(|| Error::Input("output is missing output_index".to_string()))?;
        if index > u16::MAX as u32 {
            return Err(Error::Input(format!("output_index overflow: {index}")));
        }
        inputs.push(SafeTransactionInput::new(hash, index as u16));
    }

    let mut outputs = Vec::with_capacity(recipients.len());
    for (i, recipient) in recipients.iter().enumerate() {
        match recipient {
            SafeTransactionRecipient::Withdrawal {
                destination,
                tag,
                amount,
            } => outputs.push(SafeTransactionOutput::withdrawal(amount, destination, tag)),
            SafeTransactionRecipient::MixAddress {
                mix_address,
                amount,
            } => {
                let ghost = ghosts
                    .get(i)
                    .and_then(|ghost| ghost.as_ref())
                    .ok_or_else(|| {
                        Error::Input(format!("missing ghost key for recipient index {i}"))
                    })?;
                let threshold = mix_address.threshold;
                outputs.push(SafeTransactionOutput::script(
                    amount,
                    ghost.keys.clone(),
                    ghost.mask.clone(),
                    threshold,
                )?);
            }
        }
    }

    Ok(SafeTransaction {
        version: TX_VERSION_HASH_SIGNATURE,
        asset,
        inputs,
        outputs,
        references,
        extra,
        signature_map: Vec::new(),
    })
}

pub async fn send_transfer_transaction(
    asset_id: &str,
    receiver: &str,
    amount: &str,
    trace_id: &str,
    extra: Vec<u8>,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let mix_address = MixAddress::new_uuid(vec![receiver.to_string()], 1)?;
    let recipient = SafeTransactionRecipient::mix_address(mix_address, amount);
    send_safe_transaction(
        asset_id,
        &[recipient],
        trace_id,
        extra,
        Vec::new(),
        safe_user,
    )
    .await
}

pub async fn send_safe_transaction(
    asset_id: &str,
    recipients: &[SafeTransactionRecipient],
    trace_id: &str,
    extra: Vec<u8>,
    references: Vec<String>,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let normalized_asset_id = normalize_asset_id(asset_id)?;
    let members_hash = hash_members([safe_user.user_id.as_str()]);
    let outputs =
        list_unspent_outputs(&members_hash, 1, Some(&normalized_asset_id), safe_user).await?;
    let (count, _) = get_unspent_outputs_for_recipients(&outputs, recipients)?;
    send_safe_transaction_with_outputs(
        &outputs[..count],
        recipients,
        trace_id,
        extra,
        references,
        safe_user,
    )
    .await
}

pub async fn send_safe_transaction_with_outputs(
    utxos: &[UtxoOutput],
    recipients: &[SafeTransactionRecipient],
    trace_id: &str,
    extra: Vec<u8>,
    references: Vec<String>,
    safe_user: &SafeUser,
) -> Result<TransactionView, Error> {
    let mut recipients = recipients.to_vec();
    let change = validate_totals_and_change(utxos, &recipients)?;
    if parse_units(&change)? > 0 {
        let change_address = MixAddress::new_uuid(vec![safe_user.user_id.clone()], 1)?;
        recipients.push(SafeTransactionRecipient::mix_address(
            change_address,
            change,
        ));
    }

    let ghosts = request_ghost_recipients_with_trace_id(&recipients, trace_id, safe_user).await?;
    let tx = build_safe_transaction(utxos, &recipients, &ghosts, extra, references)?;
    let raw = encode_unsigned_safe_transaction(&tx)?;

    let verified = expect_one_transaction(
        verify_transactions(
            &[TransactionRequest {
                request_id: trace_id.to_string(),
                raw,
            }],
            safe_user,
        )
        .await?,
    )?;
    if verified.state.as_deref() != Some("unspent") {
        return Err(Error::Input(format!(
            "transaction request is not unspent: {:?}",
            verified.state
        )));
    }
    let views = verified
        .views
        .ok_or_else(|| Error::DataNotFound("sequencer response is missing views".to_string()))?;
    if views.len() != tx.inputs.len() {
        return Err(Error::Input(format!(
            "invalid view keys count {} != {}",
            views.len(),
            tx.inputs.len()
        )));
    }

    let signed_raw = sign_safe_transaction_with_index(
        &tx,
        &views,
        &safe_user.spend_private_key,
        safe_user.is_spend_private_sum,
        0,
    )?;
    expect_one_transaction(
        send_transactions(
            &[TransactionRequest {
                request_id: trace_id.to_string(),
                raw: signed_raw,
            }],
            safe_user,
        )
        .await?,
    )
}

pub fn get_unspent_outputs_for_recipients(
    outputs: &[UtxoOutput],
    recipients: &[SafeTransactionRecipient],
) -> Result<(usize, String), Error> {
    let total_output = recipients.iter().try_fold(0u128, |total, recipient| {
        Ok::<_, Error>(total + parse_units(recipient.amount())?)
    })?;
    let mut total_input = 0u128;
    for (i, output) in outputs.iter().enumerate() {
        total_input = total_input
            .checked_add(output_amount(output)?)
            .ok_or_else(|| Error::Input("input amount overflow".to_string()))?;
        if total_input >= total_output {
            return Ok((i + 1, format_units(total_input - total_output)));
        }
    }
    Err(Error::Input(format!(
        "insufficient outputs {} < {}",
        format_units(total_input),
        format_units(total_output)
    )))
}

pub async fn request_ghost_recipients_with_trace_id(
    recipients: &[SafeTransactionRecipient],
    trace_id: &str,
    safe_user: &SafeUser,
) -> Result<Vec<Option<GhostKeys>>, Error> {
    let trace_hash = blake3::hash(trace_id.as_bytes());
    let private_spend = spend_private_key_bytes(&safe_user.spend_private_key)?;

    let mut ghosts: Vec<Option<GhostKeys>> = std::iter::repeat_with(|| None)
        .take(recipients.len())
        .collect();
    let mut uuid_requests = Vec::new();

    for (i, recipient) in recipients.iter().enumerate() {
        let SafeTransactionRecipient::MixAddress { mix_address, .. } = recipient else {
            continue;
        };

        let seed_hash = blake3_hash_many(&[
            trace_hash.as_bytes(),
            &integer_to_bytes_without_zero(i as u128),
        ]);
        if !mix_address.xin_members.is_empty() {
            let priv_hash = blake3_hash_many(&[&seed_hash, &private_spend]);
            let key_seed = bytes64(trace_hash.as_bytes(), &priv_hash);
            let key = new_key_from_seed(&key_seed);
            let mask = hex::encode(public_from_private_scalar(&key)?);
            let mut keys = Vec::with_capacity(mix_address.xin_members.len());
            for member in &mix_address.xin_members {
                let public = get_public_from_mainnet_address(member)?;
                let spend_key = fixed_bytes32(&public[..32], "mainnet spend key")?;
                let view_key = fixed_bytes32(&public[32..64], "mainnet view key")?;
                keys.push(hex::encode(derive_ghost_public_key(
                    &key, &view_key, &spend_key, i as u64,
                )?));
            }
            ghosts[i] = Some(GhostKeys {
                key_type: "ghost_key".to_string(),
                mask,
                keys,
            });
            continue;
        }

        let hint =
            unique_conversation_id(&hex::encode(trace_hash.as_bytes()), &hex::encode(seed_hash));
        uuid_requests.push(GhostKeyRequest {
            receivers: mix_address.members(),
            index: i as u32,
            hint,
        });
    }

    if !uuid_requests.is_empty() {
        let uuid_ghosts = request_safe_ghost_keys(&uuid_requests, safe_user).await?;
        for (ghost, request) in uuid_ghosts.into_iter().zip(uuid_requests.iter()) {
            let index = request.index as usize;
            if index >= ghosts.len() {
                return Err(Error::Input(format!("invalid ghost key index: {index}")));
            }
            ghosts[index] = Some(ghost);
        }
    }

    Ok(ghosts)
}

pub fn normalize_asset_id(asset_id: &str) -> Result<String, Error> {
    if Uuid::parse_str(asset_id).is_ok() {
        return Ok(hex::encode(hash256(asset_id.as_bytes())));
    }
    decode_fixed_hex(asset_id, 32, "asset_id")?;
    Ok(asset_id.to_lowercase())
}

pub fn derive_ghost_public_key(
    private_key: &[u8; 32],
    public_view_key: &[u8; 32],
    public_spend_key: &[u8; 32],
    index: u64,
) -> Result<[u8; 32], Error> {
    let shared = key_mult_pub_priv(public_view_key, private_key)?;
    let x = hash_scalar(&shared, index);
    let spend = CompressedEdwardsY(*public_spend_key)
        .decompress()
        .ok_or_else(|| Error::Input("invalid public spend key".to_string()))?;
    Ok((spend + EdwardsPoint::mul_base(&x)).compress().to_bytes())
}

pub fn sign_safe_transaction(
    tx: &SafeTransaction,
    views: &[String],
    private_key: &str,
) -> Result<String, Error> {
    sign_safe_transaction_with_index(tx, views, private_key, false, 0)
}

pub fn sign_safe_transaction_with_index(
    tx: &SafeTransaction,
    views: &[String],
    private_key: &str,
    is_sum_already: bool,
    signer_index: u16,
) -> Result<String, Error> {
    if views.len() != tx.inputs.len() {
        return Err(Error::Input(format!(
            "invalid view keys count {} != {}",
            views.len(),
            tx.inputs.len()
        )));
    }

    let raw = encode_unsigned_safe_transaction(tx)?;
    let raw_bytes = hex::decode(raw)?;
    let msg = blake3::hash(&raw_bytes);
    let y = spend_scalar(private_key, is_sum_already)?;

    let mut signature_map = Vec::with_capacity(tx.inputs.len());
    for view in views {
        let x = canonical_scalar_hex(view, "view key")?;
        let t = x + y;
        let key = t.to_bytes();
        let sig = ed25519_sign(msg.as_bytes(), &key)?;
        let mut signatures = BTreeMap::new();
        signatures.insert(signer_index, hex::encode(sig));
        signature_map.push(signatures);
    }

    encode_safe_transaction_with_signatures(tx, &signature_map)
}

pub fn estimate_storage_cost(extra: &[u8]) -> String {
    let step = (extra.len() / EXTRA_SIZE_STORAGE_STEP + 1) as u128;
    format_units(step * EXTRA_STORAGE_PRICE_STEP_FIXED)
}

fn validate_extra_for_recipients(
    extra: &[u8],
    recipients: &[SafeTransactionRecipient],
) -> Result<(), Error> {
    if extra.len() <= EXTRA_SIZE_GENERAL_LIMIT {
        return Ok(());
    }
    let first = recipients
        .first()
        .ok_or_else(|| Error::Input("empty safe transaction recipients".to_string()))?;
    if extra.len() > EXTRA_SIZE_STORAGE_CAPACITY {
        return Err(Error::Input(format!(
            "extra data is too long: {}",
            extra.len()
        )));
    }
    if matches!(first, SafeTransactionRecipient::Withdrawal { .. }) {
        return Err(Error::Input(
            "storage extra requires a script recipient".to_string(),
        ));
    }
    let cost = parse_units(&estimate_storage_cost(extra))?;
    let amount = parse_units(first.amount())?;
    if cost > amount {
        return Err(Error::Input(
            "first recipient amount is below storage cost".to_string(),
        ));
    }
    Ok(())
}

fn output_asset(output: &UtxoOutput) -> Result<String, Error> {
    if let Some(asset) = output
        .kernel_asset_id
        .as_ref()
        .filter(|asset| !asset.is_empty())
    {
        decode_fixed_hex(asset, 32, "kernel_asset_id")?;
        return Ok(asset.to_lowercase());
    }

    let asset_id = output
        .asset_id
        .as_ref()
        .ok_or_else(|| Error::Input("output is missing asset_id".to_string()))?;
    if decode_fixed_hex(asset_id, 32, "asset_id").is_ok() {
        return Ok(asset_id.to_lowercase());
    }
    if Uuid::parse_str(asset_id).is_ok() {
        return Ok(hex::encode(hash256(asset_id.as_bytes())));
    }
    Err(Error::Input(format!("invalid output asset: {asset_id}")))
}

fn validate_totals_and_change(
    utxos: &[UtxoOutput],
    recipients: &[SafeTransactionRecipient],
) -> Result<String, Error> {
    let total_input = utxos.iter().try_fold(0u128, |total, output| {
        total
            .checked_add(output_amount(output)?)
            .ok_or_else(|| Error::Input("input amount overflow".to_string()))
    })?;
    let total_output = recipients.iter().try_fold(0u128, |total, recipient| {
        total
            .checked_add(parse_units(recipient.amount())?)
            .ok_or_else(|| Error::Input("output amount overflow".to_string()))
    })?;
    if total_input < total_output {
        return Err(Error::Input(format!(
            "insufficient outputs {} < {}",
            format_units(total_input),
            format_units(total_output)
        )));
    }
    Ok(format_units(total_input - total_output))
}

fn output_amount(output: &UtxoOutput) -> Result<u128, Error> {
    let amount = output
        .amount
        .as_ref()
        .ok_or_else(|| Error::Input("output is missing amount".to_string()))?;
    parse_units(amount)
}

fn expect_one_transaction(
    mut transactions: Vec<TransactionView>,
) -> Result<TransactionView, Error> {
    if transactions.len() != 1 {
        return Err(Error::DataNotFound(format!(
            "expected one transaction, got {}",
            transactions.len()
        )));
    }
    Ok(transactions.remove(0))
}

fn spend_scalar(private_key: &str, is_sum_already: bool) -> Result<Scalar, Error> {
    let key = hex::decode(private_key)?;
    if key.len() < 32 {
        return Err(Error::Input("invalid spend private key length".to_string()));
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&key[..32]);

    if is_sum_already {
        return canonical_scalar(seed, "spend private key");
    }

    let digest = Sha512::digest(seed);
    let mut clamped = [0u8; 32];
    clamped.copy_from_slice(&digest[..32]);
    clamped[0] &= 248;
    clamped[31] &= 63;
    clamped[31] |= 64;
    Ok(Scalar::from_bytes_mod_order(clamped))
}

fn spend_private_key_bytes(private_key: &str) -> Result<[u8; 32], Error> {
    let key = hex::decode(private_key)?;
    if key.len() < 32 {
        return Err(Error::Input("invalid spend private key length".to_string()));
    }
    fixed_bytes32(&key[..32], "spend private key")
}

fn new_key_from_seed(seed: &[u8; 64]) -> [u8; 32] {
    Scalar::from_bytes_mod_order_wide(seed).to_bytes()
}

fn key_mult_pub_priv(public_key: &[u8; 32], private_key: &[u8; 32]) -> Result<[u8; 32], Error> {
    let point = CompressedEdwardsY(*public_key)
        .decompress()
        .ok_or_else(|| Error::Input("invalid public key".to_string()))?;
    let scalar = canonical_scalar(*private_key, "private scalar")?;
    Ok((point * scalar).compress().to_bytes())
}

fn hash_scalar(key: &[u8; 32], index: u64) -> Scalar {
    let index = put_uvarint(index);
    let hash = blake3_hash_many(&[key, &index]);
    let hash2 = blake3_hash_many(&[&hash]);
    let mut src = bytes64(&hash, &hash2);
    let scalar = Scalar::from_bytes_mod_order_wide(&src);

    let hash = blake3_hash_many(&[&scalar.to_bytes()]);
    let hash2 = blake3_hash_many(&[&hash]);
    src = bytes64(&hash, &hash2);
    Scalar::from_bytes_mod_order_wide(&src)
}

fn blake3_hash_many(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

fn bytes64(first: &[u8], second: &[u8]) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(first);
    bytes[32..].copy_from_slice(second);
    bytes
}

fn fixed_bytes32(bytes: &[u8], name: &str) -> Result<[u8; 32], Error> {
    if bytes.len() != 32 {
        return Err(Error::Input(format!(
            "invalid {name} length: {} != 32",
            bytes.len()
        )));
    }
    let mut fixed = [0u8; 32];
    fixed.copy_from_slice(bytes);
    Ok(fixed)
}

fn integer_to_bytes_without_zero(value: u128) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }
    let bytes = value.to_be_bytes();
    bytes
        .iter()
        .position(|b| *b != 0)
        .map(|i| bytes[i..].to_vec())
        .unwrap_or_default()
}

fn put_uvarint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value >= 0x80 {
        bytes.push((value as u8) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}

fn ed25519_sign(msg: &[u8], key: &[u8; 32]) -> Result<[u8; 64], Error> {
    let digest1 = Sha512::digest(key);
    let mut h = Sha512::new();
    h.update(&digest1[32..]);
    h.update(msg);
    let message_digest: [u8; 64] = h.finalize().into();
    let z = Scalar::from_bytes_mod_order_wide(&message_digest);
    let r = EdwardsPoint::mul_base(&z).compress().to_bytes();

    let public = public_from_private_scalar(key)?;
    let mut hram = Sha512::new();
    hram.update(r);
    hram.update(public);
    hram.update(msg);
    let hram_digest: [u8; 64] = hram.finalize().into();
    let x = Scalar::from_bytes_mod_order_wide(&hram_digest);
    let y = canonical_scalar(*key, "private scalar")?;
    let s = x * y + z;

    let mut sig = [0u8; 64];
    sig[..32].copy_from_slice(&r);
    sig[32..].copy_from_slice(&s.to_bytes());
    Ok(sig)
}

fn public_from_private_scalar(key: &[u8; 32]) -> Result<[u8; 32], Error> {
    let scalar = canonical_scalar(*key, "private scalar")?;
    Ok(EdwardsPoint::mul_base(&scalar).compress().to_bytes())
}

fn canonical_scalar_hex(value: &str, name: &str) -> Result<Scalar, Error> {
    let bytes = decode_fixed_hex(value, 32, name)?;
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&bytes);
    canonical_scalar(scalar_bytes, name)
}

fn canonical_scalar(bytes: [u8; 32], name: &str) -> Result<Scalar, Error> {
    let scalar = Scalar::from_canonical_bytes(bytes);
    if bool::from(scalar.is_some()) {
        Ok(scalar.unwrap())
    } else {
        Err(Error::Input(format!("invalid canonical scalar: {name}")))
    }
}

fn decode_fixed_hex(value: &str, len: usize, name: &str) -> Result<Vec<u8>, Error> {
    let bytes = hex::decode(value)?;
    if bytes.len() != len {
        return Err(Error::Input(format!(
            "invalid {name} length: {} != {len}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn parse_units(amount: &str) -> Result<u128, Error> {
    let amount = amount.trim();
    if amount.is_empty() || amount.starts_with('-') {
        return Err(Error::Input(format!("invalid amount: {amount}")));
    }

    let mut parts = amount.split('.');
    let int_part = parts.next().unwrap_or_default();
    let frac_part = parts.next().unwrap_or_default();
    if parts.next().is_some() {
        return Err(Error::Input(format!("invalid amount: {amount}")));
    }
    if !int_part.chars().all(|c| c.is_ascii_digit()) && !int_part.is_empty() {
        return Err(Error::Input(format!("invalid amount: {amount}")));
    }
    if !frac_part.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::Input(format!("invalid amount: {amount}")));
    }

    let int_value = if int_part.is_empty() {
        0
    } else {
        int_part
            .parse::<u128>()
            .map_err(|_| Error::Input(format!("invalid amount: {amount}")))?
    };
    let multiplier = 10u128.pow(DECIMALS as u32);
    let mut value = int_value
        .checked_mul(multiplier)
        .ok_or_else(|| Error::Input(format!("amount overflow: {amount}")))?;

    let mut frac = frac_part.as_bytes().to_vec();
    if frac.len() > DECIMALS {
        frac.truncate(DECIMALS);
    }
    while frac.len() < DECIMALS {
        frac.push(b'0');
    }
    if !frac.is_empty() {
        let frac_str = std::str::from_utf8(&frac)
            .map_err(|_| Error::Input(format!("invalid amount: {amount}")))?;
        value = value
            .checked_add(
                frac_str
                    .parse::<u128>()
                    .map_err(|_| Error::Input(format!("invalid amount: {amount}")))?,
            )
            .ok_or_else(|| Error::Input(format!("amount overflow: {amount}")))?;
    }
    Ok(value)
}

fn format_units(value: u128) -> String {
    let multiplier = 10u128.pow(DECIMALS as u32);
    let int = value / multiplier;
    let frac = value % multiplier;
    if frac == 0 {
        return int.to_string();
    }
    let frac = format!("{frac:0DECIMALS$}");
    format!("{int}.{}", frac.trim_end_matches('0'))
}

fn integer_to_bytes(value: u128) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }
    let bytes = value.to_be_bytes();
    bytes
        .iter()
        .position(|b| *b != 0)
        .map(|i| bytes[i..].to_vec())
        .unwrap_or_else(|| vec![0])
}

fn integer_from_bytes(bytes: &[u8]) -> Result<u128, Error> {
    if bytes.len() > 16 {
        return Err(Error::Input(format!(
            "integer overflow: {} bytes",
            bytes.len()
        )));
    }
    let mut value = 0u128;
    for byte in bytes {
        value = (value << 8) + (*byte as u128);
    }
    Ok(value)
}

struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn into_inner(self) -> Vec<u8> {
        self.bytes
    }

    fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn write_int(&mut self, value: usize) -> Result<(), Error> {
        if value > u16::MAX as usize {
            return Err(Error::Input(format!("integer overflow: {value}")));
        }
        self.write_u16(value as u16);
        Ok(())
    }

    fn write_u16(&mut self, value: u16) {
        self.write(&value.to_be_bytes());
    }

    fn write_u32(&mut self, value: usize) -> Result<(), Error> {
        if value > u32::MAX as usize {
            return Err(Error::Input(format!("integer overflow: {value}")));
        }
        self.write(&(value as u32).to_be_bytes());
        Ok(())
    }

    fn write_integer(&mut self, value: u128) -> Result<(), Error> {
        let bytes = integer_to_bytes(value);
        self.write_int(bytes.len())?;
        self.write(&bytes);
        Ok(())
    }

    fn encode_input(&mut self, input: &SafeTransactionInput) -> Result<(), Error> {
        self.write(&decode_fixed_hex(&input.hash, 32, "input hash")?);
        self.write_u16(input.index);
        self.write_int(input.genesis.len())?;
        self.write(&input.genesis);
        self.write(&EMPTY);
        self.write(&EMPTY);
        Ok(())
    }

    fn encode_output(&mut self, output: &SafeTransactionOutput) -> Result<(), Error> {
        self.write(&[0x00, output.output_type]);
        self.write_integer(parse_units(&output.amount)?)?;

        self.write_int(output.keys.len())?;
        for key in &output.keys {
            self.write(&decode_fixed_hex(key, 32, "output key")?);
        }

        match &output.mask {
            Some(mask) => self.write(&decode_fixed_hex(mask, 32, "output mask")?),
            None => self.write(&[0u8; 32]),
        }

        let script = if output.script.is_empty() {
            Vec::new()
        } else {
            hex::decode(&output.script)?
        };
        self.write_int(script.len())?;
        self.write(&script);

        if let Some(withdrawal) = &output.withdrawal {
            self.write(&MAGIC);
            self.write_int(withdrawal.address.len())?;
            self.write(withdrawal.address.as_bytes());
            self.write_int(withdrawal.tag.len())?;
            self.write(withdrawal.tag.as_bytes());
        } else {
            self.write(&EMPTY);
        }
        Ok(())
    }

    fn encode_signature(&mut self, signatures: &BTreeMap<u16, String>) -> Result<(), Error> {
        self.write_int(signatures.len())?;
        for (index, signature) in signatures {
            self.write_u16(*index);
            self.write(&decode_fixed_hex(signature, 64, "signature")?);
        }
        Ok(())
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], Error> {
        if self.offset + len > self.bytes.len() {
            return Err(Error::Input("unexpected end of transaction".to_string()));
        }
        let start = self.offset;
        self.offset += len;
        Ok(&self.bytes[start..self.offset])
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn decode_input(&mut self) -> Result<SafeTransactionInput, Error> {
        let hash = hex::encode(self.read_exact(32)?);
        let index = self.read_u16()?;

        let len_genesis = self.read_u16()? as usize;
        let genesis = self.read_exact(len_genesis)?.to_vec();

        let deposit_prefix = self.read_exact(2)?;
        if deposit_prefix != EMPTY {
            if deposit_prefix == MAGIC {
                return Err(Error::Input(
                    "safe transaction deposit inputs are not supported".to_string(),
                ));
            }
            return Err(Error::Input("invalid deposit prefix".to_string()));
        }

        let mint_prefix = self.read_exact(2)?;
        if mint_prefix != EMPTY {
            if mint_prefix == MAGIC {
                return Err(Error::Input(
                    "safe transaction mint inputs are not supported".to_string(),
                ));
            }
            return Err(Error::Input("invalid mint prefix".to_string()));
        }

        Ok(SafeTransactionInput {
            hash,
            index,
            genesis,
        })
    }

    fn decode_output(&mut self) -> Result<SafeTransactionOutput, Error> {
        let marker = self.read_u8()?;
        if marker != 0 {
            return Err(Error::Input(format!("invalid output marker: {marker}")));
        }
        let output_type = self.read_u8()?;
        let amount_len = self.read_u16()? as usize;
        let amount = format_units(integer_from_bytes(self.read_exact(amount_len)?)?);

        let keys_len = self.read_u16()? as usize;
        let mut keys = Vec::with_capacity(keys_len);
        for _ in 0..keys_len {
            keys.push(hex::encode(self.read_exact(32)?));
        }

        let mask = Some(hex::encode(self.read_exact(32)?));
        let script_len = self.read_u16()? as usize;
        let script = hex::encode(self.read_exact(script_len)?);

        let prefix = self.read_exact(2)?;
        let withdrawal = if prefix == MAGIC {
            let address_len = self.read_u16()? as usize;
            let address = String::from_utf8(self.read_exact(address_len)?.to_vec())
                .map_err(|e| Error::Input(format!("invalid withdrawal address: {e}")))?;
            let tag_len = self.read_u16()? as usize;
            let tag = String::from_utf8(self.read_exact(tag_len)?.to_vec())
                .map_err(|e| Error::Input(format!("invalid withdrawal tag: {e}")))?;
            Some(SafeWithdrawalData { address, tag })
        } else if prefix == EMPTY {
            None
        } else {
            return Err(Error::Input("invalid withdrawal prefix".to_string()));
        };

        Ok(SafeTransactionOutput {
            output_type,
            amount,
            keys,
            mask,
            script,
            withdrawal,
        })
    }

    fn decode_signature(&mut self) -> Result<BTreeMap<u16, String>, Error> {
        let len = self.read_u16()? as usize;
        let mut signatures = BTreeMap::new();
        for _ in 0..len {
            let index = self.read_u16()?;
            let signature = hex::encode(self.read_exact(64)?);
            signatures.insert(index, signature);
        }
        Ok(signatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_RAW: &str = "77770005111111111111111111111111111111111111111111111111111111111111111100010000000000000000000000000000000000000000000000000000000000000000000100000000000000010000000402faf0800001333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220003fffe010000000144444444444444444444444444444444444444444444444444444444444444440000000568656c6c6f0000";
    const EXPECTED_SIGNED: &str = "77770005111111111111111111111111111111111111111111111111111111111111111100010000000000000000000000000000000000000000000000000000000000000000000100000000000000010000000402faf0800001333333333333333333333333333333333333333333333333333333333333333322222222222222222222222222222222222222222222222222222222222222220003fffe010000000144444444444444444444444444444444444444444444444444444444444444440000000568656c6c6f000100010000bf095515491924399f0f2e1aff1c3f19dc22d77592e71a3b78abcbf3c5ea1e90113734130ee994cdb78005946d04de581e38276a9d39294dd1ab0c86cdbdad0c";

    fn fixture_tx() -> SafeTransaction {
        let utxo = UtxoOutput {
            output_id: "output-id".to_string(),
            transaction_hash: Some("00".repeat(32)),
            output_index: Some(1),
            kernel_asset_id: Some("11".repeat(32)),
            amount: Some("1".to_string()),
            state: Some("unspent".to_string()),
            ..Default::default()
        };
        let mix_address = MixAddress {
            version: 2,
            threshold: 1,
            uuid_members: vec!["67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string()],
            xin_members: Vec::new(),
        };
        let recipient = SafeTransactionRecipient::mix_address(mix_address, "0.5");
        let ghost = GhostKeys {
            key_type: "ghost_key".to_string(),
            mask: "22".repeat(32),
            keys: vec!["33".repeat(32)],
        };
        build_safe_transaction(
            &[utxo],
            &[recipient],
            &[Some(ghost)],
            b"hello".to_vec(),
            vec!["44".repeat(32)],
        )
        .expect("build tx")
    }

    #[test]
    fn test_encode_script() {
        assert_eq!(encode_script(1).unwrap(), "fffe01");
        assert_eq!(encode_script(64).unwrap(), "fffe40");
        assert!(encode_script(65).is_err());
    }

    #[test]
    fn test_encode_safe_transaction_fixture() {
        let tx = fixture_tx();
        assert_eq!(encode_safe_transaction(&tx).unwrap(), EXPECTED_RAW);
    }

    #[test]
    fn test_decode_safe_transaction_fixture() {
        let tx = decode_safe_transaction(EXPECTED_RAW).expect("decode");
        assert_eq!(tx.version, TX_VERSION_HASH_SIGNATURE);
        assert_eq!(tx.asset, "11".repeat(32));
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.inputs[0].index, 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].amount, "0.5");
        assert_eq!(tx.outputs[0].script, "fffe01");
        assert_eq!(tx.references, vec!["44".repeat(32)]);
        assert_eq!(tx.extra, b"hello".to_vec());
        assert_eq!(encode_safe_transaction(&tx).unwrap(), EXPECTED_RAW);
    }

    #[test]
    fn test_sign_safe_transaction_fixture() {
        let tx = fixture_tx();
        let views = vec!["00".repeat(32)];
        let private_key = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let signed = sign_safe_transaction(&tx, &views, private_key).expect("sign");
        assert_eq!(signed, EXPECTED_SIGNED);
    }

    #[test]
    fn test_amount_codec() {
        assert_eq!(parse_units("0.5").unwrap(), 50_000_000);
        assert_eq!(parse_units("1.234567899").unwrap(), 123_456_789);
        assert_eq!(format_units(50_000_000), "0.5");
        assert_eq!(format_units(100_000_000), "1");
    }

    #[test]
    fn test_estimate_storage_cost() {
        assert_eq!(estimate_storage_cost(&vec![0; 1024]), "0.0002");
        assert_eq!(estimate_storage_cost(&vec![0; 1025]), "0.0002");
    }

    #[test]
    fn test_unspent_output_selection_adds_change() {
        let outputs = vec![
            UtxoOutput {
                output_id: "output-1".to_string(),
                amount: Some("0.3".to_string()),
                ..Default::default()
            },
            UtxoOutput {
                output_id: "output-2".to_string(),
                amount: Some("0.4".to_string()),
                ..Default::default()
            },
        ];
        let recipient = SafeTransactionRecipient::mix_address(
            MixAddress {
                version: 2,
                threshold: 1,
                uuid_members: vec!["67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string()],
                xin_members: Vec::new(),
            },
            "0.5",
        );

        let (count, change) =
            get_unspent_outputs_for_recipients(&outputs, &[recipient]).expect("select");
        assert_eq!(count, 2);
        assert_eq!(change, "0.2");
    }

    #[tokio::test]
    async fn test_mainnet_ghost_key_fixture() {
        let safe_user = SafeUser::new(
            "67a87828-18f5-46a1-b6cc-c72a97a77c43".to_string(),
            "session-id".to_string(),
            "00".repeat(32),
            "11".repeat(32),
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".to_string(),
        );
        let mix_address = MixAddress::new_mainnet(
            vec![
                "XINSwYaJPnKiwBWqXm4i3e3My9GKguReMRyB1sRSexeHcQ7V66RWsicAiR2dokcQ5kiJsfY5QbEjTcqRQRCxkEyENBaz4AeB"
                    .to_string(),
            ],
            1,
        )
        .expect("mainnet mix address");
        let recipient = SafeTransactionRecipient::mix_address(mix_address, "1");

        let ghosts = request_ghost_recipients_with_trace_id(&[recipient], "trace-id", &safe_user)
            .await
            .expect("ghosts");
        let ghost = ghosts[0].as_ref().expect("ghost");
        assert_eq!(
            ghost.mask,
            "1790b187b0951b2bc957a5986ecb03353c67f0a84968fc38aa2622332acb179d"
        );
        assert_eq!(
            ghost.keys,
            vec!["95a8bce4f167124fadb226b5e5b3cf6860fe95d9786596eebb7bce2aa9176eac"]
        );
    }
}
