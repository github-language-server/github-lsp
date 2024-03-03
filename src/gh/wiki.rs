use reqwest::Client;
use select::{document::Document, predicate::Name};

pub(crate) struct WikiArticle {
    title: String,
    uri: String,
}
pub(crate) async fn find_wiki_articles(
    client: &Client,
    url: &str,
) -> Result<Vec<WikiArticle>, reqwest::Error> {
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    let ret: Vec<WikiArticle> = Document::from(body.as_str())
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

    Ok(ret)
}
