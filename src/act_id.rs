use log::debug;
use regex::Regex;
use reqwest::{header, Client};

const HOYOLAB_BASE: &str = "https://www.hoyolab.com";
const HOYOLAB: &str = "https://www.hoyolab.com/genshin/";

#[derive(Debug, thiserror::Error)]
pub(crate) enum ActIdError {
    #[error("network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("cannot find act_id from hoyolab")]
    NotFound,
}

pub(crate) async fn fetch_act_id(client: &Client) -> Result<String, ActIdError> {
    let forum_body = client.get(HOYOLAB).send().await?.text().await?;
    debug!("hoyolab homepage loaded");
    let js_regex = Regex::new(r#"src="([^"]*\.js)""#).unwrap();
    let sign_in_page_regex = Regex::new(r#""https://webstatic-sea\.mihoyo\.com/ys/event/signin-sea/index\.html\?act_id="\.concat\(([[:word:]]+)\)"#).unwrap();
    for file in js_regex
        .captures_iter(&forum_body)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str()))
    {
        let url = if file.starts_with("/") {
            format!("{}{}", HOYOLAB_BASE, file)
        } else if file.starts_with("https:") || file.starts_with("http:") {
            file.to_owned()
        } else {
            format!("{}{}", HOYOLAB, file)
        };

        debug!("loading file `{}`", url);
        let js_content = client
            .get(&url)
            .header(header::REFERER, header::HeaderValue::from_static(HOYOLAB))
            .send()
            .await?
            .text()
            .await?;

        debug!("finding act_id in file `{}`", url);
        if let Some(caps) = sign_in_page_regex.captures(&js_content) {
            let var_name = caps.get(1).unwrap().as_str();
            let var_regex = Regex::new(&format!(
                r#"[[:^word:]]\s*{}\s*=\s*"([[:word:]]+)""#,
                var_name
            ))
            .unwrap();

            if let Some(caps) = var_regex.captures(&js_content) {
                let act_id = caps.get(1).unwrap().as_str().to_string();
                debug!("act_id `{}` found in file `{}`", act_id, url);
                return Ok(act_id);
            }
        }
    }

    Err(ActIdError::NotFound)
}
