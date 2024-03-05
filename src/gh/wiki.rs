use reqwest::Client;
use select::{document::Document, predicate::Name};

use super::GetEdit;

#[derive(Debug)]
pub(crate) struct WikiArticle {
    pub title: String,
    pub uri: String,
}

impl GetEdit for WikiArticle {
    fn get_edit(&self) -> String {
        let title = self.title.to_owned();
        let uri = self.uri.to_owned();
        format!("[{title}](https://github.com{uri})")
    }
}

pub async fn find_wiki_articles(
    owner: &str,
    repo: &str,
) -> Result<Vec<WikiArticle>, reqwest::Error> {
    let client = Client::new();
    //TODO: find a way to support private wikis?
    let url = format!("https://github.com/{owner}/{repo}/wiki");
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    let mut ret: Vec<WikiArticle> = Document::from(body.as_str())
        .find(Name("a"))
        .filter(|a| a.attr("href").is_some())
        .filter(|a| {
            let href = a.attr("href").unwrap();
            // GitHub renders wiki as relative links
            // /$owner/$repo/$page
            // We do not want wiki/_new et.al.
            href.starts_with('/') && href.contains("wiki/") && !href.contains("/_")
        })
        .map(|link| WikiArticle {
            title: link.text(),
            uri: link.attr("href").unwrap().to_string(),
        })
        .collect();
    ret.push(WikiArticle {
        title: "Home".into(),
        uri: format!("/{owner}/{repo}/wiki"),
    });

    Ok(ret)
}
