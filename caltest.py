import json
import os
import threading
import webbrowser
from datetime import datetime, timedelta
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlencode, urlparse, parse_qs
from urllib.request import Request, urlopen

CREDENTIALS_FILE = "calendar_client_secret.json"
TOKEN_FILE = "token.json"

SCOPE = "https://www.googleapis.com/auth/calendar.readonly"
AUTH_URL = "https://accounts.google.com/o/oauth2/v2/auth"
TOKEN_URL = "https://oauth2.googleapis.com/token"
CALENDAR_URL = "https://www.googleapis.com/calendar/v3/calendars/primary/events"


def load_credentials():
    with open(CREDENTIALS_FILE) as f:
        data = json.load(f)

    # Desktop OAuth credentials are nested under "installed".
    return data["installed"]


def save_token(token):
    with open(TOKEN_FILE, "w") as f:
        json.dump(token, f)


def load_token():
    if not os.path.exists(TOKEN_FILE):
        return None

    with open(TOKEN_FILE) as f:
        return json.load(f)


def http_post(url, data):
    body = urlencode(data).encode()

    print(body)
    request = Request(
        url,
        data=body,
        headers={"Content-Type": "application/x-www-form-urlencoded"},
    )
    print(request)
    a = urlopen(request)
    print(a)

    with a as response:
        print(response)
        print(json.load(response))
        return json.load(response)


def authorize(credentials):
    # Let the OS choose an unused local port.
    server = HTTPServer(("127.0.0.1", 0), CallbackHandler)
    port = server.server_port
    redirect_uri = f"http://127.0.0.1:{port}/"

    params = {
        "client_id": credentials["client_id"],
        "redirect_uri": redirect_uri,
        "response_type": "code",
        "scope": SCOPE,
        "access_type": "offline",
        "prompt": "consent",
    }

    url = AUTH_URL + "?" + urlencode(params)

    print("Opening Google authorization page...")
    print(url)
    webbrowser.open(url)

    # Wait for Google's redirect to our local server.
    server.handle_request()

    code = CallbackHandler.code
    if not code:
        raise RuntimeError("Authorization failed.")

    token = http_post(
        TOKEN_URL,
        {
            "code": code,
            "client_id": credentials["client_id"],
            "client_secret": credentials["client_secret"],
            "redirect_uri": redirect_uri,
            "grant_type": "authorization_code",
        },
    )

    save_token(token)
    return token


class CallbackHandler(BaseHTTPRequestHandler):
    code = None

    def do_GET(self):
        query = parse_qs(urlparse(self.path).query)

        CallbackHandler.code = query.get("code", [None])[0]

        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.end_headers()
        self.wfile.write(
            b"<html><body>"
            b"<h2>Authorization complete.</h2>"
            b"You can close this window."
            b"</body></html>"
        )

    def log_message(self, format, *args):
        pass


def get_access_token(credentials):
    token = load_token()

    if token and "refresh_token" in token:
        # Get a fresh access token using the saved refresh token.
        new_token = http_post(
            TOKEN_URL,
            {
                "client_id": credentials["client_id"],
                "client_secret": credentials["client_secret"],
                "refresh_token": token["refresh_token"],
                "grant_type": "refresh_token",
            },
        )

        # Google normally doesn't return the refresh token again.
        new_token["refresh_token"] = token["refresh_token"]
        save_token(new_token)

        return new_token["access_token"]

    # First run: perform interactive authorization.
    token = authorize(credentials)
    return token["access_token"]


def get_today_events(access_token):
    now = datetime.now().astimezone()

    start = now.replace(
        hour=0, minute=0, second=0, microsecond=0
    )
    end = start + timedelta(days=1)

    params = {
        "timeMin": start.isoformat(),
        "timeMax": end.isoformat(),
        "singleEvents": "true",
        "orderBy": "startTime",
    }

    url = CALENDAR_URL + "?" + urlencode(params)

    request = Request(
        url,
        headers={
            "Authorization": f"Bearer {access_token}",
            "Accept": "application/json",
        },
    )

    with urlopen(request) as response:
        return json.load(response).get("items", [])


def format_time(value):
    if "dateTime" not in value:
        return "All day"

    dt = datetime.fromisoformat(
        value["dateTime"].replace("Z", "+00:00")
    )

    return dt.astimezone().strftime("%I:%M %p").lstrip("0")


def main():
    credentials = load_credentials()
    access_token = get_access_token(credentials)
    events = get_today_events(access_token)

    if not events:
        print("No events today.")
        return

    for event in events:
        title = event.get("summary", "(No title)")
        start = format_time(event["start"])
        end = format_time(event["end"])

        print(f"{start} - {end}: {title}")


if __name__ == "__main__":
    main()
