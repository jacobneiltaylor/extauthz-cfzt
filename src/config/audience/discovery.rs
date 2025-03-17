use crate::config::audience::schema::{AudienceProvider, StaticAudienceProvider};
use jnt::{opaque_err, types};

jnt::env!(
    discover_audience_provider_str,
    "AUDIENCE_PROVIDER",
    "static"
);
jnt::env!(discover_audience_str, "AUDIENCE", "");
jnt::env!(discover_audiences_str, "AUDIENCES", "");

type AudProviderResult = types::StdResult<Box<dyn AudienceProvider>>;

fn discover_audience() -> Option<StaticAudienceProvider> {
    let audience_str = discover_audience_str();

    if audience_str.is_empty() {
        return None;
    }

    Some(StaticAudienceProvider::new_single_aud(&audience_str))
}

fn discover_audiences() -> Option<StaticAudienceProvider> {
    let audiences_str = discover_audiences_str();

    if audiences_str.is_empty() {
        return None;
    }

    let audiences: Vec<String> = audiences_str.split(",").map(|s| s.to_string()).collect();
    Some(StaticAudienceProvider::new(audiences))
}

fn discover_static_provider() -> AudProviderResult {
    match discover_audience().or(discover_audiences()) {
        Some(provider) => Ok(Box::new(provider)),
        None => Err(opaque_err!("No audience configured for static provider")),
    }
}

pub fn discover_audience_provider() -> AudProviderResult {
    match discover_audience_provider_str().to_lowercase().as_str() {
        "static" => discover_static_provider(),
        _ => Err(opaque_err!("Invalid audience provider")),
    }
}
