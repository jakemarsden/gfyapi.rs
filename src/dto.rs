use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::de::Visitor;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::util::wrapped_with_str;

// TODO: store relevant fields `url::Url`s or as `String`s?

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub avg_color: String,
    #[serde(rename = "content_urls")]
    pub content_urls: HashMap<String, ItemContent>,
    #[serde(with = "ts_seconds")]
    pub create_date: DateTime<Utc>,
    pub description: String,
    #[serde(with = "wrapped_with_str")]
    pub dislikes: u32,
    pub domain_whitelist: Vec<String>,
    pub extra_lemmas: String,
    pub frame_rate: f32,
    pub gatekeeper: u32,
    pub geo_whitelist: Vec<String>,
    pub gfy_id: String,
    pub gfy_name: String,
    #[serde(with = "wrapped_with_str")]
    pub gfy_number: u64,
    pub gfy_slug: String,
    pub has_audio: bool,
    pub has_transparency: bool,
    pub height: u32,
    pub language_categories: Vec<String>,
    pub language_text: String,
    #[serde(with = "wrapped_with_str")]
    pub likes: u32,
    pub md5: String,
    pub nsfw: Nsfw,
    pub num_frames: f32,
    pub published: Published,
    pub sitename: String,
    pub source: u32,
    pub tags: Vec<String>,
    pub title: String,
    pub user_data: BasicUser,
    pub views: u32,
    pub width: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ItemContent {
    pub height: u32,
    pub size: u64,
    pub url: String,
    pub width: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WrappedItem {
    pub gfy_item: Item,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BasicUser {
    pub followers: u32,
    pub following: u32,
    pub name: String,
    pub profile_image_url: String,
    pub url: String,
    pub username: String,
    pub verified: bool,
    pub views: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublicUser {
    #[serde(with = "ts_seconds")]
    pub create_date: DateTime<Utc>,
    pub description: String,
    pub followers: u32,
    pub following: u32,
    pub iframe_profile_image_visible: bool,
    pub name: String,
    pub profile_image_url: String,
    pub published_gfycats: u32,
    pub url: String,
    pub userid: String,
    pub username: String,
    pub verified: bool,
    pub views: u32,
}

#[derive(ToPrimitive, FromPrimitive, PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum Nsfw {
    Clean = 0,
    Adult = 1,
    PotentiallyOffensive = 3,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum Published {
    No = 0,
    Yes = 1,
}

impl From<&PublicUser> for BasicUser {
    fn from(other: &PublicUser) -> Self {
        Self {
            followers: other.followers,
            following: other.following,
            name: other.name.clone(),
            profile_image_url: other.profile_image_url.clone(),
            url: other.url.clone(),
            username: other.username.clone(),
            verified: other.verified,
            views: other.views,
        }
    }
}

impl Serialize for Nsfw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;
        let str_value = self
            .to_u8()
            .ok_or_else(|| {
                S::Error::custom(format!(
                    "Failed to serialize Nsfw enum value to a stringify-ed u8: {:?}",
                    self
                ))
            })?
            .to_string();
        serializer.serialize_str(str_value.as_str())
    }
}

impl<'de> Deserialize<'de> for Nsfw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NsfwVisitor;

        use serde::de::Error;
        impl<'de> Visitor<'de> for NsfwVisitor {
            type Value = Nsfw;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                f.write_str("a Nsfw enum value as a stringify-ed u8")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let u8_value = value.parse::<u8>().map_err(Error::custom)?;
                Self::Value::from_u8(u8_value).ok_or_else(|| {
                    Error::custom(format!(
                        "Failed to deserialize Nsfw enum value (expected a stringify-ed u8) from: {:?}",
                        u8_value
                    ))
                })
            }
        }

        deserializer.deserialize_str(NsfwVisitor)
    }
}
