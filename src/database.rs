use crate::{Class, Performance, SteamID};

pub trait Database: Sized
{
    type Error;

    /// Start necessary database services, create files, directories or tables
    /// in a database as needed.
    fn start() -> Result<Self, Self::Error>;

    /// Add a user to be tracked as a mixes player.
    ///
    /// # Returns
    /// `true` if the player was successfully added, `false` if there is already
    /// a player with the same `steam_id` or `discord_id`. Returns an Error if
    /// anything during registering goes wrong.
    fn add_user(&mut self, steam_id: SteamID, discord_id: u64) -> Result<bool, Self::Error>;

    /// Retrieve the latest logs of the mixes players from logs.tf. Ignores
    /// games that do not contain enough mixes players. The amount of mixes
    /// players needed in one game is governed by the `min_ratio` variable,
    /// which must be between `0` (include all logs) and `1` (include only logs
    /// where all players are registered as mixes players).
    ///
    /// # Panics
    /// If `0 <= min_ratio <= 1` is *not* true.
    fn update(&mut self, min_ratio: f32) -> Result<(), Self::Error>;

    /// Get the most recent performance records (stats) of the player described
    /// by the `user`. Only logs where the player has played `class` for any
    /// amount of time are included. The damage/healing stats are accurate for
    /// that class, however the win-rate is over the entire log, not only for
    /// that class. The `limit` describes the number of most recent logs that
    /// should be included in the report, therefore the resulting vector
    /// containing the latest performances will contain at most `limit`
    /// elements.
    ///
    /// # Returns
    /// vector containing the latest performances of the player on the given
    /// class, sorted from newest logs to oldest.
    fn get_class_performance(
        &mut self,
        user: SteamID,
        class: Class,
        limit: usize,
    ) -> Result<Vec<Performance>, Self::Error>;
}
