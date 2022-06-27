use rocket::form::{self, FromFormField, ValueField};
use rocket::request::{self, Request, FromRequest, FromParam};
use ms_converter::{ms_into_time, get_max_possible_duration};
use rocket::serde::{Deserialize, Serialize, Serializer, ser};
use rocket::serde::de::Visitor;
use std::error::Error;
use std::num::TryFromIntError;
use std::ops::Deref;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Ms(Duration);

impl Ms {
    pub fn to_string(&self) -> Option<String> {
        Self::dur_to_string(self)
    }

    pub fn dur_to_string(dur: &Duration) -> Option<String> {
        dur.as_millis()
            .try_into().ok()
            .and_then(|v| get_max_possible_duration(v).ok())
    }
}

impl Deref for Ms {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Duration> for Ms {
    fn from(dur: Duration) -> Self {
        Ms(dur)
    }
}

impl Into<Duration> for Ms {
    fn into(self) -> Duration {
        self.0
    }
}

impl TryFrom<&str> for Ms {
    type Error = ms_converter::Error;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ms_into_time(value)
            .map(Duration::into)
    }
}

struct MsVisitor;

impl<'de> Visitor<'de> for MsVisitor {
    type Value = Ms;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("duration must be a valid ms string. See https://docs.rs/ms-converter/1.4.0/ms_converter/index.html#supported-time-strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: rocket::serde::de::Error, 
    {
        v.try_into()
            .map_err(|err: ms_converter::Error| rocket::serde::de::Error::custom(err.to_string()))
    }
}

impl<'de> Deserialize<'de> for Ms {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: rocket::serde::Deserializer<'de> 
    {
        deserializer.deserialize_str(MsVisitor)
    }
}

// This is big shit.
impl Serialize for Ms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: rocket::serde::Serializer
    {
        use ser::Error;

        self.to_string()
            .ok_or_else(|| Error::custom("Failed to convert Ms to String"))
            .and_then(|s: String| serializer.collect_str(&s))
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for Ms {

    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        ms_into_time(field.value)
            .map(|dur| dur.into())
            .map_err(|err| form::Error::validation(err.to_string()))
            .map_err(|err| err.into())
    }
}