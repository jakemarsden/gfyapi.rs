use reqwest::header::*;
use reqwest::Client;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::dto::*;
use crate::error::Error;
use crate::result::Result;

pub struct Gfycat {
    http_client: Client,
}

impl Gfycat {
    pub const DOMAIN: &'static str = "api.gfycat.com";
    pub const VERSION: u32 = 1;

    pub fn new() -> Result<Self> {
        let http_client = Client::builder().build().map_err(Error::Init)?;
        Ok(Self::from(http_client))
    }
}

impl Default for Gfycat {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl From<Client> for Gfycat {
    fn from(http_client: Client) -> Self {
        Self { http_client }
    }
}

impl Gfycat {
    pub async fn get_item(&self, gfy_id: &str) -> Result<Item> {
        let req_url = format!(
            "https://{domain}/v{version}/gfycats/{gfyId}",
            domain = Self::DOMAIN,
            version = Self::VERSION,
            gfyId = gfy_id
        );
        let req = self.http_client.get(req_url.as_str());
        self.send_json_request::<WrappedItem>(req)
            .await
            .map(|wrapped| wrapped.gfy_item)
    }

    pub async fn get_user(&self, userid: &str) -> Result<PublicUser> {
        let req_url = format!(
            "https://{domain}/v{version}/users/{userid}",
            domain = Self::DOMAIN,
            version = Self::VERSION,
            userid = userid
        );
        let req = self.http_client.get(req_url.as_str());
        self.send_json_request::<_>(req).await
    }

    async fn send_json_request<T>(&self, req: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let res = req
            .header(ACCEPT, "application/json")
            .header(ACCEPT_CHARSET, "utf-8")
            .send()
            .await
            .map_err(Error::Network)?;

        let res_status = res.status();
        if !res_status.is_success() {
            return Err(Error::Status(res_status));
        }

        let res_body = res.json::<T>().await.map_err(Error::Parse)?;
        Ok(res_body)
    }
}
