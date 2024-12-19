use std::sync::Arc;

use tonic::{Request, Response, Status};
use envoy_types::ext_authz::v3::{pb::{
    Authorization, CheckRequest, CheckResponse
}, CheckResponseExt, OkHttpResponseBuilder};
use rust_cfzt_validator::Validator;
use jsonwebtoken::{Validation, Algorithm};

use super::{request::{get_headers, PrincipalAssertion}, response::ResponseMutator};

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

        match client_headers.get("Cf-Access-Jwt-Assertion") {
            Some(value) => {
                let assertion = self.validate(value)?;
                let mut builder = OkHttpResponseBuilder::new();
                assertion.mutate_response(&mut builder);

                let mut response = CheckResponse::with_status(Status::ok("token validated"));
                response.set_http_response(builder);
                Ok(Response::new(response))
            },
            None => Err(Status::invalid_argument("Missing CF JWT header")),
        }
    }
}
