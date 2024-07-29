use anyhow::Result;
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Builder, PartialEq)]
struct User {
    name: String,
    age: u8,
    skills: Vec<String>,
    #[serde(rename = "workState")]
    state: WorkState,

    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,

    // De/Serialize using Display and FromStr implementation
    #[serde_as(as = "Option<DisplayFromStr>")]
    blog: Option<Uri>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
enum WorkState {
    Working(String),
    OnLeave(DateTime<Utc>),
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = BASE64_URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = BASE64_URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("Alice".to_string())
        .age(30)
        .skills(vec!["Rust".to_string(), "Python".to_string()])
        .state(WorkState::OnLeave(Utc::now()))
        .data(vec![1, 2, 3, 4])
        .blog("https://example.com".parse::<Uri>()?.into())
        .build()?;
    let req_json = serde_json::to_string(&user)?;
    println!("{}", req_json);
    let user2: User = serde_json::from_str(&req_json)?;
    println!("{:?}", user2);
    assert_eq!(user, user2);
    Ok(())
}
