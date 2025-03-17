use crate::config::bootstrap::schema::ValidatorConfiguration;
use jnt::types::StdResult;
use rust_cfzt_validator::{TeamValidator, Validator};

fn new_single_team_configuration(team_name: &str) -> StdResult<TeamValidator> {
    TeamValidator::from_team_name(team_name)
}

pub fn new_validator(configuration: &ValidatorConfiguration) -> StdResult<Box<dyn Validator>> {
    match configuration {
        ValidatorConfiguration::Team(config) => {
            Ok(Box::new(new_single_team_configuration(&config.team_name)?))
        }
    }
}
