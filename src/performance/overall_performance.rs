use std::str::FromStr;

use json::JsonValue;

use crate::score::{Score, Team};
use crate::Performance;

#[derive(Clone)]
pub struct OverallPerformance
{
    pub won_rounds:   u8,
    pub num_rounds:   u8,
    pub damage:       u32,
    pub damage_taken: u32,
    pub kills:        u8,
    pub deaths:       u8,
}

impl OverallPerformance
{
    pub fn from_json(score: &Score, json: &JsonValue) -> Self
    {
        let team = Team::from_str(json["team"].as_str().unwrap()).unwrap();
        let won_rounds = score.get_score(team);
        let lost_rounds = score.get_score(team.other());
        let num_rounds = won_rounds + lost_rounds;

        let damage = json["dmg"].as_u32().unwrap_or(0);
        let damage_taken = json["dt"].as_u32().unwrap_or(0);
        let kills = json["kills"].as_u8().unwrap_or(0);
        let deaths = json["deaths"].as_u8().unwrap_or(0);

        Self {
            won_rounds,
            num_rounds,
            damage,
            damage_taken,
            kills,
            deaths,
        }
    }
}

impl Into<Performance> for OverallPerformance
{
    fn into(self) -> Performance { Performance::Overall(self) }
}
