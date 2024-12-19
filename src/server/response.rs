use envoy_types::ext_authz::v3::OkHttpResponseBuilder;
use jnt::types::EmptyResult;

use super::request::{PrincipalAssertion, ServiceAssertion, UserAssertion};


fn get_header_name(suffix: &str) -> String {
    format!("X-Cfzt-Extauthz-{suffix}")
}

fn set_header(builder: &mut OkHttpResponseBuilder, name: &str, value: &str) {
    builder.add_header(
        get_header_name(name), value, None, false
    );
}

pub trait ResponseMutator {
    fn mutate_response(&self, builder: &mut OkHttpResponseBuilder) -> EmptyResult;
}

impl ResponseMutator for UserAssertion {
    fn mutate_response(&self, builder: &mut OkHttpResponseBuilder) -> EmptyResult {
        set_header(builder, "Token-Type", "User");
        set_header(builder, "Audiences", &self.aud.join(","));
        set_header(builder, "Email", &self.email);
        set_header(builder, "Expiry", &self.exp.to_string());
        set_header(builder, "Issued-At", &self.iat.to_string());
        set_header(builder, "Not-Before", &self.nbf.to_string());
        set_header(builder, "Issuer", &self.iss);
        set_header(builder, "Type", &self.typ);
        set_header(builder, "Nonce", &self.nonce);
        set_header(builder, "Subject", &self.sub);
        set_header(builder, "Country", &self.country);

        for (key, value) in &self.custom {
            set_header(builder, &format!("Custom-{key}"), &value);
        }

        Ok(())
    }
}

impl ResponseMutator for ServiceAssertion {
    fn mutate_response(&self, builder: &mut OkHttpResponseBuilder) -> EmptyResult {
        set_header(builder, "Token-Type", "User");
        set_header(builder, "Audiences", &self.aud.join(","));
        set_header(builder, "Expiry", &self.exp.to_string());
        set_header(builder, "Issued-At", &self.iat.to_string());
        set_header(builder, "Issuer", &self.iss);
        set_header(builder, "Type", &self.typ);
        set_header(builder, "Common-Name", &self.common_name);

        Ok(())
    }
}

impl ResponseMutator for PrincipalAssertion {
    fn mutate_response(&self, builder: &mut OkHttpResponseBuilder) -> EmptyResult {
        match self {
            Self::User(assertion) => assertion.mutate_response(builder),
            Self::Service(assertion) => assertion.mutate_response(builder),
        }
    }
}
