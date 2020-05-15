// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::status_code::Ctap2StatusCode;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::TryFrom;
use crypto::{ecdh, ecdsa};

// https://www.w3.org/TR/webauthn/#dictdef-publickeycredentialrpentity
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct PublicKeyCredentialRpEntity {
    pub rp_id: String,
    pub rp_name: Option<String>,
    pub rp_icon: Option<String>,
}

impl TryFrom<&cbor::Value> for PublicKeyCredentialRpEntity {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let rp_map = read_map(cbor_value)?;
        let rp_id = read_text_string(ok_or_missing(rp_map.get(&cbor_text!("id")))?)?;
        let rp_name = rp_map
            .get(&cbor_text!("name"))
            .map(read_text_string)
            .transpose()?;
        let rp_icon = rp_map
            .get(&cbor_text!("icon"))
            .map(read_text_string)
            .transpose()?;
        Ok(Self {
            rp_id,
            rp_name,
            rp_icon,
        })
    }
}

// https://www.w3.org/TR/webauthn/#dictdef-publickeycredentialuserentity
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct PublicKeyCredentialUserEntity {
    pub user_id: Vec<u8>,
    pub user_name: Option<String>,
    pub user_display_name: Option<String>,
    pub user_icon: Option<String>,
}

impl TryFrom<&cbor::Value> for PublicKeyCredentialUserEntity {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let user_map = read_map(cbor_value)?;
        let user_id = read_byte_string(ok_or_missing(user_map.get(&cbor_text!("id")))?)?;
        let user_name = user_map
            .get(&cbor_text!("name"))
            .map(read_text_string)
            .transpose()?;
        let user_display_name = user_map
            .get(&cbor_text!("displayName"))
            .map(read_text_string)
            .transpose()?;
        let user_icon = user_map
            .get(&cbor_text!("icon"))
            .map(read_text_string)
            .transpose()?;
        Ok(Self {
            user_id,
            user_name,
            user_display_name,
            user_icon,
        })
    }
}

impl From<PublicKeyCredentialUserEntity> for cbor::Value {
    fn from(entity: PublicKeyCredentialUserEntity) -> Self {
        cbor_map_options! {
            "id" => entity.user_id,
            "name" => entity.user_name,
            "displayName" => entity.user_display_name,
            "icon" => entity.user_icon,
        }
    }
}

// https://www.w3.org/TR/webauthn/#enumdef-publickeycredentialtype
#[derive(Clone, PartialEq)]
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug))]
pub enum PublicKeyCredentialType {
    PublicKey,
    // This is the default for all strings not covered above.
    // Unknown types should be ignored, instead of returning errors.
    Unknown,
}

impl From<PublicKeyCredentialType> for cbor::Value {
    fn from(cred_type: PublicKeyCredentialType) -> Self {
        match cred_type {
            PublicKeyCredentialType::PublicKey => "public-key",
            // We should never create this credential type.
            PublicKeyCredentialType::Unknown => "unknown",
        }
        .into()
    }
}

impl TryFrom<&cbor::Value> for PublicKeyCredentialType {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let cred_type_string = read_text_string(cbor_value)?;
        match &cred_type_string[..] {
            "public-key" => Ok(PublicKeyCredentialType::PublicKey),
            _ => Ok(PublicKeyCredentialType::Unknown),
        }
    }
}

// https://www.w3.org/TR/webauthn/#dictdef-publickeycredentialparameters
#[derive(PartialEq)]
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug))]
pub struct PublicKeyCredentialParameter {
    pub cred_type: PublicKeyCredentialType,
    pub alg: SignatureAlgorithm,
}

impl TryFrom<&cbor::Value> for PublicKeyCredentialParameter {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let cred_param_map = read_map(cbor_value)?;
        let cred_type = PublicKeyCredentialType::try_from(ok_or_missing(
            cred_param_map.get(&cbor_text!("type")),
        )?)?;
        let alg =
            SignatureAlgorithm::try_from(ok_or_missing(cred_param_map.get(&cbor_text!("alg")))?)?;
        Ok(Self { cred_type, alg })
    }
}

impl From<PublicKeyCredentialParameter> for cbor::Value {
    fn from(cred_param: PublicKeyCredentialParameter) -> Self {
        cbor_map_options! {
            "type" => cred_param.cred_type,
            "alg" => cred_param.alg as i64,
        }
    }
}

// https://www.w3.org/TR/webauthn/#enumdef-authenticatortransport
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub enum AuthenticatorTransport {
    Usb,
    Nfc,
    Ble,
    Internal,
}

impl From<AuthenticatorTransport> for cbor::Value {
    fn from(transport: AuthenticatorTransport) -> Self {
        match transport {
            AuthenticatorTransport::Usb => "usb",
            AuthenticatorTransport::Nfc => "nfc",
            AuthenticatorTransport::Ble => "ble",
            AuthenticatorTransport::Internal => "internal",
        }
        .into()
    }
}

impl TryFrom<&cbor::Value> for AuthenticatorTransport {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let transport_string = read_text_string(cbor_value)?;
        match &transport_string[..] {
            "usb" => Ok(AuthenticatorTransport::Usb),
            "nfc" => Ok(AuthenticatorTransport::Nfc),
            "ble" => Ok(AuthenticatorTransport::Ble),
            "internal" => Ok(AuthenticatorTransport::Internal),
            _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
        }
    }
}

