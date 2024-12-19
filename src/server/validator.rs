use crate::config::schema::ValidatorConfiguration;
use rust_cfzt_validator::{Validator, TeamValidator};
use jnt::types::StdResult;

fn new_single_team_configuration(team_name: &str, audience: &str) -> StdResult<TeamValidator> {
    TeamValidator::from_team_name(team_name, audience)
}

pub fn new_validator(configuration: &ValidatorConfiguration) -> StdResult<Box<dyn Validator>> {
    match configuration {
        ValidatorConfiguration::Team(config) => Ok(
            Box::new(new_single_team_configuration(&config.team_name, &config.audience)?)
        ),
    }
}