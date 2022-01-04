use crate::class::Class;

/// The performance describes what a player has done or what happened to a
/// player over the course of an entire game. If the `class` attribute is set,
/// everything except for the win/loss ratio describes the performance on that
/// class. If it is not, it is an average over the entire game, on all classes
/// the player has played.
pub struct Performance
{
    pub class:            Class,
    pub won_rounds:       u8,
    pub lost_rounds:      u8,
    pub damage:           u32,
    pub healing:          u32,
    pub damage_taken:     u32,
    pub kills:            u32,
    pub deaths:           u32,
    pub time_played_secs: u32,
}