// https://www.w3.org/TR/webauthn/#dictdef-publickeycredentialdescriptor
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct PublicKeyCredentialDescriptor {
    pub key_type: PublicKeyCredentialType,
    pub key_id: Vec<u8>,
    pub transports: Option<Vec<AuthenticatorTransport>>,
}

impl TryFrom<&cbor::Value> for PublicKeyCredentialDescriptor {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let cred_desc_map = read_map(cbor_value)?;
        let key_type = PublicKeyCredentialType::try_from(ok_or_missing(
            cred_desc_map.get(&cbor_text!("type")),
        )?)?;
        let key_id = read_byte_string(ok_or_missing(cred_desc_map.get(&cbor_text!("id")))?)?;
        let transports = match cred_desc_map.get(&cbor_text!("transports")) {
            Some(exclude_entry) => {
                let transport_vec = read_array(exclude_entry)?;
                let transports = transport_vec
                    .iter()
                    .map(AuthenticatorTransport::try_from)
                    .collect::<Result<Vec<AuthenticatorTransport>, Ctap2StatusCode>>(
                )?;
                Some(transports)
            }
            None => None,
        };
        Ok(Self {
            key_type,
            key_id,
            transports,
        })
    }
}

impl From<PublicKeyCredentialDescriptor> for cbor::Value {
    fn from(desc: PublicKeyCredentialDescriptor) -> Self {
        cbor_map_options! {
            "type" => desc.key_type,
            "id" => desc.key_id,
            "transports" => desc.transports.map(|vec| cbor_array_vec!(vec)),
        }
    }
}

#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct Extensions(BTreeMap<String, cbor::Value>);

impl TryFrom<&cbor::Value> for Extensions {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let mut extensions = BTreeMap::new();
        for (extension_key, extension_value) in read_map(cbor_value)? {
            if let cbor::KeyType::TextString(extension_key_string) = extension_key {
                extensions.insert(extension_key_string.to_string(), extension_value.clone());
            } else {
                return Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE);
            }
        }
        Ok(Extensions(extensions))
    }
}

impl From<Extensions> for cbor::Value {
    fn from(extensions: Extensions) -> Self {
        cbor_map_btree!(extensions
            .0
            .into_iter()
            .map(|(key, value)| (cbor_text!(key), value))
            .collect())
    }
}

impl Extensions {
    #[cfg(test)]
    pub fn new(extension_map: BTreeMap<String, cbor::Value>) -> Self {
        Extensions(extension_map)
    }

    pub fn has_make_credential_hmac_secret(&self) -> Result<bool, Ctap2StatusCode> {
        self.0
            .get("hmac-secret")
            .map(read_bool)
            .unwrap_or(Ok(false))
    }

    pub fn get_assertion_hmac_secret(
        &self,
    ) -> Option<Result<GetAssertionHmacSecretInput, Ctap2StatusCode>> {
        self.0
            .get("hmac-secret")
            .map(GetAssertionHmacSecretInput::try_from)
    }
}

#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct GetAssertionHmacSecretInput {
    pub key_agreement: CoseKey,
    pub salt_enc: Vec<u8>,
    pub salt_auth: Vec<u8>,
}

impl TryFrom<&cbor::Value> for GetAssertionHmacSecretInput {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let input_map = read_map(cbor_value)?;
        let cose_key = read_map(ok_or_missing(input_map.get(&cbor_unsigned!(1)))?)?;
        let salt_enc = read_byte_string(ok_or_missing(input_map.get(&cbor_unsigned!(2)))?)?;
        let salt_auth = read_byte_string(ok_or_missing(input_map.get(&cbor_unsigned!(3)))?)?;
        Ok(Self {
            key_agreement: CoseKey(cose_key.clone()),
            salt_enc,
            salt_auth,
        })
    }
}

#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct GetAssertionHmacSecretOutput(Vec<u8>);

impl From<GetAssertionHmacSecretOutput> for cbor::Value {
    fn from(message: GetAssertionHmacSecretOutput) -> cbor::Value {
        cbor_bytes!(message.0)
    }
}

impl TryFrom<&cbor::Value> for GetAssertionHmacSecretOutput {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        Ok(GetAssertionHmacSecretOutput(read_byte_string(cbor_value)?))
    }
}

// Even though options are optional, we can use the default if not present.
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct MakeCredentialOptions {
    pub rk: bool,
    pub uv: bool,
}

impl TryFrom<&cbor::Value> for MakeCredentialOptions {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let options_map = read_map(cbor_value)?;
        let rk = match options_map.get(&cbor_text!("rk")) {
            Some(options_entry) => read_bool(options_entry)?,
            None => false,
        };
        if let Some(options_entry) = options_map.get(&cbor_text!("up")) {
            if !read_bool(options_entry)? {
                return Err(Ctap2StatusCode::CTAP2_ERR_INVALID_OPTION);
            }
        }
        let uv = match options_map.get(&cbor_text!("uv")) {
            Some(options_entry) => read_bool(options_entry)?,
            None => false,
        };
        Ok(Self { rk, uv })
    }
}

#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct GetAssertionOptions {
    pub up: bool,
    pub uv: bool,
}

