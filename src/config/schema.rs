use jnt::types;
use jnt::sockets::Listener;
use rust_cfzt_validator::Validator;

pub struct StaticTeamValidatorConfiguration {
    pub audience: String,
    pub team_name: String,
}

pub enum ValidatorConfiguration {
    Team(StaticTeamValidatorConfiguration)
}

impl ValidatorConfiguration {
    pub fn get_default_team_name(&self) -> String {
        match self {
            Self::Team(config) => config.team_name.to_string()
        }
    }
}

pub struct Configuration {
    pub listener: String,
    pub validator: ValidatorConfiguration,
    pub sync_schedule: String,
}

impl Configuration {
    pub fn new(listener: &str, validator_config: ValidatorConfiguration) -> Self {
        Configuration{
            listener: listener.to_string(),
            validator: validator_config,
            sync_schedule: "0 0 0 * * *".to_string(),
        }
    }

    pub fn new_single_team_configuration(listener: &str, audience: &str, team_name: &str) -> Self {
        Configuration::new(listener, ValidatorConfiguration::Team(StaticTeamValidatorConfiguration{
            audience: audience.to_string(),
            team_name: team_name.to_string(),
        }))
    }

    pub fn open_listener(&self) -> types::StdResult<Listener> {
        Listener::from_url(url::Url::parse(&self.listener)?)
    }

    pub fn new_validator(&self) -> types::StdResult<Box<dyn Validator>> {
        crate::server::validator::new_validator(&self.validator)
    }
}
