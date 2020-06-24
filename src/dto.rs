use chrono::{DateTime, Utc};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::de::Visitor;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub avg_color: String,
    #[serde(rename = "content_urls")]
    pub content_urls: HashMap<String, ItemContent>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub create_date: DateTime<Utc>,
    pub description: String,
    pub frame_rate: f32,
    pub gfy_id: String,
    pub gfy_name: String,
    #[serde(with = "crate::serialize::wrapped_with_str")]
    pub gfy_number: u64,
    pub gfy_slug: String,
    pub has_audio: bool,
    pub has_transparency: bool,
    pub height: u32,
    pub language_categories: Vec<String>,
    pub language_text: String,
    pub md5: String,
    pub nsfw: Nsfw,
    pub num_frames: f32,
    pub published: Published,
    pub sitename: String,
    pub tags: Vec<String>,
    pub title: String,
    pub user_data: User,
    pub width: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemContent {
    pub height: u32,
    pub size: u64,
    pub url: String, // TODO: store as a url::Url ?
    pub width: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemWrapper {
    pub gfy_item: Item,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub name: String,
    pub profile_image_url: String, // TODO: store as a url::Url ?
    pub url: String,               // TODO: store as a url::Url ?
    pub username: String,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error_message: String,
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

impl Serialize for Nsfw {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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