impl TryFrom<&cbor::Value> for GetAssertionOptions {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let options_map = read_map(cbor_value)?;
        if let Some(options_entry) = options_map.get(&cbor_text!("rk")) {
            // This is only for returning the correct status code.
            read_bool(options_entry)?;
            return Err(Ctap2StatusCode::CTAP2_ERR_INVALID_OPTION);
        }
        let up = match options_map.get(&cbor_text!("up")) {
            Some(options_entry) => read_bool(options_entry)?,
            None => true,
        };
        let uv = match options_map.get(&cbor_text!("uv")) {
            Some(options_entry) => read_bool(options_entry)?,
            None => false,
        };
        Ok(Self { up, uv })
    }
}

// https://www.w3.org/TR/webauthn/#packed-attestation
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug))]
pub struct PackedAttestationStatement {
    pub alg: i64,
    pub sig: Vec<u8>,
    pub x5c: Option<Vec<Vec<u8>>>,
    pub ecdaa_key_id: Option<Vec<u8>>,
}

impl From<PackedAttestationStatement> for cbor::Value {
    fn from(att_stmt: PackedAttestationStatement) -> Self {
        cbor_map_options! {
            "alg" => att_stmt.alg,
            "sig" => att_stmt.sig,
            "x5c" => att_stmt.x5c.map(|x| cbor_array_vec!(x)),
            "ecdaaKeyId" => att_stmt.ecdaa_key_id,
        }
    }
}

#[derive(PartialEq)]
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug))]
pub enum SignatureAlgorithm {
    ES256 = ecdsa::PubKey::ES256_ALGORITHM as isize,
    // This is the default for all numbers not covered above.
    // Unknown types should be ignored, instead of returning errors.
    Unknown = 0,
}

impl TryFrom<&cbor::Value> for SignatureAlgorithm {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        match read_integer(cbor_value)? {
            ecdsa::PubKey::ES256_ALGORITHM => Ok(SignatureAlgorithm::ES256),
            _ => Ok(SignatureAlgorithm::Unknown),
        }
    }
}

// https://www.w3.org/TR/webauthn/#public-key-credential-source
//
// Note that we only use the WebAuthn definition as an example. This data-structure is not specified
// by FIDO. In particular we may choose how we serialize and deserialize it.
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug))]
pub struct PublicKeyCredentialSource {
    // TODO function to convert to / from Vec<u8>
    pub key_type: PublicKeyCredentialType,
    pub credential_id: Vec<u8>,
    pub private_key: ecdsa::SecKey, // TODO(kaczmarczyck) open for other algorithms
    pub rp_id: String,
    pub user_handle: Vec<u8>, // not optional, but nullable
    pub other_ui: Option<String>,
    pub cred_random: Option<Vec<u8>>,
}

// We serialize credentials for the persistent storage using CBOR maps. Each field of a credential
// is associated with a unique tag, implemented with a CBOR unsigned key.
enum PublicKeyCredentialSourceField {
    CredentialId = 0,
    PrivateKey = 1,
    RpId = 2,
    UserHandle = 3,
    OtherUi = 4,
    CredRandom = 5,
    // When a field is removed, its tag should be reserved and not used for new fields. We document
    // those reserved tags below.
    // Reserved tags: none.
}

impl From<PublicKeyCredentialSourceField> for cbor::KeyType {
    fn from(field: PublicKeyCredentialSourceField) -> cbor::KeyType {
        (field as u64).into()
    }
}

impl From<PublicKeyCredentialSource> for cbor::Value {
    fn from(credential: PublicKeyCredentialSource) -> cbor::Value {
        use PublicKeyCredentialSourceField::*;
        let mut private_key = [0u8; 32];
        credential.private_key.to_bytes(&mut private_key);
        cbor_map_options! {
            CredentialId => Some(credential.credential_id),
            PrivateKey => Some(private_key.to_vec()),
            RpId => Some(credential.rp_id),
            UserHandle => Some(credential.user_handle),
            OtherUi => credential.other_ui,
            CredRandom => credential.cred_random
        }
    }
}

impl TryFrom<cbor::Value> for PublicKeyCredentialSource {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: cbor::Value) -> Result<Self, Ctap2StatusCode> {
        use PublicKeyCredentialSourceField::*;
        let mut map = extract_map(cbor_value)?;
        let credential_id = extract_byte_string(ok_or_missing(map.remove(&CredentialId.into()))?)?;
        let private_key = extract_byte_string(ok_or_missing(map.remove(&PrivateKey.into()))?)?;
        if private_key.len() != 32 {
            return Err(Ctap2StatusCode::CTAP2_ERR_INVALID_CBOR);
        }
        let private_key = ecdsa::SecKey::from_bytes(array_ref!(private_key, 0, 32))
            .ok_or(Ctap2StatusCode::CTAP2_ERR_INVALID_CBOR)?;
        let rp_id = extract_text_string(ok_or_missing(map.remove(&RpId.into()))?)?;
        let user_handle = extract_byte_string(ok_or_missing(map.remove(&UserHandle.into()))?)?;
        let other_ui = map
            .remove(&OtherUi.into())
            .map(extract_text_string)
            .transpose()?;
        let cred_random = map
            .remove(&CredRandom.into())
            .map(extract_byte_string)
            .transpose()?;
        // We don't return whether there were unknown fields in the CBOR value. This means that
        // deserialization is not injective. In particular deserialization is only an inverse of
        // serialization at a given version of OpenSK. This is not a problem because:
        // 1. When a field is deprecated, its tag is reserved and never reused in future versions,
        //    including to be reintroduced with the same semantics. In other words, removing a field
        //    is permanent.
        // 2. OpenSK is never used with a more recent version of the storage. In particular, OpenSK
        //    is never rolled-back.
        // As a consequence, the unknown fields are only reserved fields and don't need to be
        // preserved.
        Ok(PublicKeyCredentialSource {
            key_type: PublicKeyCredentialType::PublicKey,
            credential_id,
            private_key,
            rp_id,
            user_handle,
            other_ui,
            cred_random,
        })
    }
}

