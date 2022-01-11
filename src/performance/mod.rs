pub mod dm_performance;
pub mod generic_performance;
pub mod medic_performance;
pub mod score;

use dm_performance::DMPerformance;
use generic_performance::GenericPerformance;
use json::JsonValue;
use medic_performance::MedicPerformance;

use self::score::Score;

/// Performance that is specific for a class. Generalises the two types of
/// performance: On the one hand the DM performance, which is for classes
/// fitting that type, like demoman, scout and soldier. On the other hand the
/// medic class, which has a very different set of challenges and is handeled
/// seperately.
pub enum SpecificPerformance
{
    DM(DMPerformance),
    Med(MedicPerformance),
}

/// A `Performance` contains what a player has done in the course of a game. It
/// includes both generic performance, where data is not available on a per
/// class basis and the specific performance with information of that class.
pub struct Performance
{
    pub generic:  GenericPerformance,
    pub specific: SpecificPerformance,
}

impl Performance
{
    /// Turn the generic information parts and specific information parts into a
    /// combined dataset.
    pub fn from_parts<S>(generic: GenericPerformance, specific: S) -> Self
    where
        S: Into<SpecificPerformance>,
    {
        Self {
            generic,
            specific: specific.into(),
        }
    }

    pub fn extract_all_from_json(score: &Score, json: &JsonValue) -> Vec<Performance>
    {
        let generic_performance = GenericPerformance::from_json(score, json);
        let dm_performances = DMPerformance::extract_all_from_json(json);
        let med_performance = MedicPerformance::extract_from_json(json);

        let mut performances: Vec<Performance> = dm_performances
            .into_iter()
            .map(|dm_perf| Performance::from_parts(generic_performance.clone(), dm_perf))
            .collect();

        if let Some(med_performance) = med_performance {
            performances.push(Performance::from_parts(
                generic_performance,
                med_performance,
            ));
        }

        performances
    }
}
