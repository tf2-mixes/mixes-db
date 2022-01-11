use std::str::FromStr;

use json::JsonValue;

use super::SpecificPerformance;
use crate::Class;

pub struct DMPerformance
{
    pub class:            Class,
    pub kills:            u8,
    pub assists:          u8,
    pub deaths:           u8,
    pub damage:           u32,
    pub time_played_secs: u32,
}

impl DMPerformance
{
    pub fn extract_all_from_json(json: &JsonValue) -> Vec<Self>
    {
        json["class_stats"]
            .members()
            .map(|class_stats| Self {
                class:            Class::from_str(class_stats["type"].as_str().unwrap()).unwrap(),
                kills:            class_stats["kills"].as_u8().unwrap(),
                assists:          class_stats["assists"].as_u8().unwrap(),
                deaths:           class_stats["deaths"].as_u8().unwrap(),
                damage:           class_stats["dmg"].as_u32().unwrap(),
                time_played_secs: class_stats["total_time"].as_u32().unwrap(),
            })
            .collect()
    }
}

impl Into<SpecificPerformance> for DMPerformance
{
    fn into(self) -> SpecificPerformance { SpecificPerformance::DM(self) }
}

#[cfg(test)]
mod tests
{
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn extract_all_from_json()
    {
        let mut json = String::new();
        File::open("test_data/log_3094861.json")
            .expect("Unable to open test file")
            .read_to_string(&mut json)
            .expect("Unable to read file to string");
        let json = json::parse(&json).expect("Unable to parse json");

        let perfs = DMPerformance::extract_all_from_json(&json["players"]["[U:1:886717065]"]);

        assert_eq!(perfs.len(), 3);
        let scout_perf = &perfs[0];
        let engi_perf = &perfs[1];
        let _pyro_perf = &perfs[2];

        assert_eq!(scout_perf.class, Class::Scout);
        assert_eq!(scout_perf.kills, 19);
        assert_eq!(scout_perf.assists, 14);
        assert_eq!(scout_perf.deaths, 16);
        assert_eq!(scout_perf.damage, 6671);
        assert_eq!(scout_perf.time_played_secs, 1618);

        assert_eq!(engi_perf.class, Class::Engineer);
        assert_eq!(engi_perf.kills, 0);
        assert_eq!(engi_perf.assists, 2);
        assert_eq!(engi_perf.deaths, 0);
        assert_eq!(engi_perf.damage, 293);
        assert_eq!(engi_perf.time_played_secs, 99);
    }
}
