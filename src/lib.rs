use std::{
    env, fs::create_dir, path::Path, str::FromStr, sync::Arc, time::Duration,
};

use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use regex::Regex;

const PFSENSE_LOGIN_PAGE_URL: &str = "https://192.168.1.10/";
const PFSENSE_BACKUP_PAGE_URL: &str = "https://192.168.1.10/diag_backup.php";

fn get_csrf_token(client: Arc<reqwest::blocking::Client>, url: &str) -> String {
    let response = client.get(url).send().expect("Unable to load page.");
    let csrf_token_regex = Regex::new(
        r#"<input type='hidden' name='__csrf_magic' value="(.+?)" />"#,
    )
    .unwrap();

    match csrf_token_regex.captures(&response.text().unwrap()) {
        Some(captures) => captures.get(1).unwrap().as_str().to_string(),
        None => panic!("Unable to retrieve CSRF token from login page"),
    }
}

pub fn login(client: Arc<reqwest::blocking::Client>) -> Result<String, String> {
    let csrf_token = get_csrf_token(client.clone(), PFSENSE_LOGIN_PAGE_URL);
    let pfsense_username = env::var("PFSENSE_USERNAME")
        .expect("'PFSENSE_USERNAME' environment variable is not set.");
    let pfsense_password = env::var("PFSENSE_PASSWORD")
        .expect("'PFSENSE_PASSWORD' environment variable is not set.");
    let form = reqwest::blocking::multipart::Form::new()
        .text("__csrf_magic", csrf_token)
        .text("usernamefld", pfsense_username)
        .text("passwordfld", pfsense_password)
        .text("login", "Sign+In");

    match client.post(PFSENSE_LOGIN_PAGE_URL).multipart(form).send() {
        Ok(response) if response.status().is_success() => {
            Ok("Logged in successfully.".to_string())
        }
        Err(error) if error.is_timeout() => {
            Err("The log in request timed out.".to_string())
        }
        _ => Err("There was an error while trying to log in.".to_string()),
    }
}

pub fn schedule_backups(
    client: Arc<reqwest::blocking::Client>,
) -> ScheduleHandle {
    let mut scheduler = Scheduler::new();

    let backup_schedule_regex =
        Regex::new(r"^(?P<quantity>\d+)(?P<unit>(?:min|hr|d|wk))$").unwrap();
    let backup_schedule = env::var("BACKUP_SCHEDULE")
        .expect("'BACKUP_SCHEDULE' environment variable is not set.");
    let captures = backup_schedule_regex.captures(&backup_schedule).expect(
        "Invalid backup schedule specified in the 'BACKUP_SCHEDULE' \
        environment variable. A valid backup schedule follows the format \
        <quantity><time-unit>, where <quantity> is a numeric digit and \
        <time-unit> can be one of the following: `min`, `hr`, `d`, or `wk`.",
    );

    let quantity =
        u32::from_str(captures.name("quantity").unwrap().as_str()).unwrap();
    let interval = match captures.name("unit").unwrap().as_str() {
        "min" => quantity.minutes(),
        "h" => quantity.hours(),
        "d" => quantity.days(),
        "wk" => quantity.weeks(),
        _ => unreachable!(),
    };

    scheduler.every(interval).run(move || {
        let _ = download_backup_config(client.clone());
    });
    scheduler.watch_thread(Duration::from_millis(1000))
}

pub fn download_backup_config(
    client: Arc<reqwest::blocking::Client>,
) -> Result<String, String> {
    let csrf_token = get_csrf_token(client.clone(), PFSENSE_BACKUP_PAGE_URL);
    let form = reqwest::blocking::multipart::Form::new()
        .text("__csrf_magic", csrf_token)
        .text("backuparea", "")
        .text("donotbackuprrd", "yes")
        .text("backupdata", "yes")
        .text("encrypt", "yes")
        .text("encrypt_password", "ok")
        .text("encrypt_password_confirm", "ok")
        .text("download", "Download configuration as XML")
        .text("restorearea", "")
        .part(
            "conffile",
            reqwest::blocking::multipart::Part::text("")
                .file_name("")
                .mime_str("application/octet-stream")
                .unwrap(),
        )
        .text("decrypt_password", "");
    let mut response = client
        .post(PFSENSE_BACKUP_PAGE_URL)
        .multipart(form)
        .send()
        .unwrap();
    let filename_regex = Regex::new(r"attachment; filename=(.+)").unwrap();
    let filename = filename_regex
        .captures(
            response
                .headers()
                .get(reqwest::header::CONTENT_DISPOSITION)
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap()
        .get(1)
        .unwrap()
        .as_str();

    if !Path::new(r".\Backups\").exists() {
        create_dir(r".\Backups\").unwrap();
    }

    let mut backup_file =
        std::fs::File::create(format!("Backups\\{}", filename)).unwrap();

    response
        .copy_to(&mut backup_file)
        .map(|_| "Config file backed up successfully.".to_string())
        .map_err(|_| "Unable to back up config file.".to_string())
}

pub fn restore_backup_config(
    client: Arc<reqwest::blocking::Client>,
    filename: &str,
) -> Result<String, String> {
    let csrf_token = get_csrf_token(client.clone(), PFSENSE_BACKUP_PAGE_URL);
    let form = reqwest::blocking::multipart::Form::new()
        .text("__csrf_magic", csrf_token)
        .text("backuparea", "")
        .text("donotbackuprrd", "yes")
        .text("encrypt_password", "")
        .text("encrypt_password_confirm", "")
        .text("restorearea", "")
        .part(
            "conffile",
            match reqwest::blocking::multipart::Part::file(format!(
                r"Backups\{}",
                filename
            )) {
                Ok(file) => file,
                Err(error) => return Err(error.to_string()),
            },
        )
        .text("decrypt", "yes")
        .text("decrypt_password", "ok")
        .text("restore", "Restore Configuration");
    let response = client
        .post(PFSENSE_BACKUP_PAGE_URL)
        .multipart(form)
        .send()
        .unwrap();

    if response.status().is_success() {
        Ok("Config file restored successfully.".to_string())
    } else {
        Err("Unable to restore config file.".to_string())
    }
}
