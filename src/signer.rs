use std::env;

use chrono::Datelike;
use log::debug;
use reqwest::{header, Client};

use crate::api::{ApiResponse, ApiResponseError, Award, AwardList, SigninInfo, SigninResult};

#[derive(Debug, Clone)]
pub(crate) struct Reward {
    pub(crate) name: String,
    pub(crate) count: usize,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum SignInError {
    #[error("network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("invalid cookie: {0}")]
    CookieError(#[from] header::InvalidHeaderValue),
    #[error("hoyolab error: {0}")]
    ApiError(String),
    #[error("already signed-in today")]
    AlreadySigned,
}

impl From<ApiResponseError> for SignInError {
    fn from(api_error: ApiResponseError) -> Self {
        Self::ApiError(api_error.0)
    }
}

const REFERER: &str = "https://webstatic-sea.mihoyo.com/";
fn get_page_url(action: &str, act_id: &str) -> String {
    let lang = env::var("MHY_LANG").unwrap_or_else(|_| "en-us".to_string());
    format!(
        "https://hk4e-api-os.mihoyo.com/event/sol/{}?lang={}&act_id={}",
        action, lang, act_id
    )
}

async fn send_request<T>(client: &Client, endpoint: &str, cookie: &str) -> Result<T, SignInError>
where
    T: serde::de::DeserializeOwned,
{
    Ok(client
        .get(endpoint)
        .header(header::REFERER, header::HeaderValue::from_static(REFERER))
        .header(header::COOKIE, header::HeaderValue::from_str(cookie)?)
        .send()
        .await?
        .json::<ApiResponse<T>>()
        .await?
        .extract()?)
}

pub(crate) async fn signin(
    client: &Client,
    act_id: &str,
    cookie: &str,
) -> Result<Option<Award>, SignInError> {
    debug!("received sign-in request");

    // pull the sign-in info
    let info: SigninInfo = send_request(client, &get_page_url("info", act_id), cookie).await?;
    debug!("sign-in info: {:?}", info);
    if info.is_sign {
        debug!("abort signing-in, user already signed-in");
        return Err(SignInError::AlreadySigned);
    }

    // pull the award list
    let award_list: AwardList = send_request(client, &get_page_url("home", act_id), cookie).await?;
    debug!("award list: {:?}", award_list);

    // sign in
    send_request::<SigninResult>(client, &get_page_url("sign", act_id), cookie).await?;
    debug!("signed in");

    let today = info.today.day0() as usize;
    Ok(award_list.awards.get(today).map(Clone::clone))
}
