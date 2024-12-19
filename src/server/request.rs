use std::collections::HashMap;
use phf::phf_set;
use serde_json::Value;
use jnt::types::{ConstHashSet, StdResult};
use tonic::Status;
use envoy_types::ext_authz::v3::{pb::CheckRequest, CheckRequestExt};

pub fn get_headers(req: &CheckRequest) -> super::StatusResult<&HashMap<String, String>> {
    req.get_client_headers().ok_or_else(|| Status::invalid_argument("headers not provided by envoy"))
}

type ClaimInteger = u64;

const DEFAULT_USER_CLAIMS: ConstHashSet<&str> = phf_set!(
    "aud",
    "email",
    "exp",
    "iat",
    "nbf",
    "iss",
    "type",
    "nonce",
    "sub",
    "country",
);

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
    pub extra_claims: HashMap<String, String>
}

fn get_required_claim<'a>(object: &'a serde_json::map::Map<String, Value>, claim: &str) -> StdResult<&'a Value> {
    Ok(object.get(claim).ok_or(format!("{claim} claim missing"))?)
}

fn get_required_str_claim(object: &serde_json::map::Map<String, Value>, claim: &str) -> StdResult<String> {
    Ok(get_required_claim(object, claim)?.as_str().ok_or(format!("{claim} claim should be str"))?.to_string())
}

fn get_required_int_claim(object: &serde_json::map::Map<String, Value>, claim: &str) -> StdResult<ClaimInteger> {
    Ok(get_required_claim(object, claim)?.as_u64().ok_or(format!("{claim} claim should be int"))?)
}

fn collect_audiences(object: &serde_json::map::Map<String, Value>) -> StdResult<Vec<String>> {
    let mut audiences: Vec<String> = vec![];

    for audience in get_required_claim(object, "aud")?.as_array().ok_or("aud must be array")? {
        audiences.push(audience.as_str().ok_or("audience values must be str")?.to_string());
    }

    Ok(audiences)
}

impl UserAssertion {
    fn from_claims_object(object: &serde_json::map::Map<String, Value>) -> StdResult<Self> {
        let mut extra_claims: HashMap<String, String> = HashMap::new();

        for claim in object.keys() {
            if !DEFAULT_USER_CLAIMS.contains(claim) {
                extra_claims.insert(claim.to_string(), get_required_str_claim(object, claim)?);
            }
        }

        Ok(UserAssertion {
            aud: collect_audiences(object)?,
            email: get_required_str_claim(object, "email")?,
            exp: get_required_int_claim(object, "exp")?,
            iat: get_required_int_claim(object, "iat")?,
            nbf: get_required_int_claim(object, "nbf")?,
            iss: get_required_str_claim(object, "iss")?,
            typ: get_required_str_claim(object, "type")?,
            nonce: get_required_str_claim(object, "nonce")?,
            sub: get_required_str_claim(object, "sub")?,
            country: get_required_str_claim(object, "country")?,
            extra_claims: extra_claims,
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
        let subject = object.get("sub").ok_or("sub claim missing")?.as_str().ok_or("sub claim must be str")?;

        if subject.len() == 0 {
            return Ok(Self::Service(ServiceAssertion::from_claims_object(object)?));
        }

        Ok(Self::User(UserAssertion::from_claims_object(object)?))
    }
}
