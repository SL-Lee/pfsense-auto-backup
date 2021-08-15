use std::{
    fs,
    io::{self, Write},
    sync::Arc,
};

use dotenv::dotenv;

use pfsense_auto_backup::{
    download_backup_config, login, restore_backup_config, schedule_backups,
};

fn main() {
    dotenv().ok();

    let client = Arc::new(
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .cookie_store(true)
            .cookie_provider(Arc::new(reqwest::cookie::Jar::default()))
            .build()
            .expect("Unable to build reqwest client."),
    );

    while let Err(error) = login(client.clone()) {
        println!("{}", error);
        println!("Retrying...");
    }

    let _thread_handle = schedule_backups(client.clone());

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

        let command = command
            .split(' ')
            .filter(|command| !command.is_empty())
            .collect::<Vec<&str>>();

        match command.get(0) {
            Some(&"backup") => match command.get(1) {
                Some(&"now") => match download_backup_config(client.clone()) {
                    Ok(msg) => println!("{}", msg),
                    Err(error) => println!("{}", error),
                },
                Some(&"list") => match fs::read_dir(r".\Backups\") {
                    Ok(dir_entries) => {
                        for dir_entry in dir_entries {
                            println!(
                                "{}",
                                dir_entry
                                    .unwrap()
                                    .path()
                                    .components()
                                    .last()
                                    .unwrap()
                                    .as_os_str()
                                    .to_os_string()
                                    .to_string_lossy()
                            );
                        }
                    }
                    Err(error) => println!("{}", error.to_string()),
                },
                Some(&"delete") => match command.get(2) {
                    Some(&"help") => println!(
                        "backup delete <filename>\n    \
                            Delete the specified backup file.\n\
                            backup delete help\n    \
                            Prints this help message."
                    ),
                    Some(filename) => {
                        match fs::remove_file(format!(r"Backups\{}", filename))
                        {
                            Ok(()) => {
                                println!(
                                    "Successfully removed '{}'.",
                                    filename
                                );
                            }
                            Err(error) => println!("{}", error.to_string()),
                        }
                    }
                    None => println!(
                        "Please specify the filename of the backup file to \
                        delete. For more information, run `backup delete help`."
                    ),
                },
                Some(&"help") => println!(
                    "backup now\n    \
                        Backup the config file now.\n\
                    backup list\n    \
                        List all backups.\n\
                    backup delete\n    \
                        Delete a backup.\n\
                    backup help\n    \
                        Prints this help message."
                ),
                Some(unrecognized_subcommand) => println!(
                    "Unrecognized subcommand '{}'. For a list of available \
                    commands, run `backup help`.",
                    unrecognized_subcommand
                ),
                None => println!(
                    "Please specify a backup subcommand. For a list of \
                    available backup subcommands, run `backup help`."
                ),
            },
            Some(&"restore") => match command.get(1) {
                Some(&"help") => println!(
                    "restore <filename>\n    \
                        Restore the specified backup file.\n\
                    restore help\n    \
                        Prints this help message."
                ),
                Some(filename) => {
                    match restore_backup_config(client.clone(), filename) {
                        Ok(msg) => println!("{}", msg),
                        Err(error) => println!("{}", error),
                    }
                }
                None => println!(
                    "Please specify a restore subcommand. For a list of \
                    available restore subcommands, run `restore help`."
                ),
            },
            Some(&"help") => println!(
                "backup\n    \
                    Perform backup operations.\n\
                restore\n    \
                    Perform restore operations.\n\
                help\n    \
                    Prints this help message.\n\
                exit / quit\n    \
                    Exit the pfSense Auto Backup tool."
            ),
            Some(&"exit" | &"quit") => break,
            Some(unrecognized_command) => println!(
                "Unrecognized command '{}'. For a list of available commands, \
                run `help`.",
                unrecognized_command
            ),
            None => {}
        }
    }
}
