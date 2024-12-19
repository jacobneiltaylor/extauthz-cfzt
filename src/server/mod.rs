use envoy_types::ext_authz::v3::pb::CheckResponse;
use tonic::{Response, Status};

pub mod extauthz;
pub mod request;
pub mod response;
pub mod validator;

type StatusResult<T> = Result<T, Status>;
type ExtAuthzResult = StatusResult<Response<CheckResponse>>;