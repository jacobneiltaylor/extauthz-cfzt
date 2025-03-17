use std::str::FromStr;

use jnt::sockets::Listener;
use jnt::{opaque_err, types};
use rust_cfzt_validator::api::TeamKeys;
use rust_cfzt_validator::Validator;

#[derive(Debug, PartialEq)]
pub enum TimeConstraintMode {
    Strict,
    Lax,
}

impl FromStr for TimeConstraintMode {
    type Err = Box<jnt::errors::OpaqueError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "strict" => Ok(Self::Strict),
            "lax" => Ok(Self::Lax),
            _ => Err(opaque_err!("invalid time constraint value")),
        }
    }
}

pub struct StaticTeamValidatorConfiguration {
    pub team_name: String,
    pub static_keys: Option<TeamKeys>,
}

impl StaticTeamValidatorConfiguration {
    pub fn is_static_keys(&self) -> bool {
        self.static_keys.is_some()
    }
}

pub enum ValidatorConfiguration {
    Team(StaticTeamValidatorConfiguration),
}

impl ValidatorConfiguration {
    pub fn get_default_team_name(&self) -> String {
        match self {
            Self::Team(config) => config.team_name.to_string(),
        }
    }

    pub fn requires_refresh(&self) -> bool {
        match self {
            Self::Team(config) => !config.is_static_keys(),
        }
    }
}

pub struct Configuration {
    pub listener: String,
    pub validator: ValidatorConfiguration,
    pub sync_schedule: String,
    pub nbf_validation: TimeConstraintMode,
    pub exp_validation: TimeConstraintMode,
}

impl Configuration {
    pub fn new(
        listener: &str,
        validator_config: ValidatorConfiguration,
        sync_schedule: &str,
        nbf_validation: TimeConstraintMode,
        exp_validation: TimeConstraintMode,
    ) -> Self {
        Configuration {
            listener: listener.to_string(),
            validator: validator_config,
            sync_schedule: sync_schedule.to_string(),
            nbf_validation,
            exp_validation,
        }
    }

    pub fn new_single_team_configuration(
        listener: &str,
        team_name: &str,
        static_keys: Option<TeamKeys>,
        sync_schedule: &str,
        nbf_validation: TimeConstraintMode,
        exp_validation: TimeConstraintMode,
    ) -> Self {
        Configuration::new(
            listener,
            ValidatorConfiguration::Team(StaticTeamValidatorConfiguration {
                team_name: team_name.to_string(),
                static_keys,
            }),
            sync_schedule,
            nbf_validation,
            exp_validation,
        )
    }

    pub fn open_listener(&self) -> types::StdResult<Listener> {
        Listener::from_url(url::Url::parse(&self.listener)?)
    }

    pub fn new_validator(&self) -> types::StdResult<Box<dyn Validator>> {
        crate::server::validator::new_validator(&self.validator)
    }
}
