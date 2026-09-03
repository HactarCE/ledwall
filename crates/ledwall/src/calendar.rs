use chrono::{DateTime, Local};
use serde::Deserialize;

const API_URL: &str = "https://www.googleapis.com/calendar/v3";

#[derive(Deserialize, Debug)]
pub struct CalendarEvents {
    pub items: Vec<CalendarEvent>,
}

#[derive(Deserialize, Debug)]
pub struct CalendarEvent {
    pub status: CalendarEventStatus,
    pub start: Option<CalendarInstant>,
    pub end: Option<CalendarInstant>,
}

#[derive(Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CalendarEventStatus {
    #[default]
    Confirmed,
    Tentative,
    Cancelled,
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarInstant {
    #[serde(default)]
    pub date_time: Option<DateTime<Local>>,
}

/// Returns a list of events for the given calendar ID and time range.
///
/// Returns `None` if authentication has not yet succeeded or if there is some
/// other error. In the event of another error, a message is emitted to stderr.
pub fn get_events(
    calendar_id: &str,
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Option<CalendarEvents> {
    let token = auth::get_access_token()?;

    let json_result: Result<serde_json::Value, _> =
        ureq::get(format!("{API_URL}/calendars/{calendar_id}/events"))
            .query("timeMin", start.to_rfc3339())
            .query("timeMax", end.to_rfc3339())
            .query("singleEvents", "true")
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/json")
            .call()
            .and_then(|response| response.into_body().read_json());

    match json_result {
        Ok(json) => match serde_json::from_value(json.clone()) {
            Ok(calendar_events) => return Some(calendar_events),
            Err(e) => {
                eprintln!("calendar json deserialize error: {e}");
                eprintln!("json: {json}");
            }
        },
        Err(e) => eprintln!("calendar json parse error: {e}"),
    }
    None
}

pub use auth::wait_for_access_token;

mod auth {
    use std::sync::{Arc, Condvar, LazyLock, Mutex};
    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

    /// How often to try again if requesting an access token fails.
    const REFRESH_TOKEN_RETRY_FREQ_SECONDS: u64 = 60;
    /// Number of seconds before expiry to request a new access token.
    const REFRESH_TOKEN_BUFFER_SECONDS: u64 = 60;

    const CREDENTIALS_FILE: &str = "calendar_credentials.json";
    const TOKEN_FILE: &str = "calendar_token.json";

    #[derive(Deserialize, Debug)]
    struct GoogleCredentials {
        installed: GoogleCredentialsInner,
    }

    #[derive(Deserialize, Debug)]
    struct GoogleCredentialsInner {
        client_id: String,
        // project_id: String,
        // auth_uri: String,
        // token_uri: String,
        // auth_provider_x509_cert_url: String,
        client_secret: String,
        // redirect_uris: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Token {
        access_token: String,
        expires_in: u64,
        #[serde(default)]
        refresh_token: String,
        scope: String,
        token_type: String,
        refresh_token_expires_in: u64,
    }

    static ACCESS_TOKEN_CONDVAR: Condvar = Condvar::new();
    static ACCESS_TOKEN: LazyLock<Arc<Mutex<Option<String>>>> = LazyLock::new(|| {
        let arc_mutex = Arc::new(Mutex::new(None));
        let arc_mutex_ref = Arc::clone(&arc_mutex);
        std::thread::spawn(move || {
            loop {
                match refresh_access_token() {
                    Ok(token) => {
                        *arc_mutex.lock().expect("error locking mutex") =
                            Some(token.access_token.clone());
                        ACCESS_TOKEN_CONDVAR.notify_all();
                        let sleep_time = token
                            .expires_in
                            .saturating_sub(REFRESH_TOKEN_BUFFER_SECONDS);
                        std::thread::sleep(Duration::from_secs(sleep_time));
                    }
                    Err(e) => {
                        eprintln!("error refreshing access token: {e}");
                        std::thread::sleep(Duration::from_secs(REFRESH_TOKEN_RETRY_FREQ_SECONDS));
                    }
                }
            }
        });
        arc_mutex_ref
    });

    pub fn get_access_token() -> Option<String> {
        ACCESS_TOKEN.lock().expect("error locking mutex").clone()
    }

    pub fn wait_for_access_token() {
        let mut access_token_guard = ACCESS_TOKEN.lock().expect("error locking mutex");
        if access_token_guard.is_none() {
            access_token_guard = ACCESS_TOKEN_CONDVAR
                .wait(access_token_guard)
                .expect("error waiting for access token");
        }
        drop(access_token_guard);
    }

    fn refresh_access_token() -> Result<Token, ureq::Error> {
        let creds: GoogleCredentials = serde_json::from_slice(&std::fs::read(CREDENTIALS_FILE)?)?;
        let mut token: Token = serde_json::from_slice(&std::fs::read(TOKEN_FILE)?)?;
        let new_token: Token = ureq::post(TOKEN_URL)
            .send_form([
                ("grant_type", "refresh_token"),
                ("client_id", &creds.installed.client_id),
                ("client_secret", &creds.installed.client_secret),
                ("refresh_token", &token.refresh_token),
            ])?
            .into_body()
            .read_json()?;
        token.access_token = new_token.access_token;
        if !new_token.refresh_token.is_empty() {
            token.refresh_token = new_token.refresh_token;
        }
        std::fs::write(TOKEN_FILE, &serde_json::to_vec(&token)?)?;
        Ok(token)
    }
}
