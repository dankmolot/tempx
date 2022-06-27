use rocket::serde::{self, Deserialize, Serialize};
use crate::ms::Ms;
use std::time::Duration;
use rocket::figment::{self, Provider, Error, Figment, Profile};
use rocket::request::{FromRequest, Request, Outcome};
use rocket::{Rocket, Orbit};
use rocket::http::Status;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AppConfig {
    pub default_expire: Ms,
    pub max_expire: Ms,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig { 
            default_expire: Duration::from_secs(60).into(), // Default expire is 1 minute
            max_expire: Duration::from_secs(60 * 10).into(), // Maximux expire is 10 minutes
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

// impl AppConfig {
//     fn from<T: Provider>(provider: T) -> Result<AppConfig, Error> {
//         Figment::from(provider).extract()
//     }

//     fn figment() -> Figment {
//         use figment::providers::Env;

//         Figment::from(AppConfig::default()).merge(Env::prefixed("APP_"))
//     }
// }

use figment::value::{Map, Dict};

impl Provider for AppConfig {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("TempX Conifg")
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