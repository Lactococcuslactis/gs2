use std::env;

use log::{error, info, warn};
use reqwest::{header::HeaderValue, Client};

mod act_id;
mod api;
mod signer;

const ACT_ID: &str = "e202102251931481";
const USER_AGENT: &str = r#"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.190 Safari/537.36"#;

async fn run() -> anyhow::Result<()> {
    let cookie = env::var("MHY_COOKIE")?;
    let client = Client::builder()
        .gzip(true)
        .referer(false)
        .user_agent(HeaderValue::from_static(USER_AGENT))
        .build()?;

    let act_id = match act_id::fetch_act_id(&client).await {
        Ok(act_id) => act_id,
        Err(e) => {
            warn!(
                "failed to get act_id from hoyolab: {}, fallback to `{}`",
                e, ACT_ID
            );
            ACT_ID.to_string()
        }
    };
    info!("use act_id {} to sign-in", act_id);

    match signer::signin(&client, &act_id, &cookie).await? {
        Some(award) => {
            info!("signed in, award: {}", award);
        }
        None => {
            warn!("signed in, but unable to retrieve award details");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    match run().await {
        Ok(_) => (),
        Err(e) => {
            error!("sign-in failed: {}", e);
            std::process::exit(-1);
        }
    }
}
