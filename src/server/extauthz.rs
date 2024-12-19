use std::sync::Arc;

use tonic::{Request, Response, Status};
use envoy_types::ext_authz::v3::{pb::{
    Authorization, CheckRequest, CheckResponse
}, CheckResponseExt, OkHttpResponseBuilder};
use rust_cfzt_validator::Validator;
use jsonwebtoken::{Validation, Algorithm};

use super::{request::{get_address, get_headers, PrincipalAssertion}, response::ResponseMutator};

pub struct CloudflareZeroTrustAuthorizationServer {
    validator: Arc<Box<dyn Validator>>,
    default_team_name: String
}

impl CloudflareZeroTrustAuthorizationServer {
    pub fn new(validator: Arc<Box<dyn Validator>>, default_team_name: &str) -> Self {
        CloudflareZeroTrustAuthorizationServer {
            validator: validator,
            default_team_name: default_team_name.to_string(),
        }
    }

    fn validate(&self, token: &str) -> super::StatusResult<PrincipalAssertion> {
        let mut constraints = Validation::new(Algorithm::RS256);
        match self.validator.validate_token(token, &self.default_team_name, &mut constraints) {
            Ok(claims) => {
                match PrincipalAssertion::from_claims_value(&claims.claims) {
                    Ok(assertion) => Ok(assertion),
                    Err(e) => Err(Status::invalid_argument(format!("failed claims processing: {e}"))),
                }
            },
            Err(e) => Err(Status::unauthenticated(format!("failed CF JWT validation: {e}")))
        }
    }
}

#[allow(unused)]
#[tonic::async_trait]
impl Authorization for CloudflareZeroTrustAuthorizationServer {
    async fn check(&self, request: Request<CheckRequest>) -> super::ExtAuthzResult {
        let check_request = request.into_inner();
        let client_headers = get_headers(&check_request)?;
        let client_address = get_address(&check_request)?;

        log::info!("Recieved request from {}", client_address);

        let header = "cf-access-jwt-assertion".to_string();
        match client_headers.get(&header) {
            Some(value) => {
                log::debug!("Request has JWT: {}", value);
                match self.validate(value) {
                    Ok(assertion) => {
                        log::info!("Request passed validation");
                        let mut builder = OkHttpResponseBuilder::new();
                        assertion.mutate_response(&mut builder);
        
                        let mut response = CheckResponse::with_status(Status::ok("token validated"));
                        response.set_http_response(builder);
                        Ok(Response::new(response))
                    }
                    Err(e) => {
                        log::info!("Request failed validation: {}", e.to_string());
                        Err(e)
                    }
                }

            },
            None => {
                log::warn!("Request from {} missing JWT header", client_address);
                Err(Status::invalid_argument("Missing CF JWT header"))
            },
        }
    }
}
