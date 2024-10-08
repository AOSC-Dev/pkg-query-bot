use std::fmt::Display;

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pkg {
    name: String,
    description: String,
    version_matrix: Vec<Version>,
}

impl Display for Pkg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<b>{}</b>", self.name)?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f)?;

        for v in &self.version_matrix {
            for m in &v.meta {
                if !m.hasmeta || m.version.is_empty() {
                    continue;
                }

                writeln!(f, "<b>{}</b>: <code>{}</code>", v.repo, m.version)?;

                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Version {
    repo: String,
    meta: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    hasmeta: bool,
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    packages: Vec<SearchPackage>,
}

#[derive(Debug, Deserialize)]
pub struct SearchPackage {
    name: String,
}

impl SearchResult {
    pub fn fmt_result(&self, search: &str, pacakge_site_url: &str) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "<b>Found {} matching package(s)</b>:\n\n",
            self.packages.len()
        ));
        for (idx, pkg) in self.packages.iter().enumerate() {
            if idx > 10 {
                s.push('\n');
                s.push_str(&format!(
                    "For more results, check out <a href=\"{}/search?q={}&noredir=true\">packages.aosc.io</a>",
                    pacakge_site_url,
                    search
                ));
                break;
            }

            s.push_str(&format!(
                "<a href=\"{}/packages/{}\">{}</a>",
                pacakge_site_url, pkg.name, pkg.name
            ));
            s.push('\n');
        }

        s
    }

    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }
}

pub struct PackageSiteClient {
    pub url: String,
    client: Client,
}

impl PackageSiteClient {
    pub fn from_env() -> Self {
        Self::new(std::env::var("PACKAGE_SITE_URL").expect("PACKAGE_SITE_URL var is not set"))
    }

    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::builder()
                .user_agent("bot")
                .build()
                .expect("Failed to create client"),
        }
    }

    pub async fn get_package(&self, name: &str) -> reqwest::Result<Pkg> {
        self.client
            .get(format!("{}/packages/{}?type=json", self.url, name))
            .send()
            .await?
            .error_for_status()?
            .json::<Pkg>()
            .await
    }

    pub async fn search(&self, name: &str) -> reqwest::Result<SearchResult> {
        let resp = self
            .client
            .get(format!(
                "{}/search?q={}&type=json&noredir=true",
                self.url, name
            ))
            .send()
            .await?
            .error_for_status()?;

        resp.json::<SearchResult>().await
    }
}
