use super::schema::Configuration;
use jnt::types;
use rust_cfzt_validator::api::TeamKeys;
use std::env;
use std::str::FromStr;

use crate::config::bootstrap::schema::TimeConstraintMode;

jnt::env!(discover_listener_str, "LISTENER", "tcp://[::1]:10000");
jnt::env!(discover_static_keys_str, "STATIC_KEYS", "");
jnt::env!(discover_iat_validation_str, "NBF_VALIDATION", "strict");
jnt::env!(discover_exp_validation_str, "EXP_VALIDATION", "strict");
jnt::env!(discover_sync_schedule_str, "SYNC_SCHEDULE", "0 0 0 * * *");

pub fn discover_bootstrap_configuration() -> types::StdResult<Configuration> {
    let static_key_str = discover_static_keys_str();
    let team_name = env::var("TEAM_NAME")?;
    let nbf_validation = TimeConstraintMode::from_str(&discover_iat_validation_str())?;
    let exp_validation = TimeConstraintMode::from_str(&discover_exp_validation_str())?;

    if !static_key_str.is_empty() {
        let static_keys = Some(TeamKeys::from_str(&team_name, &static_key_str)?);
        return Ok(Configuration::new_single_team_configuration(
            &discover_listener_str(),
            &team_name,
            static_keys,
            &discover_sync_schedule_str(),
            nbf_validation,
            exp_validation,
        ));
    }

    Ok(Configuration::new_single_team_configuration(
        &discover_listener_str(),
        &team_name,
        None,
        &discover_sync_schedule_str(),
        nbf_validation,
        exp_validation,
    ))
}
