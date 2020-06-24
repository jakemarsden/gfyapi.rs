use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug)]
pub struct Client {
    http_client: reqwest::Client,
    api_domain: String,
    api_version: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    avg_color: String,
    #[serde(rename = "content_urls")]
    content_urls: HashMap<String, ItemContent>,
    #[serde(with = "ts_seconds")]
    create_date: DateTime<Utc>,
    description: String,
    frame_rate: f32,
    gfy_id: String,
    gfy_name: String,
    #[serde(with = "wrapped_with_str")]
    gfy_number: u64,
    gfy_slug: String,
    has_audio: bool,
    has_transparency: bool,
    height: u32,
    language_categories: Vec<String>,
    language_text: String,
    md5: String,
    nsfw: Nsfw,
    num_frames: f32,
    published: Published,
    sitename: String,
    tags: Vec<String>,
    title: String,
    user_data: User,
    width: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemContent {
    height: u32,
    size: u64,
    url: String, // TODO: store as a url::Url ?
    width: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    name: String,
    profile_image_url: String, // TODO: store as a url::Url ?
    url: String,               // TODO: store as a url::Url ?
    username: String,
    verified: bool,
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
        struct NsfwVisitor;

        impl<'de> Visitor<'de> for NsfwVisitor {
            type Value = Nsfw;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a stringify-ed integer")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use serde::de::Error;

                let u8_value = value.parse::<u8>().map_err(Error::custom)?;
                Self::Value::from_u8(u8_value)
                    // TODO: nice error message
                    .ok_or_else(|| Error::custom("TODO: nice error message"))
            }
        }

        deserializer.deserialize_str(NsfwVisitor)
    }
}

mod wrapped_with_str {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt::Display;
    use std::result::Result;
    use std::str::FromStr;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ToString,
        S: Serializer,
    {
        let str = value.to_string();
        String::serialize(&str, serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        T::from_str(&str).map_err(Error::custom)
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
    use super::{Client, Error, Item, ItemContent, Nsfw, Published, User};
    use chrono::{TimeZone, Utc};
    use maplit::hashmap;

    #[tokio::test]
    async fn my_test() -> std::result::Result<(), Error> {
        let to_string_vec = |vec: &[&str]| {
            vec.iter()
                .map(|str| String::from(*str))
                .collect::<Vec<String>>()
        };
        let expected_content_urls = hashmap! {
            String::from("100pxGif") => ItemContent {
                height: 217,
                size: 870_224,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-max-1mb.gif"),
                width: 340,
            },
            String::from("largeGif") => ItemContent {
                height: 383,
                size: 3_675_771,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-size_restricted.gif"),
                width: 600,
            },
            String::from("max1mbGif") => ItemContent {
                height: 217,
                size: 870_224,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-max-1mb.gif"),
                width: 340,
            },
            String::from("max2mbGif") => ItemContent {
                height: 306,
                size: 1_698_034,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-small.gif"),
                width: 480,
            },
            String::from("max5mbGif") => ItemContent {
                height: 383,
                size: 3_675_771,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-size_restricted.gif"),
                width: 600,
            },
            String::from("mobile") => ItemContent {
                height: 408,
                size: 64_812,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-mobile.mp4"),
                width: 640,
            },
            String::from("mobilePoster") => ItemContent {
                height: 408,
                size: 17_164,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri-mobile.jpg"),
                width: 640,
            },
            String::from("mp4") => ItemContent {
                height: 600,
                size: 652_935,
                url: String::from("https://giant.gfycat.com/EnormousDescriptiveIndri.mp4"),
                width: 940,
            },
            String::from("webm") => ItemContent {
                height: 600,
                size: 96_156,
                url: String::from("https://giant.gfycat.com/EnormousDescriptiveIndri.webm"),
                width: 940,
            },
            String::from("webp") => ItemContent {
                height: 0,
                size: 131_840,
                url: String::from("https://thumbs.gfycat.com/EnormousDescriptiveIndri.webp"),
                width: 0,
            },
        };
        let expected_item = Item {
            avg_color: String::from("#B99A65"),
            content_urls: expected_content_urls.clone(),
            create_date: Utc.timestamp(1_592_739_359, 0),
            description: String::from(""),
            frame_rate: 24.675325,
            gfy_id: String::from("enormousdescriptiveindri"),
            gfy_name: String::from("EnormousDescriptiveIndri"),
            gfy_number: 504_490_887,
            gfy_slug: String::from("terrible-haircut-mistake-devine-mirror-color-wrong"),
            has_audio: true,
            has_transparency: false,
            height: 600,
            language_categories: to_string_vec(&["trending"]),
            language_text: String::from(""),
            md5: String::from("86b4a541321b0985c4f30e2997a6594f"),
            num_frames: 38.0,
            nsfw: Nsfw::Clean,
            published: Published::Yes,
            sitename: String::from("gfycat"),
            tags: to_string_vec(&[
                "adam", "bad", "color", "devine", "dye", "first", "god", "hair", "haircut", "met",
                "mirror", "mistake", "my", "oh", "omg", "oops", "terrible", "we", "when", "wrong",
            ]),
            title: String::from("OMG"),
            user_data: User {
                name: String::from("Deus GIF Machina"),
                profile_image_url: String::from("https://profiles.gfycat.com/f78474b28ad6ace82bc702e4fdcdd12bb2a5416ecf62e73967811f63b3fcadc5.png"),
                url: String::from("https://gfycat.com/@gifmachina"),
                username: String::from("gifmachina"),
                verified: false,
            },
            width: 940,
        };

        let item = Client::default()?
            .get_item("enormousdescriptiveindri")
            .await?;
        assert_eq!(item, expected_item);

        Ok(())
    }
}
