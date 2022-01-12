pub mod dm_performance;
pub mod medic_performance;
pub mod overall_performance;
pub mod score;

use dm_performance::DMPerformance;
use json::JsonValue;
use medic_performance::MedicPerformance;
use overall_performance::OverallPerformance;

use self::score::Score;

/// A `Performance` contains what a player has done in the course of a game. It
/// contains either a generic performance, where data is not available on a per
/// class basis and the specific performance with information of that class,
/// being either a DM class or the medic.
pub enum Performance
{
    Overall(OverallPerformance),
    DM(DMPerformance),
    Med(MedicPerformance),
}

impl Performance
{
    pub fn extract_all_from_json(score: &Score, json: &JsonValue) -> Vec<Performance>
    {
        let overall_performance = OverallPerformance::from_json(score, json);
        let dm_performances = DMPerformance::extract_all_from_json(json);
        let med_performance = MedicPerformance::extract_from_json(json);

        let mut performances = vec![overall_performance.into()];

        for dm_perf in dm_performances {
            performances.push(dm_perf.into());
        }

        if let Some(med_performance) = med_performance {
            performances.push(med_performance.into());
        }

        performances
    }
}
