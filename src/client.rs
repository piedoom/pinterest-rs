//! The `Client` is used to access all API methods.
use oauth2;
use hyper;
use hyper::client::HttpConnector;
use futures::{Future, Stream};
use tokio_core::reactor::Core;

/// Base API request string
const API_BASE: &str = "https://api.pinterest.com/v1/";

/// Defines what permissions the token should grant.  By default, all are false.
pub struct Scope {
    /// Use GET method on a user’s Pins, boards.
    read_public: bool,
    /// Use PATCH, POST and DELETE methods on a user’s Pins and boards.
    write_public: bool,
    /// Use GET method on a user’s follows and followers (on boards, users and interests).
    read_relationships: bool,
    /// Use PATCH, POST and DELETE methods on a user’s follows and followers (on boards, users and interests).
    write_relationships: bool,
}

impl Default for Scope {
    fn default() -> Scope {
        Scope {
            read_public: false,
            write_public: false,
            read_relationships: false,
            write_relationships: false,
        }
    }
}

/// Defines Pinterest error types
#[derive(Debug)]
pub enum PinterestError {
    Token(oauth2::TokenError),
}

impl From<oauth2::TokenError> for PinterestError {
    fn from(err: oauth2::TokenError) -> PinterestError {
        PinterestError::Token(err)
    }
}

/// Defines general OAuth2 configuration with Pinterest specific options
pub struct Config<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    authorize_url: &'a str,
    token_url: &'a str,
    redirect_url: &'a str,
    scope: Scope,
}

/// Handles API methods
pub struct Client { 
    token: Option<oauth2::Token>,
    hyper: hyper::Client<HttpConnector>, 
    core: Core,
}

impl Default for Client {
    fn default() -> Self {
        let mut core = Core::new().unwrap();
        Client {
            token: None,
            hyper: hyper::Client::new(&core.handle()),
            core: core,
        }
    }
}

impl Client {
    pub fn new(token: oauth2::Token) -> Self {
        Client { token: Some(token), .. Client::default() }
    }
}

/// Handles authentication with OAuth flow
pub struct TokenBuilder { 
    config: oauth2::Config,
}

impl TokenBuilder {
    /// Create and return a `TokenBuilder` structure using a custom config type.  Note that
    /// the parameter is not the same as `oauth2::Config`, but `pinterest::client::Config`.
    /// After generating the authorization URL, the application implementer must
    /// find a way to listen to the callback URL and receive an access token.  Please
    /// see [the oauth2 documentation](https://github.com/ramosbugs/oauth2-rs/blob/master/examples/github.rs#L82)
    /// for a way to do this.
    pub fn new(config: Config) -> Self {
        // build the oauth2 config structure
        let mut oauth_config = oauth2::Config::new(
            config.client_id,
            config.client_secret,
            config.authorize_url,
            config.token_url,
        );

        // add scope to config
        if config.scope.read_public {
            oauth_config = oauth_config.add_scope("read_public");
        }
        if config.scope.read_relationships {
            oauth_config = oauth_config.add_scope("read_relationships");
        }
        if config.scope.write_public {
            oauth_config = oauth_config.add_scope("write_public");
        }
        if config.scope.write_relationships {
            oauth_config = oauth_config.add_scope("write_relationships");
        }

        // set redirect URL
        oauth_config = oauth_config.set_redirect_url(config.redirect_url);

        // return our `TokenBuilder`
        TokenBuilder { config: oauth_config }
    }

    /// Exchange for an access token which can then be used to create a `Client`.
    pub fn exchange_code(&self, code: &str) -> Result<oauth2::Token, PinterestError> {
        let token = self.config.exchange_code(code)?;
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_authentication_url() {
        let config = Config { 
            client_id: "myclientid", 
            client_secret: "myclientsecret",  
            authorize_url: "https://example.com/authorize",
            token_url: "https://example.com/token",
            redirect_url: "https://mysite.com:8000",
            scope: Scope { read_public: true, read_relationships: true, ..Scope::default() }
        };
        assert_eq!(TokenBuilder::new(config).config.authorize_url().as_str(), "https://example.com/authorize?client_id=myclientid&scope=read_public+read_relationships&response_type=code&redirect_uri=https%3A%2F%2Fmysite.com%3A8000");
    }
}
