use std::{
    io::{self, Write},
    sync::Arc,
};

use dotenv::dotenv;

use pfsense_auto_backup::{
    download_backup_config, login, restore_backup_config,
};

fn main() {
    dotenv().ok();

    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .cookie_provider(Arc::new(reqwest::cookie::Jar::default()))
        .build()
        .expect("Unable to build reqwest client.");
    login(&client);

    println!("pfSense Auto Backup Tool v0.1.0");

    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read command.");

        if command.ends_with('\n') {
            command.pop();

            if command.ends_with('\r') {
                command.pop();
            }
        }

        match command.as_str() {
            "backup" => download_backup_config(&client),
            "restore" => restore_backup_config(&client),
            "help" => println!(
                "backup\n    \
                    Backup the config file.\n\
                restore\n    \
                    Restore the config file.\n\
                help\n    \
                    Prints this help message.\n\
                exit / quit\n    \
                    Exit the pfSense Auto Backup tool."
            ),
            "exit" | "quit" => break,
            "" => {}
            unrecognized_command => println!(
                "Unrecognized command '{}'. For a list of available commands, \
                run `help`.",
                unrecognized_command
            ),
        }
    }
}
