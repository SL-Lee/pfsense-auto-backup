use std::{env, fs::create_dir, path::Path};

use regex::Regex;

const PFSENSE_LOGIN_PAGE_URL: &str = "https://192.168.1.10/";
const PFSENSE_BACKUP_PAGE_URL: &str = "https://192.168.1.10/diag_backup.php";

fn get_csrf_token(client: &reqwest::blocking::Client, url: &str) -> String {
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

pub fn login(client: &reqwest::blocking::Client) {
    let csrf_token = get_csrf_token(client, PFSENSE_LOGIN_PAGE_URL);
    let pfsense_username = env::var("PFSENSE_USERNAME")
        .expect("'PFSENSE_USERNAME' environment variable is not set.");
    let pfsense_password = env::var("PFSENSE_PASSWORD")
        .expect("'PFSENSE_PASSWORD' environment variable is not set.");
    let form = reqwest::blocking::multipart::Form::new()
        .text("__csrf_magic", csrf_token)
        .text("usernamefld", pfsense_username)
        .text("passwordfld", pfsense_password)
        .text("login", "Sign+In");
    let response = client
        .post(PFSENSE_LOGIN_PAGE_URL)
        .multipart(form)
        .send()
        .unwrap();

    if response.status().is_success() {
        println!("Logged in successfully.");
    } else {
        println!("Log in unsuccessful.");
    }
}

pub fn download_backup_config(client: &reqwest::blocking::Client) {
    let csrf_token = get_csrf_token(client, PFSENSE_BACKUP_PAGE_URL);
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

    match response.copy_to(&mut backup_file) {
        Ok(_) => println!("Config file backed up successfully."),
        Err(_) => println!("Unable to back up config file."),
    };
}

pub fn restore_backup_config(client: &reqwest::blocking::Client) {
    let csrf_token = get_csrf_token(client, PFSENSE_BACKUP_PAGE_URL);
    let form = reqwest::blocking::multipart::Form::new()
        .text("__csrf_magic", csrf_token)
        .text("backuparea", "")
        .text("donotbackuprrd", "yes")
        .text("encrypt_password", "")
        .text("encrypt_password_confirm", "")
        .text("restorearea", "")
        .part(
            "conffile",
            reqwest::blocking::multipart::Part::file(
                "config-pfSense-primary.home.arpa-20210814180228.xml",
            )
            .unwrap(),
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
        println!("Config file restored successfully.");
    } else {
        println!("Unable to restore config file.");
    }
}
