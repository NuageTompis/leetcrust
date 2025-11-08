use crate::args::{ConfigCommand, ConfigSubcommand, GivenBooleanValue};
use crate::logger::Log;
use crate::{read_write, LOGGER};

pub fn handle_config_command(config: ConfigCommand) {
    let (variable, value) = match config.command {
        ConfigSubcommand::Username(command) => (String::from("username"), command.username),
        ConfigSubcommand::Cookie(command) => (String::from("cookie"), command.cookie),
        ConfigSubcommand::Premium(command) => {
            let res = handle_boolean_subcommand(command);
            if let Ok(res) = res {
                res
            } else {
                return;
            }
        }
        ConfigSubcommand::AllowDeadCode(command) => {
            let res = handle_boolean_subcommand(command);
            if let Ok(res) = res {
                res
            } else {
                return;
            }
        }
    };

    let result = read_write::try_update_env_variable(&variable, &value);
    match result {
        Ok(_) => {
            LOGGER.success(&format!("Wrote {} to .env", variable));
        }
        Err(e) => {
            println!("Unexpected error while handling config command: {}.", e);
        }
    }
}

fn handle_boolean_subcommand<T: GivenBooleanValue>(command: T) -> Result<(String, String), ()> {
    if command.is_valid() {
        Ok((command.get_name(), command.get_value().to_string()))
    } else {
        command.display_wrong_value();
        Err(())
    }
}
