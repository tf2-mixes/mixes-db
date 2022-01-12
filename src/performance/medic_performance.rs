use std::str::FromStr;

use json::JsonValue;

use super::Performance;
use crate::Class;

pub struct MedicPerformance
{
    pub healing: u32,
    pub average_uber_length_secs: f32,
    pub num_ubers: u8,
    pub num_drops: u8,
    pub deaths: u8,
    pub time_played_secs: u32,
}

impl MedicPerformance
{
    pub fn extract_from_json(json: &JsonValue) -> Option<Self>
    {
        let class_stats = json["class_stats"].members().find(|class_stats| {
            Class::from_str(class_stats["type"].as_str().unwrap()).unwrap() == Class::Medic
        });

        if !json.has_key("medicstats") || class_stats.is_none() {
            return None;
        }
        let class_stats = class_stats.unwrap();

        Some(Self {
            healing: json["heal"].as_u32().unwrap_or(0),
            average_uber_length_secs: json["medicstats"]["avg_uber_length"]
                .as_f32()
                .unwrap_or(0.0),
            num_ubers: json["ubers"].as_u8().unwrap_or(0),
            num_drops: json["drops"].as_u8().unwrap_or(0),
            deaths: class_stats["deaths"].as_u8().unwrap_or(0),
            time_played_secs: class_stats["total_time"].as_u32().unwrap_or(0),
        })
    }
}

impl Into<Performance> for MedicPerformance
{
    fn into(self) -> Performance { Performance::Med(self) }
}

#[cfg(test)]
mod tests
{
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn extract_from_json()
    {
        let mut json = String::new();
        File::open("test_data/log_3094861.json")
            .expect("Unable to open test file")
            .read_to_string(&mut json)
            .expect("Unable to read file to string");
        let json = json::parse(&json).expect("Unable to parse json");

        let stats = MedicPerformance::extract_from_json(&json["players"]["[U:1:71020853]"])
            .expect("Unable to find medic performance");
        assert_eq!(stats.healing, 22732);
        assert_eq!(stats.average_uber_length_secs, 6.875);
        assert_eq!(stats.num_ubers, 12);
        assert_eq!(stats.num_drops, 0);
        assert_eq!(stats.deaths, 10);
        assert_eq!(stats.time_played_secs, 1738);
    }
}
