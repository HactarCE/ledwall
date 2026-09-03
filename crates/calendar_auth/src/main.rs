#![allow(clippy::expect_fun_call)]

use std::net::TcpListener;

use serde::Deserialize;
use tinyweb::{Config, ContentType, Request, Response};

const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
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
    auth_uri: String,
    token_uri: String,
    // auth_provider_x509_cert_url: String,
    client_secret: String,
    // redirect_uris: Vec<String>,
}

fn urlencode_params<'a>(params: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut first = true;
    let mut ret = String::new();
    for (k, v) in params {
        ret += if std::mem::take(&mut first) { "?" } else { "&" };
        ret += k;
        ret += "=";
        for s in url::form_urlencoded::byte_serialize(v.as_bytes()) {
            ret += s;
        }
    }
    ret
}

fn main() {
    let secret: GoogleCredentials = serde_json::from_slice(
        &std::fs::read(CREDENTIALS_FILE).expect(&format!("missing {CREDENTIALS_FILE}")),
    )
    .expect(&format!("missing {CREDENTIALS_FILE}"));

    let listener = TcpListener::bind("localhost:0").unwrap_or_else(|e| {
        eprintln!("bind: {e}");
        std::process::exit(1);
    });
    let port = listener.local_addr().unwrap().port();

    let redirect_uri = format!("http://localhost:{port}");

    let url = secret.installed.auth_uri.clone()
        + &urlencode_params([
            ("client_id", secret.installed.client_id.as_str()),
            ("redirect_uri", &redirect_uri),
            ("response_type", "code"),
            ("scope", SCOPE),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ]);

    println!("Click this link: {url}");

    tinyweb::serve(listener, Config::default(), move |req: &Request| {
        if req.path == "/"
            && let Some(code) = req.query.get("code")
        {
            let token_json = ureq::post(&secret.installed.token_uri)
                .send_form([
                    ("code", code[0].as_str()),
                    ("client_id", &secret.installed.client_id),
                    ("client_secret", &secret.installed.client_secret),
                    ("redirect_uri", &redirect_uri),
                    ("grant_type", "authorization_code"),
                ])
                .expect("error getting auth code")
                .into_body()
                .read_to_string()
                .unwrap();
            std::fs::write(TOKEN_FILE, token_json).unwrap();
            println!("Success! Token stored in {TOKEN_FILE}");
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(2));
                std::process::exit(0);
            });
            Response::ok(ContentType::HTML, "<h1>Success!</h1>")
        } else {
            Response::not_found()
        }
    });
}
