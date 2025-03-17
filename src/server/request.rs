use envoy_types::ext_authz::v3::{pb::CheckRequest, CheckRequestExt};
use jnt::types::StdResult;
use serde_json::Value;
use std::collections::HashMap;
use tonic::Status;

pub fn get_headers(req: &CheckRequest) -> super::StatusResult<&HashMap<String, String>> {
    req.get_client_headers()
        .ok_or_else(|| Status::invalid_argument("headers not provided by envoy"))
}

type ClaimInteger = u64;

pub struct UserAssertion {
    pub aud: Vec<String>,
    pub email: String,
    pub exp: ClaimInteger,
    pub iat: ClaimInteger,
    pub nbf: ClaimInteger,
    pub iss: String,
    pub typ: String,
    pub nonce: String,
    pub sub: String,
    pub country: String,
    pub custom: HashMap<String, String>,
}

fn get_required_claim<'a>(
    object: &'a serde_json::map::Map<String, Value>,
    claim: &str,
) -> StdResult<&'a Value> {
    Ok(object.get(claim).ok_or(format!("{claim} claim missing"))?)
}

fn get_required_str_claim(
    object: &serde_json::map::Map<String, Value>,
    claim: &str,
) -> StdResult<String> {
    Ok(get_required_claim(object, claim)?
        .as_str()
        .ok_or(format!("{claim} claim should be str"))?
        .to_string())
}

fn get_required_int_claim(
    object: &serde_json::map::Map<String, Value>,
    claim: &str,
) -> StdResult<ClaimInteger> {
    Ok(get_required_claim(object, claim)?
        .as_u64()
        .ok_or(format!("{claim} claim should be int"))?)
}

fn collect_audiences(object: &serde_json::map::Map<String, Value>) -> StdResult<Vec<String>> {
    let mut audiences: Vec<String> = vec![];

    for audience in get_required_claim(object, "aud")?
        .as_array()
        .ok_or("aud must be array")?
    {
        audiences.push(
            audience
                .as_str()
                .ok_or("audience values must be str")?
                .to_string(),
        );
    }

    Ok(audiences)
}

fn force_as_string(value: &Value) -> String {
    match value.as_str() {
        Some(strval) => strval.to_string(),
        None => value.to_string(),
    }
}

fn get_custom_claims(
    object: &serde_json::map::Map<String, Value>,
) -> StdResult<HashMap<String, String>> {
    match object.get("custom") {
        Some(value) => {
            let mut claims: HashMap<String, String> = HashMap::new();
            let custom_obj = value.as_object().ok_or("custom claim must be obj")?;

            for (custom_claim, custom_val) in custom_obj.into_iter() {
                claims.insert(custom_claim.to_string(), force_as_string(custom_val));
            }

            Ok(claims)
        }
        None => Ok(HashMap::new()),
    }
}

impl UserAssertion {
    fn from_claims_object(object: &serde_json::map::Map<String, Value>) -> StdResult<Self> {
        Ok(UserAssertion {
            aud: collect_audiences(object)?,
            email: get_required_str_claim(object, "email")?,
            exp: get_required_int_claim(object, "exp")?,
            iat: get_required_int_claim(object, "iat")?,
            nbf: get_required_int_claim(object, "nbf")?,
            iss: get_required_str_claim(object, "iss")?,
            typ: get_required_str_claim(object, "type")?,
            nonce: get_required_str_claim(object, "identity_nonce")?,
            sub: get_required_str_claim(object, "sub")?,
            country: get_required_str_claim(object, "country")?,
            custom: get_custom_claims(object)?,
        })
    }
}

pub struct ServiceAssertion {
    pub aud: Vec<String>,
    pub exp: ClaimInteger,
    pub iat: ClaimInteger,
    pub iss: String,
    pub typ: String,
    pub common_name: String,
}

impl ServiceAssertion {
    fn from_claims_object(object: &serde_json::map::Map<String, Value>) -> StdResult<Self> {
        Ok(ServiceAssertion {
            aud: collect_audiences(object)?,
            exp: get_required_int_claim(object, "exp")?,
            iat: get_required_int_claim(object, "iat")?,
            iss: get_required_str_claim(object, "iss")?,
            typ: get_required_str_claim(object, "type")?,
            common_name: get_required_str_claim(object, "common_name")?,
        })
    }
}

pub enum PrincipalAssertion {
    User(UserAssertion),
    Service(ServiceAssertion),
}

impl PrincipalAssertion {
    pub fn from_claims_value(val: &serde_json::Value) -> StdResult<Self> {
        let object = val.as_object().ok_or("invalid claims value")?;
        let subject = object
            .get("sub")
            .ok_or("sub claim missing")?
            .as_str()
            .ok_or("sub claim must be str")?;

        if subject.is_empty() {
            return Ok(Self::Service(ServiceAssertion::from_claims_object(object)?));
        }

        Ok(Self::User(UserAssertion::from_claims_object(object)?))
    }
}
