use std::str::FromStr;

use json::JsonValue;

use crate::score::{Score, Team};

#[derive(Clone)]
pub struct GenericPerformance
{
    pub won_rounds:   u8,
    pub num_rounds:   u8,
    pub damage_taken: u32,
}

impl GenericPerformance
{
    pub fn from_json(score: &Score, json: &JsonValue) -> Self
    {
        let team = Team::from_str(json["team"].as_str().unwrap()).unwrap();
        let won_rounds = score.get_score(team);
        let lost_rounds = score.get_score(team.other());
        let num_rounds = won_rounds + lost_rounds;

        let damage_taken = json["dt"].as_u32().unwrap();

        Self {
            won_rounds,
            num_rounds,
            damage_taken,
        }
    }
}