// TODO(kaczmarczyck) we could decide to split this data type up
// It depends on the algorithm though, I think.
// So before creating a mess, this is my workaround.
#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub struct CoseKey(pub BTreeMap<cbor::KeyType, cbor::Value>);

// This is the algorithm specifier that is supposed to be used in a COSE key
// map. The CTAP specification says -25 which represents ECDH-ES + HKDF-256
// here: https://www.iana.org/assignments/cose/cose.xhtml#algorithms
// In fact, this is just used for compatibility with older specification versions.
const ECDH_ALGORITHM: i64 = -25;
// This is the identifier used by OpenSSH. To be compatible, we accept both.
const ES256_ALGORITHM: i64 = -7;
const EC2_KEY_TYPE: i64 = 2;
const P_256_CURVE: i64 = 1;

impl From<ecdh::PubKey> for CoseKey {
    fn from(pk: ecdh::PubKey) -> Self {
        let mut x_bytes = [0; ecdh::NBYTES];
        let mut y_bytes = [0; ecdh::NBYTES];
        pk.to_coordinates(&mut x_bytes, &mut y_bytes);
        let x_byte_cbor: cbor::Value = cbor_bytes_lit!(&x_bytes);
        let y_byte_cbor: cbor::Value = cbor_bytes_lit!(&y_bytes);
        // TODO(kaczmarczyck) do not write optional parameters, spec is unclear
        let cose_cbor_value = cbor_map_options! {
            1 => EC2_KEY_TYPE,
            3 => ECDH_ALGORITHM,
            -1 => P_256_CURVE,
            -2 => x_byte_cbor,
            -3 => y_byte_cbor,
        };
        if let cbor::Value::Map(cose_map) = cose_cbor_value {
            CoseKey(cose_map)
        } else {
            unreachable!();
        }
    }
}

impl TryFrom<CoseKey> for ecdh::PubKey {
    type Error = Ctap2StatusCode;

    fn try_from(cose_key: CoseKey) -> Result<Self, Ctap2StatusCode> {
        let key_type = read_integer(ok_or_missing(cose_key.0.get(&cbor_int!(1)))?)?;
        if key_type != EC2_KEY_TYPE {
            return Err(Ctap2StatusCode::CTAP2_ERR_UNSUPPORTED_ALGORITHM);
        }
        let algorithm = read_integer(ok_or_missing(cose_key.0.get(&cbor_int!(3)))?)?;
        if algorithm != ECDH_ALGORITHM && algorithm != ES256_ALGORITHM {
            return Err(Ctap2StatusCode::CTAP2_ERR_UNSUPPORTED_ALGORITHM);
        }
        let curve = read_integer(ok_or_missing(cose_key.0.get(&cbor_int!(-1)))?)?;
        if curve != P_256_CURVE {
            return Err(Ctap2StatusCode::CTAP2_ERR_UNSUPPORTED_ALGORITHM);
        }
        let x_bytes = read_byte_string(ok_or_missing(cose_key.0.get(&cbor_int!(-2)))?)?;
        if x_bytes.len() != ecdh::NBYTES {
            return Err(Ctap2StatusCode::CTAP1_ERR_INVALID_PARAMETER);
        }
        let y_bytes = read_byte_string(ok_or_missing(cose_key.0.get(&cbor_int!(-3)))?)?;
        if y_bytes.len() != ecdh::NBYTES {
            return Err(Ctap2StatusCode::CTAP1_ERR_INVALID_PARAMETER);
        }
        let x_array_ref = array_ref![x_bytes.as_slice(), 0, ecdh::NBYTES];
        let y_array_ref = array_ref![y_bytes.as_slice(), 0, ecdh::NBYTES];
        ecdh::PubKey::from_coordinates(x_array_ref, y_array_ref)
            .ok_or(Ctap2StatusCode::CTAP1_ERR_INVALID_PARAMETER)
    }
}

#[cfg_attr(any(test, feature = "debug_ctap"), derive(Debug, PartialEq))]
pub enum ClientPinSubCommand {
    GetPinRetries,
    GetKeyAgreement,
    SetPin,
    ChangePin,
    GetPinUvAuthTokenUsingPin,
    GetPinUvAuthTokenUsingUv,
    GetUvRetries,
}

impl From<ClientPinSubCommand> for cbor::Value {
    fn from(subcommand: ClientPinSubCommand) -> Self {
        match subcommand {
            ClientPinSubCommand::GetPinRetries => 0x01,
            ClientPinSubCommand::GetKeyAgreement => 0x02,
            ClientPinSubCommand::SetPin => 0x03,
            ClientPinSubCommand::ChangePin => 0x04,
            ClientPinSubCommand::GetPinUvAuthTokenUsingPin => 0x05,
            ClientPinSubCommand::GetPinUvAuthTokenUsingUv => 0x06,
            ClientPinSubCommand::GetUvRetries => 0x07,
        }
        .into()
    }
}

impl TryFrom<&cbor::Value> for ClientPinSubCommand {
    type Error = Ctap2StatusCode;

