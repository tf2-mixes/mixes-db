use std::collections::HashMap;
use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use json::JsonValue;

use super::{QueryResult, LOGS_TF_API_BASE};
use crate::score::Score;
use crate::{Performance, SteamID};

pub struct LogMetadata
{
    pub id:          u32,
    pub date_time:   DateTime<Utc>,
    pub map:         String,
    pub num_players: u8,
}

pub struct Log
{
    meta:          LogMetadata,
    performances:  HashMap<SteamID, Vec<Performance>>,
    duration_secs: u32,
}

impl LogMetadata
{
    pub fn from_json(json: &JsonValue) -> Self
    {
        Self {
            id:          json["id"].as_u32().unwrap(),
            date_time:   DateTime::from_utc(
                NaiveDateTime::from_timestamp(json["date"].as_i64().unwrap(), 0),
                Utc,
            ),
            map:         json["map"].as_str().unwrap().to_owned(),
            num_players: json["players"].as_u8().unwrap(),
        }
    }
}

impl Log
{
    /// Download the log with the given id from logs.tf and turn it into a
    /// format that can be processed by a rating system easily.
    pub fn download(id: u32) -> QueryResult<Self>
    {
        let log = reqwest::blocking::get(format!("{}/{}", LOGS_TF_API_BASE, id))?
            .text()
            .expect("Unable to read response body");

        let json = json::parse(&log)?;
        super::check_json_success(&json)?;

        Ok(Self::from_json(id, &json))
    }

    /// Parse the json information as found on logs.tf into a format easily
    /// digestible by the rating system.
    // XXX: Check presumed logs.tf json for any format deviances
    pub fn from_json(id: u32, json: &JsonValue) -> Self
    {
        let info = &json["info"];
        let duration_secs = info["total_length"]
            .as_u32()
            .expect("Duration is not an unsigned int");
        let map = info["map"]
            .as_str()
            .expect("Unable to read map of log")
            .to_owned();
        let timestamp = info["date"]
            .as_u32()
            .expect("Unable to read date as Unix timestamp") as i64;
        let date_time = DateTime::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
        let num_players = json["names"].members().len() as u8;

        let meta = LogMetadata {
            id,
            date_time,
            map,
            num_players,
        };

        let score = Score::from_json(json);

        let mut performances = HashMap::new();
        for (player_id, stats) in json["players"].entries() {
            let player_id =
                SteamID::from_str(player_id).expect("Player id is not a valid steam id");

            let player_performances = Performance::extract_all_from_json(&score, stats);
            performances.insert(player_id, player_performances);
        }

        Self {
            meta,
            performances,
            duration_secs,
        }
    }

    pub fn meta(&self) -> &LogMetadata { &self.meta }
    pub fn duration_secs(&self) -> u32 { self.duration_secs }
    pub fn performances(&self) -> &HashMap<SteamID, Vec<Performance>> { &self.performances }
}
