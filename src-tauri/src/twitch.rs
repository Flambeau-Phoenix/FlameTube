//! Twitch: signing in, and which of the channels the account follows are live.
//!
//! Twitch's API needs a registered application, so the Client ID comes from the user's own app at
//! dev.twitch.tv/console/apps — the Twitch page has a field for it. Sign-in is the device code
//! grant, which needs no client secret: nothing secret ships in the build, and the token never
//! leaves this process (the webview only ever sees the list of live channels). `user:read:follows`
//! is the only scope asked for, and it is read-only.
//!
//! `helix/streams/followed` is already "live only": an offline channel is absent from it, so there
//! is no per-channel status query to make, just one request per refresh.

use crate::http;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;

const DEVICE_URL: &str = "https://id.twitch.tv/oauth2/device";
const TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const HELIX: &str = "https://api.twitch.tv/helix";
const SCOPES: &str = "user:read:follows";
const UA: &str = concat!("FlameTube/", env!("CARGO_PKG_VERSION"));
const CLIENT_ID_KEY: &str = "twitch_client_id";
const TOKENS_KEY: &str = "twitch_tokens";
/// Refresh a little early: a token that expires mid-request would fail the refresh that was meant
/// to save it, and this compares against the system clock, which is not trusted to the second.
const REFRESH_MARGIN: i64 = 120;
const TIMEOUT: Duration = Duration::from_secs(15);
/// Helix's maximum page, and far more streams than the rail can show.
const PAGE: usize = 100;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    client_id_set: bool,
    /// The user's own Client ID, echoed back so the field can be prefilled. It is not a secret: it
    /// is public in every Twitch app, and useless without the token that stays on this side.
    client_id: Option<String>,
    signed_in: bool,
    login: Option<String>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct Tokens {
    access_token: String,
    refresh_token: String,
    /// Unix seconds. Twitch sends `expires_in`; storing the deadline keeps the check one compare.
    expires_at: i64,
    user_id: String,
    login: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    generation: u64,
    user_code: String,
    verification_uri: String,
    device_code: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveChannel {
    id: String,
    /// The channel name to embed. Names are stored lowercased by Twitch, which is what the player
    /// and the chat embed expect.
    login: String,
    /// The channel's display name, which keeps its capitalisation.
    name: String,
    title: String,
    game: String,
    viewers: u64,
    thumbnail: String,
    started_at: String,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn client_id(state: &AppState) -> Option<String> {
    state
        .db
        .get_setting(CLIENT_ID_KEY)
        .map(|raw| raw.trim().to_owned())
        .filter(|raw| !raw.is_empty())
}

fn tokens(state: &AppState) -> Option<Tokens> {
    state
        .db
        .get_setting(TOKENS_KEY)
        .and_then(|raw| serde_json::from_str::<Tokens>(&raw).ok())
        .filter(|session| !session.access_token.is_empty())
}

fn commit_session(expected: u64, commit: impl FnOnce()) -> Result<(), String> {
    let _guard = session_writes().lock().unwrap();
    if generation().load(Ordering::SeqCst) != expected { return Err("Sign-in cancelled".into()); }
    commit();
    Ok(())
}

fn save_tokens(state: &AppState, session: &Tokens, expected: u64) -> Result<(), String> {
    let raw = serde_json::to_string(session).map_err(|_| "Could not save Twitch session")?;
    commit_session(expected, || state.db.set_setting(TOKENS_KEY, &raw))
}

fn forget_if_current(state: &AppState, expected: u64) {
    let _ = commit_session(expected, || {
        generation().fetch_add(1, Ordering::SeqCst);
        state.db.delete_setting(TOKENS_KEY);
    });
}

fn forget_tokens(state: &AppState) {
    let _guard = session_writes().lock().unwrap();
    generation().fetch_add(1, Ordering::SeqCst);
    state.db.delete_setting(TOKENS_KEY);
}

fn status_of(state: &AppState) -> Status {
    let id = client_id(state);
    let session = tokens(state);
    Status {
        client_id_set: id.is_some(),
        client_id: id,
        signed_in: session.is_some(),
        login: session.map(|session| session.login),
    }
}

/// Twitch Client IDs are 30 lowercase alphanumerics. Accept any plausible token of that shape so a
/// pasted ID with a stray space still works, and reject anything that could not be one, rather than
/// sending it to Twitch and surfacing their generic error.
fn valid_client_id(raw: &str) -> bool {
    let id = raw.trim();
    (10..=64).contains(&id.len()) && id.chars().all(|c| c.is_ascii_alphanumeric())
}

fn needs_refresh(expires_at: i64, now: i64) -> bool {
    expires_at - REFRESH_MARGIN <= now
}

/// Helix thumbnails arrive as templates; the rail renders them at one size only.
fn thumbnail(url: &str) -> String {
    url.replace("{width}", "440").replace("{height}", "248")
}

fn cards_from_streams(body: &serde_json::Value) -> Vec<LiveChannel> {
    let mut cards: Vec<LiveChannel> = body
        .get("data")
        .and_then(serde_json::Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    // A stream entry without a login can't be embedded, so it can't be shown.
                    let login = row.get("user_login")?.as_str()?;
                    Some(LiveChannel {
                        id: row.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_owned(),
                        login: login.to_owned(),
                        name: row
                            .get("user_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(login)
                            .to_owned(),
                        title: row.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_owned(),
                        game: row
                            .get("game_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_owned(),
                        viewers: row.get("viewer_count").and_then(|v| v.as_u64()).unwrap_or(0),
                        thumbnail: thumbnail(
                            row.get("thumbnail_url").and_then(|v| v.as_str()).unwrap_or_default(),
                        ),
                        started_at: row
                            .get("started_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    // Most-watched first: the rail is one row, so the channels a viewer is most likely to want
    // must not sit off the right edge behind ones they would not have picked.
    cards.sort_by(|a, b| b.viewers.cmp(&a.viewers));
    cards
}

/// What one device-grant poll reply means. Anything unrecognised is fatal: retrying a reply this
/// build cannot read would just burn the code's lifetime.
#[derive(Debug)]
enum Step {
    Pending,
    SlowDown,
    Fatal(String),
}

fn device_step(status: u16, body: &serde_json::Value) -> Step {
    match body.get("message").and_then(|v| v.as_str()) {
        Some("authorization_pending") => Step::Pending,
        Some("slow_down") => Step::SlowDown,
        Some("expired_token") => Step::Fatal("That sign-in code expired — try again".to_owned()),
        Some("access_denied") => Step::Fatal("Twitch sign-in was denied".to_owned()),
        Some(other) => Step::Fatal(format!("Twitch: {other}")),
        None => Step::Fatal(format!("Twitch sign-in failed ({status})")),
    }
}

async fn post_form(
    url: &str,
    params: &[(&str, &str)],
) -> Result<(u16, serde_json::Value), String> {
    let response = http::client()
        .post(url)
        .header("User-Agent", UA)
        .form(params)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Twitch: {e}"))?;
    let status = response.status().as_u16();
    let text = response.text().await.map_err(|e| format!("Twitch: {e}"))?;
    Ok((status, serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)))
}

async fn helix(
    client_id: &str,
    token: &str,
    path: &str,
    query: &[(&str, String)],
) -> Result<serde_json::Value, String> {
    let mut request = http::client()
        .get(format!("{HELIX}{path}"))
        .header("Client-Id", client_id)
        .bearer_auth(token)
        .timeout(TIMEOUT);
    if !query.is_empty() {
        request = request.query(query);
    }
    let response = request.send().await.map_err(|e| format!("Twitch: {e}"))?;
    let status = response.status().as_u16();
    let text = response.text().await.map_err(|e| format!("Twitch: {e}"))?;
    if !(200..300).contains(&status) {
        let body: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
        let message = body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("request failed");
        return Err(format!("Twitch: {message} ({status})"));
    }
    serde_json::from_str(&text).map_err(|_| "Twitch sent a reply this build cannot read".to_owned())
}

/// A second sign-in (or a cancel) bumps this; an in-flight poll notices on its next tick, so two
/// codes can never race to write the session.
fn session_writes() -> &'static Mutex<()> {
    static LOCK: Mutex<()> = Mutex::new(());
    &LOCK
}

fn generation() -> &'static AtomicU64 {
    static GEN: OnceLock<AtomicU64> = OnceLock::new();
    GEN.get_or_init(|| AtomicU64::new(0))
}

#[tauri::command]
pub async fn twitch_status(state: State<'_, Arc<AppState>>) -> Result<Status, String> {
    if let Some(id) = client_id(&state) { if tokens(&state).is_some() { let _ = active_session(&state, &id).await; } }
    Ok(status_of(&state))
}

#[tauri::command]
pub fn twitch_set_client_id(
    state: State<'_, Arc<AppState>>,
    client_id: String,
) -> Result<Status, String> {
    let id = client_id.trim().to_owned();
    if !valid_client_id(&id) {
        return Err("That does not look like a Twitch Client ID (30 letters and digits)".to_owned());
    }
    let changed = self::client_id(&state).as_deref() != Some(id.as_str());
    state.db.set_setting(CLIENT_ID_KEY, &id);
    // A token belongs to the app that issued it; keeping it after the Client ID changes would
    // query Helix with a mismatch and fail every refresh.
    if changed {
        forget_tokens(&state);
    }
    Ok(status_of(&state))
}

#[tauri::command]
pub fn twitch_sign_out(state: State<'_, Arc<AppState>>) {
    forget_tokens(&state);
}

#[tauri::command]
pub fn twitch_device_cancel() {
    let _guard = session_writes().lock().unwrap();
    generation().fetch_add(1, Ordering::SeqCst);
}

/// Asks Twitch for a code the user types at twitch.tv/activate.
#[tauri::command]
pub async fn twitch_device_start(state: State<'_, Arc<AppState>>) -> Result<DeviceCode, String> {
    let gen = generation().load(Ordering::SeqCst);
    let id = client_id(&state).ok_or("Add your Twitch Client ID first")?;
    let (status, body) = post_form(DEVICE_URL, &[("client_id", &id), ("scopes", SCOPES)]).await?;
    if status != 200 {
        let message = body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Twitch rejected that Client ID");
        return Err(format!("{message} ({status})"));
    }
    let field = |name: &str| body.get(name).and_then(|v| v.as_str()).map(str::to_owned);
    let user_code = field("user_code").ok_or("Twitch sent no sign-in code")?;
    let device_code = field("device_code").ok_or("Twitch sent no sign-in code")?;
    Ok(DeviceCode {
        generation: gen,
        user_code,
        verification_uri: field("verification_uri")
            .unwrap_or_else(|| "https://www.twitch.tv/activate".to_owned()),
        device_code,
        expires_in: body.get("expires_in").and_then(|v| v.as_u64()).unwrap_or(1800),
        interval: body.get("interval").and_then(|v| v.as_u64()).unwrap_or(5).max(1),
    })
}

/// Polls until the user approves, the code expires, or [`twitch_device_cancel`] bumps the
/// generation. Resolves to the signed-in login so the page can show who is connected.
#[tauri::command]
pub async fn twitch_device_wait(
    state: State<'_, Arc<AppState>>,
    device_code: String,
    interval: u64,
    expires_in: u64,
    expected_generation: u64,
) -> Result<String, String> {
    let client_id = client_id(&state).ok_or("Add your Twitch Client ID first")?;
    let gen = expected_generation;
    // Twitch's own deadline, plus a little slack for the clock; never longer than the code lives.
    let deadline = now_secs() + expires_in.min(1800) as i64 + 5;
    let mut every = interval.clamp(1, 30);
    loop {
        tokio::time::sleep(Duration::from_secs(every)).await;
        if generation().load(Ordering::SeqCst) != gen {
            return Err("Sign-in cancelled".to_owned());
        }
        if now_secs() > deadline {
            return Err("That sign-in code expired — try again".to_owned());
        }
        let polled = post_form(
            TOKEN_URL,
            &[
                ("client_id", client_id.as_str()),
                ("scopes", SCOPES),
                ("device_code", device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ],
        )
        .await;
        // A dropped connection is not the user's fault and the code is still good, so keep polling
        // until the deadline instead of failing the sign-in on one bad packet.
        let (status, body) = match polled {
            Ok(reply) => reply,
            Err(e) => {
                tracing::debug!(error = %e, "twitch device poll failed, retrying");
                continue;
            }
        };
        if status == 200 {
            return store_session(&state, &client_id, &body, gen).await;
        }
        match device_step(status, &body) {
            Step::Pending => {}
            Step::SlowDown => every = (every + 5).min(30),
            Step::Fatal(message) => return Err(message),
        }
    }
}

async fn store_session(
    state: &AppState,
    client_id: &str,
    body: &serde_json::Value,
    gen: u64,
) -> Result<String, String> {
    let access = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Twitch sent no access token")?;
    let refresh = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let expires_in = body.get("expires_in").and_then(|v| v.as_i64()).unwrap_or(0);
    // Helix needs the numeric user id for `streams/followed`, and the rail shows the login, so the
    // one `/users` call at sign-in supplies both and neither has to be asked for again.
    let profile = helix(client_id, access, "/users", &[]).await?;
    let user = profile
        .get("data")
        .and_then(|v| v.as_array())
        .and_then(|rows| rows.first())
        .ok_or("Twitch did not return the signed-in account")?;
    let login = user
        .get("login")
        .and_then(|v| v.as_str())
        .ok_or("Twitch did not return the signed-in account")?
        .to_owned();
    let user_id = user
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned();
    save_tokens(
        state,
        &Tokens {
            access_token: access.to_owned(),
            refresh_token: refresh.to_owned(),
            expires_at: now_secs() + expires_in,
            user_id,
            login: login.clone(),
        },
        gen,
    )?;
    tracing::info!(user = %login, "twitch connected");
    Ok(login)
}

async fn refresh(state: &AppState, client_id: &str, session: &Tokens, gen: u64) -> Result<Tokens, String> {
    if session.refresh_token.is_empty() {
        forget_if_current(state, gen);
        return Err("Twitch session expired — sign in again".to_owned());
    }
    let (status, body) = post_form(
        TOKEN_URL,
        &[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", session.refresh_token.as_str()),
        ],
    )
    .await?;
    if status != 200 {
        forget_if_current(state, gen);
        return Err("Twitch session expired — sign in again".to_owned());
    }
    let access = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Twitch sent no access token")?
        .to_owned();
    let refreshed = Tokens {
        access_token: access,
        // Twitch rotates the refresh token on some grants and repeats the old one on others.
        refresh_token: body
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| session.refresh_token.clone()),
        expires_at: now_secs()
            + body.get("expires_in").and_then(|v| v.as_i64()).unwrap_or(0),
        ..session.clone()
    };
    save_tokens(state, &refreshed, gen)?;
    Ok(refreshed)
}


/// Serialize token refresh and validate each token at startup and at least hourly.
async fn active_session(state: &AppState, id: &str) -> Result<Tokens, String> {
    static GATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    static VALIDATED: Mutex<Option<(String, i64)>> = Mutex::new(None);
    let _gate = GATE.lock().await;
    if client_id(state).as_deref() != Some(id) { return Err("Twitch account changed".into()); }
    let gen = generation().load(Ordering::SeqCst);
    let mut session = tokens(state).ok_or("Sign in to Twitch")?;
    if needs_refresh(session.expires_at, now_secs()) {
        session = refresh(state, id, &session, gen).await?;
    }
    let valid = VALIDATED.lock().unwrap().as_ref().is_some_and(|(token, at)| token == &session.access_token && now_secs() - at < 3500);
    if !valid {
        let response = http::client().get("https://id.twitch.tv/oauth2/validate")
            .bearer_auth(&session.access_token).timeout(TIMEOUT).send().await.map_err(|_| "Could not validate Twitch session")?;
        if response.status().as_u16() == 401 {
            forget_if_current(state, gen);
            return Err("Twitch session expired - sign in again".into());
        }
        let body: serde_json::Value = response.error_for_status().map_err(|_| "Twitch session validation failed")?.json().await.map_err(|_| "Invalid Twitch validation reply")?;
        if body["client_id"].as_str() != Some(id) || body["user_id"].as_str() != Some(session.user_id.as_str()) ||
            !body["scopes"].as_array().is_some_and(|scopes| scopes.iter().any(|scope| scope.as_str() == Some(SCOPES))) {
            forget_if_current(state, gen);
            return Err("Reconnect Twitch to grant access to followed channels".into());
        }
        if generation().load(Ordering::SeqCst) != gen { return Err("Sign-in cancelled".into()); }
        *VALIDATED.lock().unwrap() = Some((session.access_token.clone(), now_secs()));
    }
    if generation().load(Ordering::SeqCst) != gen { return Err("Sign-in cancelled".into()); }
    Ok(session)
}

/// The followed channels that are live right now, most watched first.
#[tauri::command]
pub async fn twitch_followed_live(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<LiveChannel>, String> {
    let client_id = client_id(&state).ok_or("Add your Twitch Client ID first")?;
    let session = active_session(&state, &client_id).await?;
    let body = match helix(
        &client_id,
        &session.access_token,
        "/streams/followed",
        &[
            ("user_id", session.user_id.clone()),
            ("first", PAGE.to_string()),
        ],
    )
    .await
    {
        Ok(body) => body,
        // The refresh token is the long-lived one, so a 401 after a refresh means Twitch revoked
        // the session; keeping it would fail every sixty seconds from here on.
        Err(e) if e.contains("(401)") => {
            forget_tokens(&state);
            return Err("Twitch session expired — sign in again".to_owned());
        }
        Err(e) => return Err(e),
    };
    Ok(cards_from_streams(&body))
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcasterProfile {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub description: String,
    pub profile_image_url: String,
    pub broadcaster_type: String,
    pub view_count: u64,
    pub game_name: String,
    pub title: String,
    pub is_live: bool,
    pub viewer_count: u64,
    pub started_at: String,
}

/// Fetches broadcaster profile and live stream info by channel login.
#[tauri::command]
pub async fn twitch_broadcaster_profile(
    state: State<'_, Arc<AppState>>,
    login: String,
) -> Result<BroadcasterProfile, String> {
    let clean_login = login.trim().to_lowercase();
    let client_id = client_id(&state).ok_or("Add your Twitch Client ID first")?;
    let session = active_session(&state, &client_id).await?;
    let user_res = helix(
        &client_id,
        &session.access_token,
        "/users",
        &[("login", clean_login.clone())],
    )
    .await?;
    let user_data = user_res
        .get("data")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| "Channel not found".to_owned())?;

    let id = user_data.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_owned();
    let display_name = user_data
        .get("display_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&clean_login)
        .to_owned();
    let description = user_data
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned();
    let profile_image_url = user_data
        .get("profile_image_url")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned();
    let broadcaster_type = user_data
        .get("broadcaster_type")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned();
    let view_count = user_data.get("view_count").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut is_live = false;
    let mut viewer_count = 0;
    let mut started_at = String::new();
    let mut title = String::new();
    let mut game_name = String::new();

    if let Ok(stream_res) = helix(
        &client_id,
        &session.access_token,
        "/streams",
        &[("user_id", id.clone())],
    )
    .await
    {
        if let Some(stream_data) = stream_res
            .get("data")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
        {
            is_live = true;
            viewer_count = stream_data.get("viewer_count").and_then(|v| v.as_u64()).unwrap_or(0);
            started_at = stream_data
                .get("started_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned();
            title = stream_data
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned();
            game_name = stream_data
                .get("game_name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned();
        }
    }

    if title.is_empty() {
        if let Ok(channel_res) = helix(
            &client_id,
            &session.access_token,
            "/channels",
            &[("broadcaster_id", id.clone())],
        )
        .await
        {
            if let Some(ch) = channel_res
                .get("data")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
            {
                title = ch.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_owned();
                game_name = ch
                    .get("game_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_owned();
            }
        }
    }

    Ok(BroadcasterProfile {
        id,
        login: clean_login,
        display_name,
        description,
        profile_image_url,
        broadcaster_type,
        view_count,
        game_name,
        title,
        is_live,
        viewer_count,
        started_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cancelled_session_cannot_commit_after_network_returns() {
        use std::sync::atomic::AtomicBool;
        let before = generation().load(Ordering::SeqCst);
        twitch_device_cancel();
        let wrote = AtomicBool::new(false);
        assert!(commit_session(before, || wrote.store(true, Ordering::SeqCst)).is_err());
        assert!(!wrote.load(Ordering::SeqCst));
        let current = generation().load(Ordering::SeqCst);
        commit_session(current, || wrote.store(true, Ordering::SeqCst)).unwrap();
        assert!(wrote.load(Ordering::SeqCst));
    }

    #[test]
    fn client_ids_must_look_like_client_ids() {
        assert!(valid_client_id(" kimne78kx3ncx6brgo4mv6wki5h1ko "));
        assert!(!valid_client_id(""));
        assert!(!valid_client_id("short"));
        assert!(!valid_client_id("has a space in it"));
        assert!(!valid_client_id("https://dev.twitch.tv/apps"));
    }

    #[test]
    fn refresh_runs_before_the_deadline_not_after_it() {
        assert!(needs_refresh(1_000, 900)); // inside the margin
        assert!(!needs_refresh(1_000, 700));
        assert!(needs_refresh(0, 0)); // no expiry stored at all
    }

    #[test]
    fn streams_become_cards_sorted_by_viewers_with_a_sized_thumbnail() {
        let body = json!({ "data": [
            { "id": "1", "user_login": "quiet", "user_name": "Quiet", "title": "t", "game_name": "G",
              "viewer_count": 3, "thumbnail_url": "https://x/{width}x{height}.jpg",
              "started_at": "2026-11-01T00:00:00Z" },
            { "id": "2", "user_login": "loud", "user_name": "Loud", "title": "t2", "game_name": "G2",
              "viewer_count": 900, "thumbnail_url": "https://y/{width}x{height}.jpg",
              "started_at": "2026-11-01T00:00:00Z" },
            { "user_name": "no login to embed" }
        ]});
        let cards = cards_from_streams(&body);
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].login, "loud");
        assert_eq!(cards[1].name, "Quiet");
        assert_eq!(cards[0].thumbnail, "https://y/440x248.jpg");
        assert!(cards_from_streams(&json!({})).is_empty());
        assert!(cards_from_streams(&json!({ "data": "not a list" })).is_empty());
    }

    #[test]
    fn device_replies_are_classified_before_the_ui_sees_them() {
        assert!(matches!(
            device_step(400, &json!({ "message": "authorization_pending" })),
            Step::Pending
        ));
        assert!(matches!(
            device_step(400, &json!({ "message": "slow_down" })),
            Step::SlowDown
        ));
        assert!(matches!(
            device_step(400, &json!({ "message": "expired_token" })),
            Step::Fatal(_)
        ));
        assert!(matches!(
            device_step(400, &json!({ "message": "access_denied" })),
            Step::Fatal(_)
        ));
        // A body this build cannot read must end the sign-in, not loop on it.
        assert!(matches!(device_step(500, &json!({})), Step::Fatal(_)));
    }
}
