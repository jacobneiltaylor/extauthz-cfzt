use crate::config::bootstrap::schema::{ValidatorConfiguration, StaticTeamValidatorConfiguration, CommonValidatorConfiguration};
use jnt::types::StdResult;
use rust_cfzt_validator::{TeamValidator, Validator, api::TeamKeys};

fn new_single_team_configuration(static_config: &StaticTeamValidatorConfiguration, common_config: &CommonValidatorConfiguration) -> StdResult<TeamValidator> {
    let mut builder = ureq::Agent::config_builder();

    if common_config.proxy_discovery {
        builder = builder.proxy(
            ureq::Proxy::try_from_env(),
        );
    }

    let agent = ureq::Agent::new_with_config(builder.build());

    match &static_config.static_keys {
        Some(team_keys) => Ok(TeamValidator::from_team_keys(
            TeamKeys::from_str(&static_config.team_name, &team_keys)?,
            agent,
        )),
        None => TeamValidator::from_team_name(&static_config.team_name, agent),
    }
}

pub fn new_validator(configuration: &ValidatorConfiguration) -> StdResult<Box<dyn Validator>> {
    match configuration {
        ValidatorConfiguration::Team(static_config, common_config) => {
            Ok(Box::new(new_single_team_configuration(static_config, common_config)?))
        }
    }
}