    fn try_from(cbor_value: &cbor::Value) -> Result<Self, Ctap2StatusCode> {
        let subcommand_int = read_unsigned(cbor_value)?;
        match subcommand_int {
            0x01 => Ok(ClientPinSubCommand::GetPinRetries),
            0x02 => Ok(ClientPinSubCommand::GetKeyAgreement),
            0x03 => Ok(ClientPinSubCommand::SetPin),
            0x04 => Ok(ClientPinSubCommand::ChangePin),
            0x05 => Ok(ClientPinSubCommand::GetPinUvAuthTokenUsingPin),
            0x06 => Ok(ClientPinSubCommand::GetPinUvAuthTokenUsingUv),
            0x07 => Ok(ClientPinSubCommand::GetUvRetries),
            // TODO(kaczmarczyck) what is the correct status code for this error?
            _ => Err(Ctap2StatusCode::CTAP1_ERR_INVALID_PARAMETER),
        }
    }
}

pub(super) fn read_unsigned(cbor_value: &cbor::Value) -> Result<u64, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::KeyValue(cbor::KeyType::Unsigned(unsigned)) => Ok(*unsigned),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub(super) fn read_integer(cbor_value: &cbor::Value) -> Result<i64, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::KeyValue(cbor::KeyType::Unsigned(unsigned)) => {
            if *unsigned <= core::i64::MAX as u64 {
                Ok(*unsigned as i64)
            } else {
                Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
            }
        }
        cbor::Value::KeyValue(cbor::KeyType::Negative(signed)) => Ok(*signed),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub fn read_byte_string(cbor_value: &cbor::Value) -> Result<Vec<u8>, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::KeyValue(cbor::KeyType::ByteString(byte_string)) => Ok(byte_string.to_vec()),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

fn extract_byte_string(cbor_value: cbor::Value) -> Result<Vec<u8>, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::KeyValue(cbor::KeyType::ByteString(byte_string)) => Ok(byte_string),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub(super) fn read_text_string(cbor_value: &cbor::Value) -> Result<String, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::KeyValue(cbor::KeyType::TextString(text_string)) => {
            Ok(text_string.to_string())
        }
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

fn extract_text_string(cbor_value: cbor::Value) -> Result<String, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::KeyValue(cbor::KeyType::TextString(text_string)) => Ok(text_string),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub(super) fn read_array(cbor_value: &cbor::Value) -> Result<&Vec<cbor::Value>, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::Array(array) => Ok(array),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub(super) fn read_map(
    cbor_value: &cbor::Value,
) -> Result<&BTreeMap<cbor::KeyType, cbor::Value>, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::Map(map) => Ok(map),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

fn extract_map(
    cbor_value: cbor::Value,
) -> Result<BTreeMap<cbor::KeyType, cbor::Value>, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::Map(map) => Ok(map),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub(super) fn read_bool(cbor_value: &cbor::Value) -> Result<bool, Ctap2StatusCode> {
    match cbor_value {
        cbor::Value::Simple(cbor::SimpleValue::FalseValue) => Ok(false),
        cbor::Value::Simple(cbor::SimpleValue::TrueValue) => Ok(true),
        _ => Err(Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE),
    }
}

pub(super) fn ok_or_missing<T>(value_option: Option<T>) -> Result<T, Ctap2StatusCode> {
    value_option.ok_or(Ctap2StatusCode::CTAP2_ERR_MISSING_PARAMETER)
}

#[cfg(test)]
mod test {
    use self::Ctap2StatusCode::CTAP2_ERR_CBOR_UNEXPECTED_TYPE;
    use super::*;
    use alloc::collections::BTreeMap;
    use crypto::rng256::{Rng256, ThreadRng256};

