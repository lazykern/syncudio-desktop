use dropbox_sdk::{
    default_client::{NoauthDefaultClient, UserAuthDefaultClient},
    oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode},
};
use log::{error, info, warn};
use tokio::sync::Mutex;

const DROPBOX_CLIENT_ID: &str = "jgibk23zkucv2ec";

pub struct Dropbox {
    pkce_code: Mutex<Option<PkceCode>>,
    authorization: Mutex<Option<Authorization>>,
    client: Mutex<Option<UserAuthDefaultClient>>,
}

impl Dropbox {
    pub fn new() -> Self {
        Self {
            pkce_code: Mutex::new(None),
            authorization: Mutex::new(None),
            client: Mutex::new(None),
        }
    }

    pub async fn is_authorized(&self) -> bool {
        self.authorization.lock().await.is_some() && self.client.lock().await.is_some()
    }

    pub async fn unauthorize(&self) {
        let mut auth_guard = self.authorization.lock().await;
        *auth_guard = None;
        let mut client_guard = self.client.lock().await;
        *client_guard = None;
    }

    pub async fn complete_authorization(
        &self,
        auth_code: &str,
    ) -> Result<(String, Option<String>), String> {
        info!("Completing Dropbox authorization");

        let pkce_code = self.pkce_code.lock().await.take().ok_or_else(|| {
            error!("No PKCE code found in state");
            "No PKCE code found. Please start the authorization process again.".to_string()
        })?;

        let flow_type: Oauth2Type = Oauth2Type::PKCE(pkce_code);

        let mut auth = Authorization::from_auth_code(
            DROPBOX_CLIENT_ID.to_string(),
            flow_type,
            auth_code.to_string(),
            None,
        );

        info!("Obtaining access token...");
        let client = NoauthDefaultClient::default();
        let access_token = auth.obtain_access_token(client).map_err(|e| {
            error!("Failed to obtain access token: {}", e);
            format!("Failed to obtain access token: {}", e)
        })?;
        info!("Successfully obtained access token: {}", access_token);

        let mut auth_guard = self.authorization.lock().await;
        *auth_guard = Some(auth.clone());

        let client = UserAuthDefaultClient::new(auth.clone());
        self.client.lock().await.replace(client);

        match auth.save() {
            Some(auth_data) => {
                info!("Authorization completed and saved successfully");
                info!("Auth data: {}", auth_data);
                Ok((access_token, Some(auth_data)))
            }
            None => {
                warn!("Authorization completed but failed to save auth data");
                Ok((access_token, None))
            }
        }
    }

    pub async fn auth_info(&self) -> Result<String, String> {
        Ok("".to_string())
    }

    pub async fn start_authorization(&self, pkce_code: PkceCode) -> Result<String, String> {
        info!("Generating Dropbox authorization URL");
        let mut pkce_code_guard = self.pkce_code.lock().await;
        *pkce_code_guard = Some(pkce_code.clone());
        let flow_type = Oauth2Type::PKCE(pkce_code.clone());
        let auth_url = AuthorizeUrlBuilder::new(DROPBOX_CLIENT_ID, &flow_type).build();

        info!("Generated authorization URL successfully");
        Ok(auth_url.to_string())
    }
}
