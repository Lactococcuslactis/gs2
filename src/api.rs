use std::fmt;

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Award {
    pub(crate) name: String,
    #[serde(rename = "cnt")]
    pub(crate) count: usize,
    pub(crate) icon: String,
}

impl fmt::Display for Award {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}x{}", self.name, self.count)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AwardList {
    pub(crate) awards: Vec<Award>,
    pub(crate) month: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SigninInfo {
    pub(crate) first_bind: bool,
    pub(crate) is_sign: bool,
    #[serde(with = "beijing_time")]
    pub(crate) today: NaiveDate,
    pub(crate) total_sign_day: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SigninResult {
    pub(crate) code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HoyolabGameTool {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) web_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HoyolabGame {
    pub(crate) game_id: i32,
    pub(crate) game_name: String,
    pub(crate) tools: Vec<HoyolabGameTool>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HoyolabGameList {
    pub(crate) games: Vec<HoyolabGame>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub(crate) retcode: i32,
    pub(crate) message: String,
    pub(crate) data: Option<T>,
}

#[derive(Debug, Clone)]
pub(crate) struct ApiResponseError(pub(crate) String);

impl<T> ApiResponse<T> {
    pub(crate) fn extract(self) -> Result<T, ApiResponseError> {
        if self.retcode == 0 {
            self.data
                .ok_or_else(|| ApiResponseError("malformed response".to_owned()))
        } else {
            Err(ApiResponseError(self.message))
        }
    }
}

mod beijing_time {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
