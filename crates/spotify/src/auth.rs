use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, anyhow};
use librespot_core::authentication::Credentials;
use librespot_core::cache::Cache;
use librespot_core::{Session, SessionConfig};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse as _, TokenUrl, basic::BasicClient,
};

pub const DEFAULT_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
pub const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8989/login";
pub const CREDENTIALS_FILE: &str = "credentials.json";

pub const SCOPES: &[&str] = &[
    "playlist-read-collaborative",
    "playlist-read-private",
    "streaming",
    "user-follow-read",
    "user-library-read",
    "user-read-email",
    "user-read-playback-state",
    "user-read-private",
    "user-read-recently-played",
    "user-top-read",
];

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub client_id: String,
    pub redirect_uri: String,
    pub cache_dir: PathBuf,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("music_tools")
            .join("spotify");
        Self {
            client_id: DEFAULT_CLIENT_ID.to_owned(),
            redirect_uri: DEFAULT_REDIRECT_URI.to_owned(),
            cache_dir,
        }
    }
}

impl AuthConfig {
    pub fn new(
        client_id: impl Into<String>,
        redirect_uri: impl Into<String>,
        cache_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            redirect_uri: redirect_uri.into(),
            cache_dir: cache_dir.into(),
        }
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(client_id) = std::env::var("SPOTIFY_CLIENT_ID") {
            config.client_id = client_id;
        }
        if let Ok(redirect_uri) = std::env::var("SPOTIFY_REDIRECT_URI") {
            config.redirect_uri = redirect_uri;
        }
        config
    }

    /// The stored credentials file path that librespot writes after a successful connection.
    pub fn file(&self) -> PathBuf {
        self.cache_dir.join(CREDENTIALS_FILE)
    }
}

/// Extracts a host:port socket address from a redirect URI string.
pub fn socket_address(uri: &str) -> Option<String> {
    let rest = uri
        .strip_prefix("http://")
        .or_else(|| uri.strip_prefix("https://"))?;
    let authority = rest.split('/').next().filter(|host| !host.is_empty())?;
    match authority.rsplit_once(':') {
        Some((_, port)) if port.chars().all(|digit| digit.is_ascii_digit()) => {
            Some(authority.to_owned())
        }
        _ => Some(format!("{authority}:80")),
    }
}

/// Unblocks any local TCP listener by sending a dummy request.
pub fn release(config: &AuthConfig) {
    let Some(address) = socket_address(&config.redirect_uri) else {
        log::warn!("auth: cannot read a socket address from {}", config.redirect_uri);
        return;
    };
    let Ok(mut stream) = std::net::TcpStream::connect(address) else {
        return;
    };
    let _ = stream.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
}

/// Secures credentials file permissions to 0o600 on Unix systems.
pub fn secure(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if let Err(error) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
            if error.kind() != std::io::ErrorKind::NotFound {
                log::warn!("credentials: cannot restrict {}: {error}", path.display());
            }
        }
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// Removes stored credentials from disk.
pub fn forget(config: &AuthConfig) {
    let file = config.file();
    if let Err(error) = std::fs::remove_file(&file) {
        if error.kind() != std::io::ErrorKind::NotFound {
            log::warn!("credentials: cannot remove {}: {error}", file.display());
        }
    }
}

/// Instantiates a librespot Session configured with local cache storage.
pub fn session(config: &AuthConfig) -> Result<Session> {
    if let Some(parent) = config.cache_dir.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::create_dir_all(&config.cache_dir).ok();

    let cache = Cache::new(Some(config.cache_dir.as_path()), None, None, None)
        .with_context(|| format!("cannot open cache at {}", config.cache_dir.display()))?;

    let session_config = SessionConfig {
        client_id: config.client_id.clone(),
        ..Default::default()
    };

    Ok(Session::new(session_config, Some(cache)))
}

/// Restores a previously saved session if credentials exist in the cache.
pub async fn restore(config: &AuthConfig) -> Result<Option<Session>> {
    let session = session(config)?;
    let Some(credentials) = session.cache().and_then(|cache| cache.credentials()) else {
        return Ok(None);
    };

    session
        .connect(credentials, true)
        .await
        .context("Failed to reconnect with cached Spotify credentials")?;

    secure(&config.file());
    Ok(Some(session))
}

