#![allow(clippy::unwrap_used)]

use std::fmt;

use camino::Utf8PathBuf;

use crate::parser::{CliLanguage, CliTranslate};

const GITHUB_ISSUE_URL_STR: &str = "https://github.com/bencherdev/bencher/issues/new";

const _MODEL: &str = "gpt-4-32k";

#[derive(Debug)]
pub struct Translate {
    pub lang: CliLanguage,
    pub input_path: Utf8PathBuf,
    pub output_path: Option<Utf8PathBuf>,
}

impl TryFrom<CliTranslate> for Translate {
    type Error = anyhow::Error;

    fn try_from(translate: CliTranslate) -> Result<Self, Self::Error> {
        let CliTranslate {
            lang,
            input_path,
            output_path,
        } = translate;
        Ok(Self {
            lang,
            input_path,
            output_path,
        })
    }
}

impl fmt::Display for CliLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CliLanguage::Arabic => "Modern Standard Arabic",
                CliLanguage::Chinese => "Simplified Chinese",
                CliLanguage::Spanish => "Spanish",
                CliLanguage::French => "French",
                CliLanguage::German => "German",
                CliLanguage::Japanese => "Japanese",
                CliLanguage::Kannada => "Kannada",
                CliLanguage::Portuguese => "Portuguese",
                CliLanguage::Russian => "Russian",
            }
        )
    }
}

impl CliLanguage {
    // Returns the ISO 639-1 language code for the language
    fn iso_code(self) -> &'static str {
        match self {
            CliLanguage::Arabic => "ar",
            CliLanguage::Chinese => "zh",
            CliLanguage::Spanish => "es",
            CliLanguage::French => "fr",
            CliLanguage::German => "de",
            CliLanguage::Japanese => "ja",
            CliLanguage::Kannada => "kn",
            CliLanguage::Portuguese => "pt",
            CliLanguage::Russian => "ru",
        }
    }
}

impl Translate {
    #[allow(clippy::unused_async)]
    pub async fn exec(&self) -> anyhow::Result<()> {
        let lang = self.lang;

        let _input_path = std::fs::read_to_string(&self.input_path)?;
        let _output_path = self.output_path.clone().unwrap_or_else(|| {
            let mut output_path = self.input_path.parent().unwrap().to_path_buf();
            output_path.push(lang.iso_code());
            output_path.set_file_name(self.input_path.file_name().unwrap());
            output_path
        });

        let _prompt = format!("You are a professional translator for software documentation. Translate the Markdown (MDX) text below from American English to {lang}. Keep in mind that the audience for the translation is software developers.");
        let issue_url = github_issue_url(lang, &self.input_path);
        let _disclosure = format!(
            r#"This document was automatically generated by OpenAI GPT-4. It might not be accurate and might contain errors. If you find any errors, please <a href="{issue_url}">open an issue on GitHub</a>."#,
        );

        // serde_json::from_str(&swagger_spec_str).map_err(Into::into)
        Ok(())
    }
}

pub fn github_issue_url(lang: CliLanguage, file_path: &Utf8PathBuf) -> url::Url {
    let mut url: url::Url = GITHUB_ISSUE_URL_STR.parse().unwrap();
    let title = format!("Issue with translation to {lang}");
    let body = format!("Issue with translation to {lang} for {file_path}.");
    let query = serde_urlencoded::to_string([
        ("title", title.as_str()),
        ("body", body.as_str()),
        ("labels", "documentation"),
    ])
    .ok();
    url.set_query(query.as_deref());
    url
}
