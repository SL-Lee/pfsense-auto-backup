# pfsense-auto-backup

pfsense-auto-backup is a tool that helps you to automatically and securely backup your pfSense configurations at a specified interval.

## Configuration

The following environment variables must be set before running the tool:

- `BACKUP_SCHEDULE`: Specifies the schedule for backups in the format `<quantity><time-unit>`, where `<quantity>` is a numeric digit and `<time-unit>` can be one of the following: `min`, `hr`, `d`, or `wk`. Example backup schedules include `30min`, `12hr` or `1d`.
- `ENCRYPTION_PASSPHRASE`: The passphrase from which the Key-Encryption-Key (KEK) will be derived from.
- `PFSENSE_DOMAIN`: The domain address where pfSense is located, with the `https://` prefix and **NO** trailing slash. Example domains include `https://192.168.1.1` and `https://pfsense.example.com`.
- `PFSENSE_USERNAME`: The username to authenticate with when logging in to pfSense.
- `PFSENSE_PASSWORD`: THe password to authenticate with when logging in to pfSense.