    #[test]
    fn test_read_unsigned() {
        assert_eq!(read_unsigned(&cbor_int!(123)), Ok(123));
        assert_eq!(
            read_unsigned(&cbor_bool!(true)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_unsigned(&cbor_text!("foo")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_unsigned(&cbor_bytes_lit!(b"bar")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_unsigned(&cbor_array![]),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_unsigned(&cbor_map! {}),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_read_unsigned_limits() {
        assert_eq!(
            read_unsigned(&cbor_unsigned!(std::u64::MAX)),
            Ok(std::u64::MAX)
        );
        assert_eq!(
            read_unsigned(&cbor_unsigned!((std::i64::MAX as u64) + 1)),
            Ok((std::i64::MAX as u64) + 1)
        );
        assert_eq!(
            read_unsigned(&cbor_int!(std::i64::MAX)),
            Ok(std::i64::MAX as u64)
        );
        assert_eq!(read_unsigned(&cbor_int!(123)), Ok(123));
        assert_eq!(read_unsigned(&cbor_int!(1)), Ok(1));
        assert_eq!(read_unsigned(&cbor_int!(0)), Ok(0));
        assert_eq!(
            read_unsigned(&cbor_int!(-1)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_unsigned(&cbor_int!(-123)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_unsigned(&cbor_int!(std::i64::MIN)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_read_integer() {
        assert_eq!(read_integer(&cbor_int!(123)), Ok(123));
        assert_eq!(read_integer(&cbor_int!(-123)), Ok(-123));
        assert_eq!(
            read_integer(&cbor_bool!(true)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_integer(&cbor_text!("foo")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_integer(&cbor_bytes_lit!(b"bar")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_integer(&cbor_array![]),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_integer(&cbor_map! {}),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_read_integer_limits() {
        assert_eq!(
            read_integer(&cbor_unsigned!(std::u64::MAX)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_integer(&cbor_unsigned!((std::i64::MAX as u64) + 1)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(read_integer(&cbor_int!(std::i64::MAX)), Ok(std::i64::MAX));
        assert_eq!(read_integer(&cbor_int!(123)), Ok(123));
        assert_eq!(read_integer(&cbor_int!(1)), Ok(1));
        assert_eq!(read_integer(&cbor_int!(0)), Ok(0));
        assert_eq!(read_integer(&cbor_int!(-1)), Ok(-1));
        assert_eq!(read_integer(&cbor_int!(-123)), Ok(-123));
        assert_eq!(read_integer(&cbor_int!(std::i64::MIN)), Ok(std::i64::MIN));
    }

    #[test]
    fn test_read_byte_string() {
        assert_eq!(
            read_byte_string(&cbor_int!(123)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_byte_string(&cbor_bool!(true)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_byte_string(&cbor_text!("foo")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(read_byte_string(&cbor_bytes_lit!(b"")), Ok(Vec::new()));
        assert_eq!(
            read_byte_string(&cbor_bytes_lit!(b"bar")),
            Ok(b"bar".to_vec())
        );
        assert_eq!(
            read_byte_string(&cbor_array![]),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_byte_string(&cbor_map! {}),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_read_text_string() {
        assert_eq!(
            read_text_string(&cbor_int!(123)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_text_string(&cbor_bool!(true)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(read_text_string(&cbor_text!("")), Ok(String::new()));
        assert_eq!(
            read_text_string(&cbor_text!("foo")),
            Ok(String::from("foo"))
        );
        assert_eq!(
            read_text_string(&cbor_bytes_lit!(b"bar")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_text_string(&cbor_array![]),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_text_string(&cbor_map! {}),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_read_array() {
        assert_eq!(
            read_array(&cbor_int!(123)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_array(&cbor_bool!(true)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_array(&cbor_text!("foo")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_array(&cbor_bytes_lit!(b"bar")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(read_array(&cbor_array![]), Ok(&Vec::new()));
        assert_eq!(
            read_array(&cbor_array![
                123,
                cbor_null!(),
                "foo",
                cbor_array![],
                cbor_map! {},
            ]),
            Ok(&vec![
                cbor_int!(123),
                cbor_null!(),
                cbor_text!("foo"),
                cbor_array![],
                cbor_map! {},
            ])
        );
        assert_eq!(
            read_array(&cbor_map! {}),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_read_map() {
        assert_eq!(
            read_map(&cbor_int!(123)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_map(&cbor_bool!(true)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_map(&cbor_text!("foo")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_map(&cbor_bytes_lit!(b"bar")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_map(&cbor_array![]),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(read_map(&cbor_map! {}), Ok(&BTreeMap::new()));
        assert_eq!(
            read_map(&cbor_map! {
                1 => cbor_false!(),
                "foo" => b"bar",
                b"bin" => -42,
            }),
            Ok(&[
                (cbor_unsigned!(1), cbor_false!()),
                (cbor_text!("foo"), cbor_bytes_lit!(b"bar")),
                (cbor_bytes_lit!(b"bin"), cbor_int!(-42)),
            ]
            .iter()
            .cloned()
            .collect::<BTreeMap<_, _>>())
        );
    }

    #[test]
    fn test_read_bool() {
        assert_eq!(
            read_bool(&cbor_int!(123)),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(read_bool(&cbor_bool!(true)), Ok(true));
        assert_eq!(read_bool(&cbor_bool!(false)), Ok(false));
        assert_eq!(
            read_bool(&cbor_text!("foo")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_bool(&cbor_bytes_lit!(b"bar")),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_bool(&cbor_array![]),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
        assert_eq!(
            read_bool(&cbor_map! {}),
            Err(CTAP2_ERR_CBOR_UNEXPECTED_TYPE)
        );
    }

    #[test]
    fn test_from_public_key_credential_rp_entity() {
        let cbor_rp_entity = cbor_map! {
            "id" => "example.com",
            "name" => "Example",
            "icon" => "example.com/icon.png",
        };
        let rp_entity = PublicKeyCredentialRpEntity::try_from(&cbor_rp_entity);
        let expected_rp_entity = PublicKeyCredentialRpEntity {
            rp_id: "example.com".to_string(),
            rp_name: Some("Example".to_string()),
            rp_icon: Some("example.com/icon.png".to_string()),
        };
        assert_eq!(rp_entity, Ok(expected_rp_entity));
    }

    #[test]
    fn test_from_into_public_key_credential_user_entity() {
        let cbor_user_entity = cbor_map! {
            "id" => vec![0x1D, 0x1D, 0x1D, 0x1D],
            "name" => "foo",
            "displayName" => "bar",
            "icon" => "example.com/foo/icon.png",
        };
        let user_entity = PublicKeyCredentialUserEntity::try_from(&cbor_user_entity);
        let expected_user_entity = PublicKeyCredentialUserEntity {
            user_id: vec![0x1D, 0x1D, 0x1D, 0x1D],
            user_name: Some("foo".to_string()),
            user_display_name: Some("bar".to_string()),
            user_icon: Some("example.com/foo/icon.png".to_string()),
        };
        assert_eq!(user_entity, Ok(expected_user_entity));
        let created_cbor: cbor::Value = user_entity.unwrap().into();
        assert_eq!(created_cbor, cbor_user_entity);
    }

    #[test]
    fn test_from_into_public_key_credential_type() {
        let cbor_credential_type = cbor_text!("public-key");
        let credential_type = PublicKeyCredentialType::try_from(&cbor_credential_type);
        let expected_credential_type = PublicKeyCredentialType::PublicKey;
        assert_eq!(credential_type, Ok(expected_credential_type));
        let created_cbor: cbor::Value = credential_type.unwrap().into();
        assert_eq!(created_cbor, cbor_credential_type);

        let cbor_unknown_type = cbor_text!("unknown-type");
        let unknown_type = PublicKeyCredentialType::try_from(&cbor_unknown_type);
        let expected_unknown_type = PublicKeyCredentialType::Unknown;
        assert_eq!(unknown_type, Ok(expected_unknown_type));
    }

    #[test]
    fn test_from_into_signature_algorithm() {
        let cbor_signature_algorithm = cbor_int!(ecdsa::PubKey::ES256_ALGORITHM);
        let signature_algorithm = SignatureAlgorithm::try_from(&cbor_signature_algorithm);
        let expected_signature_algorithm = SignatureAlgorithm::ES256;
        assert_eq!(signature_algorithm, Ok(expected_signature_algorithm));
        let created_cbor: cbor::Value = cbor_int!(signature_algorithm.unwrap() as i64);
        assert_eq!(created_cbor, cbor_signature_algorithm);

        let cbor_unknown_algorithm = cbor_int!(-1);
        let unknown_algorithm = SignatureAlgorithm::try_from(&cbor_unknown_algorithm);
        let expected_unknown_algorithm = SignatureAlgorithm::Unknown;
        assert_eq!(unknown_algorithm, Ok(expected_unknown_algorithm));
    }

    #[test]
    fn test_from_into_authenticator_transport() {
        let cbor_authenticator_transport = cbor_text!("usb");
        let authenticator_transport =
            AuthenticatorTransport::try_from(&cbor_authenticator_transport);
        let expected_authenticator_transport = AuthenticatorTransport::Usb;
        assert_eq!(
            authenticator_transport,
            Ok(expected_authenticator_transport)
        );
        let created_cbor: cbor::Value = authenticator_transport.unwrap().into();
        assert_eq!(created_cbor, cbor_authenticator_transport);
    }

    #[test]
    fn test_from_into_public_key_credential_parameter() {
        let cbor_credential_parameter = cbor_map! {
            "type" => "public-key",
            "alg" => ecdsa::PubKey::ES256_ALGORITHM,
        };
        let credential_parameter =
            PublicKeyCredentialParameter::try_from(&cbor_credential_parameter);
        let expected_credential_parameter = PublicKeyCredentialParameter {
            cred_type: PublicKeyCredentialType::PublicKey,
            alg: SignatureAlgorithm::ES256,
        };
        assert_eq!(credential_parameter, Ok(expected_credential_parameter));
        let created_cbor: cbor::Value = credential_parameter.unwrap().into();
        assert_eq!(created_cbor, cbor_credential_parameter);
    }

    #[test]
    fn test_from_into_public_key_credential_descriptor() {
        let cbor_credential_descriptor = cbor_map! {
            "type" => "public-key",
            "id" => vec![0x2D, 0x2D, 0x2D, 0x2D],
            "transports" => cbor_array!["usb"],
        };
        let credential_descriptor =
            PublicKeyCredentialDescriptor::try_from(&cbor_credential_descriptor);
        let expected_credential_descriptor = PublicKeyCredentialDescriptor {
            key_type: PublicKeyCredentialType::PublicKey,
            key_id: vec![0x2D, 0x2D, 0x2D, 0x2D],
            transports: Some(vec![AuthenticatorTransport::Usb]),
        };
        assert_eq!(credential_descriptor, Ok(expected_credential_descriptor));
        let created_cbor: cbor::Value = credential_descriptor.unwrap().into();
        assert_eq!(created_cbor, cbor_credential_descriptor);
    }

    #[test]
    fn test_from_into_extensions() {
        let cbor_extensions = cbor_map! {
            "the_answer" => 42,
        };
        let extensions = Extensions::try_from(&cbor_extensions);
        let mut expected_extensions = Extensions(BTreeMap::new());
        expected_extensions
            .0
            .insert("the_answer".to_string(), cbor_int!(42));
        assert_eq!(extensions, Ok(expected_extensions));
        let created_cbor: cbor::Value = extensions.unwrap().into();
        assert_eq!(created_cbor, cbor_extensions);
    }

    #[test]
    fn test_from_into_get_assertion_hmac_secret_output() {
        let cbor_output = cbor_bytes![vec![0xC0; 32]];
        let output = GetAssertionHmacSecretOutput::try_from(&cbor_output);
        let expected_output = GetAssertionHmacSecretOutput(vec![0xC0; 32]);
        assert_eq!(output, Ok(expected_output));
        let created_cbor: cbor::Value = output.unwrap().into();
        assert_eq!(created_cbor, cbor_output);
    }

    #[test]
    fn test_hmac_secret_extension() {
        let cbor_extensions = cbor_map! {
            "hmac-secret" => true,
        };
        let extensions = Extensions::try_from(&cbor_extensions).unwrap();
        assert!(extensions.has_make_credential_hmac_secret().unwrap());

        let cbor_extensions = cbor_map! {
            "hmac-secret" => false,
        };
        let extensions = Extensions::try_from(&cbor_extensions).unwrap();
        assert!(!extensions.has_make_credential_hmac_secret().unwrap());

        let mut rng = ThreadRng256 {};
        let sk = crypto::ecdh::SecKey::gensk(&mut rng);
        let pk = sk.genpk();
        let cose_key = CoseKey::from(pk.clone());
        let cbor_extensions = cbor_map! {
            "hmac-secret" => cbor_map! {
                1 => cbor::Value::Map(cose_key.0.clone()),
                2 => vec![0x02; 32],
                3 => vec![0x03; 32],
            },
        };
        let extensions = Extensions::try_from(&cbor_extensions).unwrap();
        let get_assertion_input = extensions.get_assertion_hmac_secret();
        let expected_input = GetAssertionHmacSecretInput {
            key_agreement: cose_key,
            salt_enc: vec![0x02; 32],
            salt_auth: vec![0x03; 32],
        };
        assert_eq!(get_assertion_input, Some(Ok(expected_input)));
    }

    #[test]
    fn test_from_make_credential_options() {
        let cbor_make_options = cbor_map! {
            "rk" => true,
            "uv" => false,
        };
        let make_options = MakeCredentialOptions::try_from(&cbor_make_options);
        let expected_make_options = MakeCredentialOptions {
            rk: true,
            uv: false,
        };
        assert_eq!(make_options, Ok(expected_make_options));
    }

    #[test]
    fn test_from_get_assertion_options() {
        let cbor_get_assertion = cbor_map! {
            "up" => true,
            "uv" => false,
        };
        let get_assertion = GetAssertionOptions::try_from(&cbor_get_assertion);
        let expected_get_assertion = GetAssertionOptions {
            up: true,
            uv: false,
        };
        assert_eq!(get_assertion, Ok(expected_get_assertion));
    }

    #[test]
    fn test_into_packed_attestation_statement() {
        let certificate: cbor::values::KeyType = cbor_bytes![vec![0x5C, 0x5C, 0x5C, 0x5C]];
        let cbor_packed_attestation_statement = cbor_map! {
            "alg" => 1,
            "sig" => vec![0x55, 0x55, 0x55, 0x55],
            "x5c" => cbor_array_vec![vec![certificate]],
            "ecdaaKeyId" => vec![0xEC, 0xDA, 0x1D],
        };
        let packed_attestation_statement = PackedAttestationStatement {
            alg: 1,
            sig: vec![0x55, 0x55, 0x55, 0x55],
            x5c: Some(vec![vec![0x5C, 0x5C, 0x5C, 0x5C]]),
            ecdaa_key_id: Some(vec![0xEC, 0xDA, 0x1D]),
        };
        let created_cbor: cbor::Value = packed_attestation_statement.into();
        assert_eq!(created_cbor, cbor_packed_attestation_statement);
    }

    #[test]
    fn test_from_into_cose_key() {
        let mut rng = ThreadRng256 {};
        let sk = crypto::ecdh::SecKey::gensk(&mut rng);
        let pk = sk.genpk();
        let cose_key = CoseKey::from(pk.clone());
        let created_pk = ecdh::PubKey::try_from(cose_key);
        assert_eq!(created_pk, Ok(pk));
    }

    #[test]
    fn test_from_into_client_pin_sub_command() {
        let cbor_sub_command = cbor_int!(0x01);
        let sub_command = ClientPinSubCommand::try_from(&cbor_sub_command);
        let expected_sub_command = ClientPinSubCommand::GetPinRetries;
        assert_eq!(sub_command, Ok(expected_sub_command));
        let created_cbor: cbor::Value = sub_command.unwrap().into();
        assert_eq!(created_cbor, cbor_sub_command);
    }

    #[test]
    fn test_credential_source_cbor_round_trip() {
        let mut rng = ThreadRng256 {};
        let credential = PublicKeyCredentialSource {
            key_type: PublicKeyCredentialType::PublicKey,
            credential_id: rng.gen_uniform_u8x32().to_vec(),
            private_key: crypto::ecdsa::SecKey::gensk(&mut rng),
            rp_id: "example.com".to_string(),
            user_handle: b"foo".to_vec(),
            other_ui: None,
            cred_random: None,
        };

        assert_eq!(
            PublicKeyCredentialSource::try_from(cbor::Value::from(credential.clone())),
            Ok(credential.clone())
        );

        let credential = PublicKeyCredentialSource {
            other_ui: Some("other".to_string()),
            ..credential
        };

        assert_eq!(
            PublicKeyCredentialSource::try_from(cbor::Value::from(credential.clone())),
            Ok(credential.clone())
        );

        let credential = PublicKeyCredentialSource {
            cred_random: Some(vec![0x00; 32]),
            ..credential
        };

        assert_eq!(
            PublicKeyCredentialSource::try_from(cbor::Value::from(credential.clone())),
            Ok(credential)
        );
    }

    #[test]
    fn test_credential_source_invalid_cbor() {
        assert!(PublicKeyCredentialSource::try_from(cbor_false!()).is_err());
        assert!(PublicKeyCredentialSource::try_from(cbor_array!(false)).is_err());
        assert!(PublicKeyCredentialSource::try_from(cbor_array!(b"foo".to_vec())).is_err());
    }
}
