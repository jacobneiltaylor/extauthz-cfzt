use std::sync::Arc;

use envoy_types::ext_authz::v3::{
    pb::{Authorization, CheckRequest, CheckResponse},
    CheckResponseExt, OkHttpResponseBuilder,
};
use jsonwebtoken::{Algorithm, Validation};
use rust_cfzt_validator::Validator;
use tonic::{Request, Response, Status};

use super::{
    request::{get_headers, PrincipalAssertion},
    response::ResponseMutator,
};
use crate::config::audience::schema::AudienceProvider;
use crate::config::bootstrap::schema::TimeConstraintMode;

pub struct CloudflareZeroTrustAuthorizationServer {
    validator: Arc<Box<dyn Validator>>,
    aud_provider: Arc<Box<dyn AudienceProvider>>,
    default_team_name: String,
    nbf_validation: TimeConstraintMode,
    exp_validation: TimeConstraintMode,
}

impl CloudflareZeroTrustAuthorizationServer {
    pub fn new(
        validator: Arc<Box<dyn Validator>>,
        aud_provider: Arc<Box<dyn AudienceProvider>>,
        default_team_name: &str,
        nbf_validation: TimeConstraintMode,
        exp_validation: TimeConstraintMode,
    ) -> Self {
        CloudflareZeroTrustAuthorizationServer {
            validator,
            aud_provider,
            default_team_name: default_team_name.to_string(),
            nbf_validation,
            exp_validation,
        }
    }

    fn validate(&self, token: &str) -> super::StatusResult<PrincipalAssertion> {
        let mut constraints = Validation::new(Algorithm::RS256);
        constraints.set_audience(&self.aud_provider.get_audiences());

        if self.nbf_validation == TimeConstraintMode::Lax {
            constraints.validate_nbf = false;
        }

        if self.exp_validation == TimeConstraintMode::Lax {
            constraints.validate_exp = false;
        }

        match self
            .validator
            .validate_token(token, &self.default_team_name, &mut constraints)
        {
            Ok(claims) => match PrincipalAssertion::from_claims_value(&claims.claims) {
                Ok(assertion) => Ok(assertion),
                Err(e) => Err(Status::invalid_argument(format!(
                    "failed claims processing: {e}"
                ))),
            },
            Err(e) => Err(Status::unauthenticated(format!(
                "failed CF JWT validation: {e}"
            ))),
        }
    }
}

#[allow(unused)]
#[tonic::async_trait]
impl Authorization for CloudflareZeroTrustAuthorizationServer {
    async fn check(&self, request: Request<CheckRequest>) -> super::ExtAuthzResult {
        let check_request = request.into_inner();
        let client_headers = get_headers(&check_request)?;

        let header = "cf-access-jwt-assertion".to_string();
        match client_headers.get(&header) {
            Some(value) => match self.validate(value) {
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
            },
            None => {
                log::warn!("Request missing JWT header");
                Err(Status::invalid_argument("Missing CF JWT header"))
            }
        }
    }
}
