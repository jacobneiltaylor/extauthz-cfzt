use super::schema::Configuration;
use jnt::types;
use jnt::extensions::contains::ConstHashSetExt;
use std::env;
use std::str::FromStr;
use phf::phf_set;

use crate::config::bootstrap::schema::TimeConstraintMode;

const TRUTHY_STRS: ConstHashSetExt<&str> = ConstHashSetExt::<&str>(phf_set!("yes", "on", "enabled",));

jnt::bool_parser!(bool_parser, TRUTHY_STRS);

jnt::env!(discover_listener_str, "LISTENER", "tcp://[::1]:10000");
jnt::env!(discover_static_keys_str, "STATIC_KEYS", "");
jnt::env!(discover_iat_validation_str, "NBF_VALIDATION", "strict");
jnt::env!(discover_exp_validation_str, "EXP_VALIDATION", "strict");
jnt::env!(discover_sync_schedule_str, "SYNC_SCHEDULE", "0 0 0 * * *");
jnt::env!(discover_enable_proxy_discovery, "ENABLE_PROXY_DISCOVERY", bool, false, bool_parser);

pub fn discover_bootstrap_configuration() -> types::StdResult<Configuration> {
    let static_key_str = discover_static_keys_str();
    let team_name = env::var("TEAM_NAME")?;
    let nbf_validation = TimeConstraintMode::from_str(&discover_iat_validation_str())?;
    let exp_validation = TimeConstraintMode::from_str(&discover_exp_validation_str())?;
    let mut static_keys: Option<String> = None;

    if !static_key_str.is_empty() {
        static_keys = Some(static_key_str);
    }

    Ok(Configuration::new_single_team_configuration(
        &discover_listener_str(),
        &team_name,
        static_keys,
        &discover_sync_schedule_str(),
        nbf_validation,
        exp_validation,
        discover_enable_proxy_discovery(),
    ))
}
