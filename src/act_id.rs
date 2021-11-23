use log::debug;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client, Url,
};

use crate::api::{ApiResponse, HoyolabGameList};

const HOYOLAB_TOOLS: &str = "https://bbs-api-os.mihoyo.com/community/gametool/wapi/main";
const GENSHIN_ID: i32 = 2;
const SIGNIN_ID: i32 = 19;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ActIdError {
    #[error("network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("cannot find act_id from hoyolab")]
    NotFound,
}

pub(crate) async fn fetch_act_id(client: &Client) -> Result<String, ActIdError> {
    let games = client
        .get(HOYOLAB_TOOLS)
        .header(
            HeaderName::from_static("x-rpc-client_type"),
            HeaderValue::from_static("4"),
        )
        .header(
            HeaderName::from_static("x-rpc-language"),
            HeaderValue::from_static("zh-cn"),
        )
        .send()
        .await?
        .json::<ApiResponse<HoyolabGameList>>()
        .await?
        .extract()
        .map_err(|_| ActIdError::NotFound)?;
    debug!("hoyolab game list loaded");
    let tool = games
        .games
        .into_iter()
        .find(|game| game.game_id == GENSHIN_ID)
        .and_then(|game| game.tools.into_iter().find(|tool| tool.id == SIGNIN_ID))
        .ok_or(ActIdError::NotFound)?;
    debug!("genshin singin tool located");
    let web_path = Url::parse(&tool.web_path).map_err(|_| ActIdError::NotFound)?;
    let act_id = web_path
        .query_pairs()
        .find(|(name, _)| name == "act_id")
        .map(|pair| pair.1)
        .ok_or(ActIdError::NotFound)?;

    Ok(act_id.into_owned())
}
