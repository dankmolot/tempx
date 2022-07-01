use rocket::serde::{Deserialize, Serialize};
use crate::ms::Ms;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use rocket::figment::{self, Provider, Error, Profile};
use rocket::data::{Limits, ToByteUnit};
use rocket::request::{FromRequest, Request, Outcome};
use rocket::http::Status;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AppConfig {
    pub default_expire: Ms,
    pub max_expire: Ms,
    pub port: u16,
    pub limits: Limits,
    pub address: IpAddr
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_expire: Duration::from_secs(60).into(), // Default expire is 1 minute
            max_expire: Duration::from_secs(60 * 10).into(), // Maximum expire is 10 minutes

            // Here we rewrite default settings, it only works properly only here.
            port: 3000,
            limits: Limits::new().limit("file", 10.mebibytes()),
            address: IpAddr::from_str("0.0.0.0").unwrap()
        }
    }
}

impl AppConfig {
    pub fn get_safe_duration(&self, dur: Option<Ms>) -> Duration {
        let mut ms = dur.unwrap_or_else(|| self.default_expire);

        if *ms > *self.max_expire {
            ms = self.max_expire;           
        }

        *ms
    }
}

use figment::value::{Map, Dict};

// Returns default settings for Figment.
impl Provider for AppConfig {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("TempX Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(AppConfig::default()).data()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AppConfig {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().figment().extract::<AppConfig>() {
            Ok(config) => Outcome::Success(config),
            Err(_) => Outcome::Failure((Status::InternalServerError, ()))
        }
    }
}