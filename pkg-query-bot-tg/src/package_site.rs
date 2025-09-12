use std::fmt::Display;

use pkgsite_lib::{info::Info, search::Search};

pub struct Pkg<'a> {
    inner: &'a Info,
}

impl<'a> From<&'a Info> for Pkg<'a> {
    fn from(inner: &'a Info) -> Self {
        Self { inner }
    }
}

impl Display for Pkg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "<b>{}</b>", self.inner.name)?;
        writeln!(f)?;
        writeln!(f, "{}", self.inner.description)?;
        writeln!(f)?;

        for v in &self.inner.version_matrix {
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

pub struct SearchResult<'a> {
    inner: &'a Search,
}

impl<'a> From<&'a Search> for SearchResult<'a> {
    fn from(inner: &'a Search) -> Self {
        Self { inner }
    }
}

impl SearchResult<'_> {
    pub fn fmt_result(&self, search: &str, pacakge_site_url: &str) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "<b>Found {} matching package(s)</b>:\n\n",
            self.inner.packages.len()
        ));
        for (idx, pkg) in self.inner.packages.iter().enumerate() {
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
        self.inner.packages.is_empty()
    }
}
