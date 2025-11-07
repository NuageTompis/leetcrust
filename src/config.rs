use crate::args::{ConfigCommand, ConfigSubcommand};
use crate::logger::Log;
use crate::{read_write, LOGGER};

pub fn handle_config_command(config: ConfigCommand) {
    let (variable, value) = match config.command {
        ConfigSubcommand::Username(username_command) => ("username", username_command.username),
        ConfigSubcommand::Cookie(cookie_command) => ("cookie", cookie_command.cookie),
        ConfigSubcommand::Premium(premium_command) => {
            let res = assert_valid_u8_bool(premium_command.premium, "premium");
            if let Ok(res) = res {
                res
            } else {
                return;
            }
        }
        ConfigSubcommand::AllowDeadCode(allow_dead_code_command) => {
            let res =
                assert_valid_u8_bool(allow_dead_code_command.allow_dead_code, "allow_dead_code");
            if let Ok(res) = res {
                res
            } else {
                return;
            }
        }
    };

    let result = read_write::try_update_env_variable(variable, &value);
    match result {
        Ok(_) => {
            LOGGER.success(&format!("Wrote {} to .env", variable));
        }
        Err(e) => {
            println!("Unexpected error while handling config command: {}.", e);
        }
    }
}

fn assert_valid_u8_bool(val: u8, name: &str) -> Result<(&str, String), ()> {
    if val >= 2 {
        println!(
            "The {} value {} is not valid, it should be either 0 or 1",
            name, val
        );
        Err(())
    } else {
        Ok((name, val.to_string()))
    }
}
