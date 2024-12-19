use std::env;
use jnt::types;

jnt::env!(discover_listener, "LISTENER", "tcp://[::1]:10000");

pub fn discover_configuration() -> types::StdResult<super::schema::Configuration> {
    let listener = discover_listener();
    let audience = env::var("AUDIENCE")?;
    let team_name = env::var("TEAM_NAME")?;
    Ok(super::schema::Configuration::new_single_team_configuration(&listener, &audience, &team_name))
}
