use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::PartialEq;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug)]
pub struct Client {
    http_client: reqwest::Client,
    api_domain: String,
    api_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    gfy_id: String,
    gfy_name: String,
    language_categories: Vec<String>,
    nsfw: Nsfw,
    published: Published,
    tags: Vec<String>,
    title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ItemWrapper {
    gfy_item: Item,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Published {
    No = 0,
    Yes = 1,
}

#[derive(ToPrimitive, FromPrimitive, PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
pub enum Nsfw {
    Clean = 0,
    Adult = 1,
    PotentiallyOffensive = 3,
}

struct NsfwVisitor;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Initialization(reqwest::Error),
    Connect(reqwest::Error),
    HttpNotOk(reqwest::StatusCode),
    BadResponse(reqwest::Error),
}

impl Client {
    pub fn default() -> Result<Self> {
        // TODO: Which is better?:
        //   - return `Result<Self>`, as we do now
        //   - return `Self` and `impl Default for Client`, and trash `Error::Initialization`
        Ok(Self::from(
            Self::default_http_client().map_err(Error::Initialization)?,
            Self::default_api_domain(),
            Self::default_api_version(),
        ))
    }

    pub fn from(http_client: reqwest::Client, api_domain: &str, api_version: u32) -> Self {
        Self {
            http_client,
            api_domain: String::from(api_domain),
            api_version,
        }
    }

    pub fn default_http_client() -> reqwest::Result<reqwest::Client> {
        reqwest::Client::builder().build()
    }

    pub fn default_api_domain() -> &'static str {
        "api.gfycat.com"
    }

    pub fn default_api_version() -> u32 {
        1
    }
}

impl Client {
    pub fn api_domain(&self) -> &str {
        self.api_domain.as_str()
    }

    pub fn api_version(&self) -> u32 {
        self.api_version
    }

    pub async fn get_item(&self, gfy_id: &str) -> Result<Item> {
        let url = format!(
            "https://{domain}/v{version}/gfycats/{gfyid}",
            domain = self.api_domain,
            version = self.api_version,
            gfyid = gfy_id
        );
        let res = self
            .http_client
            .get(url.as_str())
            .send()
            .await
            .map_err(Error::Connect)?;
        let res_status = res.status();
        if !res_status.is_success() {
            return Err(Error::HttpNotOk(res_status));
        }
        res.json::<ItemWrapper>()
            .await
            .map(|res_obj| res_obj.gfy_item)
            .map_err(Error::BadResponse)
    }
}

impl Serialize for Nsfw {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: `unimplemented!()` is not error handling!
        //   e.g. make this work: `.ok_or(S::Error::custom(format!("{}", self)))?.to_string()`
        let str_value = self.to_u8().ok_or_else(|| unimplemented!())?.to_string();
        serializer.serialize_str(str_value.as_str())
    }
}

impl<'de> Deserialize<'de> for Nsfw {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Nsfw, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(NsfwVisitor)
    }
}

impl<'de> Visitor<'de> for NsfwVisitor {
    type Value = Nsfw;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a stringify-ed integer")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // TODO: `unimplemented!()` is not error handling!
        //   e.g.: `E::custom(format!("{}", value))`
        let u8_value = value.parse::<u8>().map_err(|_err| unimplemented!())?;
        // TODO: `unimplemented!()` is not error handling!
        //   e.g.: `E::custom(format!("{}", value))`
        Self::Value::from_u8(u8_value).ok_or_else(|| unimplemented!())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Error::Initialization(err) => err,
            Error::Connect(err) => err,
            Error::HttpNotOk(_) => return None,
            Error::BadResponse(err) => err,
        })
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Client, Error, Nsfw, Published};

    #[tokio::test]
    async fn my_test() -> std::result::Result<(), Error> {
        let item = Client::default()?
            .get_item("enormousdescriptiveindri")
            .await?;

        let expected_tags = [
            "adam", "bad", "color", "devine", "dye", "first", "god", "hair", "haircut", "met",
            "mirror", "mistake", "my", "oh", "omg", "oops", "terrible", "we", "when", "wrong",
        ];
        assert_eq!(item.gfy_id, "enormousdescriptiveindri");
        assert_eq!(item.gfy_name, "EnormousDescriptiveIndri");
        assert_eq!(item.language_categories, ["trending"]);
        assert_eq!(item.nsfw, Nsfw::Clean);
        assert_eq!(item.published, Published::Yes);
        assert_eq!(item.tags, expected_tags);
        assert_eq!(item.title, "OMG");
        Ok(())
    }
}