/// Starts an OAuth2 PKCE login flow, opening the authorization URL and awaiting callback.
pub async fn login<F>(config: &AuthConfig, on_auth_url: F) -> Result<Session>
where
    F: FnOnce(&str) + Send + 'static,
{
    let client_id = config.client_id.clone();
    let redirect_uri = config.redirect_uri.clone();

    let access_token = tokio::task::spawn_blocking(move || {
        authorize(&client_id, &redirect_uri, on_auth_url)
    })
    .await??;

    let session = session(config)?;
    session
        .connect(Credentials::with_access_token(access_token), true)
        .await
        .context("Failed to connect Spotify session with exchanged access token")?;

    secure(&config.file());
    Ok(session)
}

fn authorize<F>(client_id: &str, redirect_uri: &str, on_auth_url: F) -> Result<String>
where
    F: FnOnce(&str),
{
    let client = BasicClient::new(ClientId::new(client_id.to_owned()))
        .set_auth_uri(AuthUrl::new(
            "https://accounts.spotify.com/authorize".to_owned(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://accounts.spotify.com/api/token".to_owned(),
        )?)
        .set_redirect_uri(RedirectUrl::new(redirect_uri.to_owned())?);

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let scopes = SCOPES.iter().map(|s| Scope::new((*s).to_owned()));
    let (url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(scopes)
        .set_pkce_challenge(challenge)
        .url();

    let url_str = url.to_string();
    on_auth_url(&url_str);
    let _ = open::that_in_background(&url_str);

    let address = socket_address(redirect_uri)
        .with_context(|| format!("redirect URI has no socket address: {redirect_uri}"))?;
    let listener = TcpListener::bind(&address)
        .with_context(|| format!("cannot listen for Spotify login callback at {address}"))?;

    let mut stream = listener
        .incoming()
        .next()
        .context("Spotify login callback did not arrive")??;

    let mut request = String::new();
    BufReader::new(&stream).read_line(&mut request)?;
    let target = request
        .split_whitespace()
        .nth(1)
        .context("Spotify login callback HTTP request was malformed")?;

    let callback = oauth2::url::Url::parse(&format!("http://localhost{target}"))?;
    let query = callback
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();

    let response_body = "<html><body><h2>Authenticated! You can return to Music Tools.</h2></body></html>";
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
        response_body.len()
    )?;

    if let Some(error) = query.get("error") {
        return Err(anyhow!("Spotify authorization was denied or failed: {error}"));
    }

    let code = query
        .get("code")
        .context("Spotify login callback did not contain an authorization code")?;

    let token_resp = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(verifier)
        .request(&exchange)
        .map_err(|e| anyhow!("Failed to exchange Spotify authorization code: {e}"))?;

    Ok(token_resp.access_token().secret().to_owned())
}

fn exchange(request: oauth2::HttpRequest) -> Result<oauth2::HttpResponse, reqwest::Error> {
    let sent = reqwest::blocking::Client::new().execute(request.try_into()?)?;
    let status = sent.status();
    let headers = sent.headers().clone();
    let mut response = http::Response::new(sent.bytes()?.to_vec());
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_address_parsing() {
        assert_eq!(
            socket_address("http://127.0.0.1:8989/login").as_deref(),
            Some("127.0.0.1:8989")
        );
        assert_eq!(
            socket_address("http://localhost:3000/callback").as_deref(),
            Some("localhost:3000")
        );
        assert_eq!(
            socket_address("http://localhost/login").as_deref(),
            Some("localhost:80")
        );
        assert_eq!(socket_address("invalid-url"), None);
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert_eq!(config.client_id, DEFAULT_CLIENT_ID);
        assert_eq!(config.redirect_uri, DEFAULT_REDIRECT_URI);
        assert!(config.cache_dir.to_string_lossy().contains("spotify"));
        assert_eq!(config.file(), config.cache_dir.join(CREDENTIALS_FILE));
    }
}
