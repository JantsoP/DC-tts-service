use std::marker::PhantomData;

use anyhow::Result;
use serde::ser::SerializeStruct;
use small_fixed_array::FixedString;

fn get_deepl_api_url() -> String {
    let tier = std::env::var("DEEPL_API_TIER")
        .unwrap_or_else(|_| "Free".to_string())
        .to_lowercase();
    
    match tier.as_str() {
        "pro" => "https://api.deepl.com".to_string(),
        _ => "https://api-free.deepl.com".to_string(),
    }
}

fn deserialize_single_seq<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    struct SingleVisitor<T>(PhantomData<T>);

    impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for SingleVisitor<T> {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            seq.next_element()
        }
    }

    deserializer.deserialize_seq(SingleVisitor(PhantomData))
}

#[derive(serde::Serialize)]
struct TranslateRequest<'a> {
    text: &'a str,
    target_lang: &'a str,
    preserve_formatting: u8,
}

#[derive(serde::Deserialize)]
struct Translation {
    pub text: FixedString,
    pub detected_source_language: FixedString<u8>,
}

#[derive(serde::Deserialize)]
struct TranslateResponse {
    #[serde(deserialize_with = "deserialize_single_seq")]
    pub translations: Option<Translation>,
}

fn auth_header(token: &str) -> String {
    format!("DeepL-Auth-Key {token}")
}

pub async fn run(
    reqwest: &reqwest::Client,
    token: &str,
    content: &str,
    target_lang: &str,
) -> Result<Option<FixedString>> {
    let request = TranslateRequest {
        target_lang,
        text: content,
        preserve_formatting: 1,
    };

    let api_url = format!("{}/v2/translate", get_deepl_api_url());
    
    let response: TranslateResponse = reqwest
        .get(&api_url)
        .query(&request)
        .header("Authorization", auth_header(token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(translation) = response.translations
        && translation.detected_source_language != target_lang
    {
        return Ok(Some(translation.text));
    }

    Ok(None)
}

#[derive(serde::Deserialize)]
struct Voice {
    pub name: FixedString,
    pub language: FixedString,
}

struct VoiceRequest;
impl serde::Serialize for VoiceRequest {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serializer = serializer.serialize_struct("DeeplVoiceRequest", 1)?;
        serializer.serialize_field("type", "target")?;
        serializer.end()
    }
}

pub async fn get_languages(
    reqwest: &reqwest::Client,
    token: &str,
) -> Result<Vec<(FixedString, FixedString)>> {
    let api_url = format!("{}/v2/languages", get_deepl_api_url());
    
    let languages: Vec<Voice> = reqwest
        .get(&api_url)
        .query(&VoiceRequest)
        .header("Authorization", auth_header(token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let language_map = languages
        .into_iter()
        .map(|v| (v.language, v.name))
        .collect();

    Ok(language_map)
}
