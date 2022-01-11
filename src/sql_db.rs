use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::RangeInclusive;

use postgres as sql;

use crate::class::Class;
use crate::database::Database;
use crate::logs_tf::search_params::SearchParams;
use crate::logs_tf::{self, Log, LogMetadata};
use crate::performance::Performance;
use crate::steam_id::SteamID;
use crate::SpecificPerformance;

/// Abstraction over a Postgresql database containing the saved mixes stats.
/// Requires a postgresql server to be running on the system. Make sure a role
/// with the name `mixes` exists and the database `mixes-stats` is present.
pub struct SQLDb
{
    client: sql::Client,
}

impl SQLDb
{
    /// Create the necessary tables in the database, in case they are not yet
    /// present.
    fn init_tables(&mut self) -> Result<(), sql::Error>
    {
        self.client.batch_execute(
            "CREATE TABLE IF NOT EXISTS users (
                steam_id bigint,
                discord_id bigint NOT NULL UNIQUE,
                PRIMARY KEY (steam_id)
            );
            CREATE TABLE IF NOT EXISTS logs (
                log_id OID,
                date timestamptz,
                map varchar(50),
                duration_secs int,
                num_players smallint,
                PRIMARY KEY (log_id)
            );
            CREATE TABLE IF NOT EXISTS dm_stats (
                log_id OID,
                steam_id bigint,
                class int,
                won_rounds smallint,
                num_rounds smallint,
                damage int,
                damage_taken int,
                kills smallint,
                deaths smallint,
                time_played_secs int
            );
            CREATE TABLE IF NOT EXISTS med_stats (
                log_id OID,
                steam_id bigint,
                won_rounds smallint,
                num_rounds smallint,
                damage_taken int,
                deaths smallint,
                time_played_secs int
            );
            ",
        )
    }

    /// Look up the ids of all logs already saved in the database. Since the
    /// data in them remains constant, they won't have to be queried again.
    /// They are always ordered by log_id descending, which means the newest
    /// logs are on the top. This is in accordance to the logs.tf API, which
    /// orders in the same manner.
    pub fn known_logs(&mut self) -> Result<Vec<u32>, sql::Error>
    {
        Ok(self
            .client
            .query("SELECT log_id FROM logs ORDER BY log_id DESC", &[])?
            .iter()
            .map(|row| row.get(0))
            .collect())
    }

    pub fn add_log(&mut self, log: Log) -> Result<(), sql::Error>
    {
        // Add log metadata to the logs table
        self.client.execute(
            "INSERT INTO logs (log_id, date, map, duration_secs, num_players) VALUES ($1, $2, $3, \
             $4, $5)",
            &[
                &log.meta().id,
                &log.meta().date_time,
                &log.meta().map,
                &log.duration_secs(),
                &(log.meta().num_players as i16),
            ],
        )?;

        // Add all performances of all players in the log
        for (steam_id, performances) in log.performances() {
            for performance in performances {
                match &performance.specific {
                    SpecificPerformance::DM(dm_perf) => {
                        self.client.execute(
                            "INSERT INTO dm_stats (log_id, steam_id, class, won_rounds, \
                             num_rounds, damage, damage_taken, kills, deaths, time_played_secs) \
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                            &[
                                &log.meta().id,
                                &(steam_id.id64() as i64),
                                &(dm_perf.class as i16),
                                &(performance.generic.won_rounds as i16),
                                &(performance.generic.num_rounds as i16),
                                &(dm_perf.damage as i32),
                                &(performance.generic.damage_taken as i32),
                                &(dm_perf.kills as i16),
                                &(dm_perf.deaths as i16),
                                &(dm_perf.time_played_secs),
                            ],
                        )?;
                    },
                    SpecificPerformance::Med(med_perf) => {
                        self.client.execute(
                            "INSERT INTO med_stats (log_id, steam_id, won_rounds, num_rounds, \
                             damage_taken, deaths, time_played_secs) VALUES ($1, $2, $3, $4, $5, \
                             $6, $7)",
                            &[
                                &log.meta().id,
                                &(steam_id.id64() as i64),
                                &(performance.generic.won_rounds as i16),
                                &(performance.generic.num_rounds as i16),
                                &(performance.generic.damage_taken),
                                &(med_perf.deaths as i16),
                                &(med_perf.time_played_secs),
                            ],
                        )?;
                    },
                }
            }
        }

        Ok(())
    }
}

impl Database for SQLDb
{
    type Error = sql::Error;

    fn start() -> Result<Self, Self::Error>
    {
        let client =
            sql::Client::connect("host=localhost user=mixes dbname=mixes-stats", sql::NoTls)?;
        let mut db = Self { client };

        db.init_tables()?;

        Ok(db)
    }

    fn add_user(&mut self, steam_id: SteamID, discord_id: u64) -> Result<bool, Self::Error>
    {
        // Convert to bigint
        let steam_id: i64 = steam_id.id64() as i64;
        let discord_id: i64 = discord_id as i64;
        // Check if the steam id or discord id is already in the database
        if self
            .client
            .query(
                "SELECT FROM users WHERE steam_id = $1 OR discord_id = $2",
                &[&steam_id, &discord_id],
            )?
            .is_empty()
        {
            // No entries yet. Add user to the database.
            self.client.execute(
                "INSERT INTO users (steam_id, discord_id) VALUES ($1, $2)",
                &[&steam_id, &discord_id],
            )?;

            Ok(true)
        }
        else {
            // Already in the database
            Ok(false)
        }
    }

    fn remove_user(&mut self, steam_id: SteamID) -> Result<bool, Self::Error> { todo!() }

    fn users(&mut self) -> Result<Vec<SteamID>, Self::Error>
    {
        Ok(self
            .client
            .query("SELECT steam_id FROM users", &[])?
            .iter()
            .map(|row| {
                let steam_id: i64 = row.get(0);
                SteamID::new_checked(steam_id as u64).expect("Invalid steam id in the database")
            })
            .collect())
    }

    fn update(&mut self, min_ratio: f32, num_players: RangeInclusive<u8>)
        -> Result<(), Self::Error>
    {
        let user_ids = self.users()?;
        let known_logs = self.known_logs()?;

        // HashMap of logs to be added. First, all the logs from every player unknown to
        // the database are added in here, together with a counter showing how many
        // (registered) players have an entry for that log, and have therefore
        // participated.
        let mut new_logs: HashMap<u32, (LogMetadata, u8)> = HashMap::new();
        for user_id in user_ids {
            let mut recent_logs =
                logs_tf::search_logs(SearchParams::player_id(user_id).add_limit(10000))
                    .expect("Unable to read players logs");

            // Remove all logs that are already in the database
            remove_external_occurrences(&mut recent_logs, &known_logs);

            // Remove logs that do not have the correct number of players (wrong game-type)
            recent_logs.drain_filter(|meta| !num_players.contains(&meta.num_players));

            // Add all found logs into the new logs hash-map.
            for log in recent_logs {
                match new_logs.get_mut(&log.id) {
                    Some((_, ref mut occ)) => *occ += 1,
                    None => {
                        new_logs.insert(log.id, (log, 1));
                    },
                }
            }
        }

        // Keep only the logs where enough mixes players were there, in accordance with
        // the ratio.
        new_logs.drain_filter(|_, (meta, occ)| {
            if meta.num_players != 0 {
                let ratio = *occ as f32 / meta.num_players as f32;

                ratio >= min_ratio
            }
            else {
                false
            }
        });

        // Download the new logs and add it to the database
        for (meta, _) in new_logs.values() {
            let log = Log::download(meta.id).expect("Failed to download a log");
            self.add_log(log)?;
        }

        Ok(())
    }

    fn get_class_performance(
        &mut self,
        user: SteamID,
        class: Class,
        limit: usize,
    ) -> Result<Vec<Performance>, Self::Error>
    {
        todo!()
    }
}

/// Takes two vectors, which are sorted in descending order and removes every
/// item from the first vector, which is already in the second vector.
fn remove_external_occurrences(target: &mut Vec<LogMetadata>, check: &[u32])
{
    if check.is_empty() || target.is_empty() {
        return;
    }

    // Walk through the fields from back to front and remove all items from target
    // which are contained in check. Back to front is used to ensure very little
    // vector reallocations should `check` be a superset of, or close to a superset
    // of `target`.
    //
    // WARNING: The indexes are is ONE-INDEXED to make checking for a done state
    // painless.
    let mut check_i = check.len();
    let mut target_i = target.len();
    while target_i != 0 && check_i != 0 {
        match target[target_i - 1].id.cmp(&check[check_i - 1]) {
            Ordering::Equal => {
                target.remove(target_i - 1);
                target_i -= 1;
            },
            Ordering::Less => check_i -= 1,
            Ordering::Greater => target_i -= 1,
        }
    }
}

#[cfg(test)]
mod tests
{
    use postgres::{Client, NoTls};

    use super::{Database, SQLDb};

    #[test]
    fn connect_to_db()
    {
        Client::connect("host=localhost user=mixes dbname=mixes-stats", NoTls)
            .expect("Unable to connect to the database. Make sure postgresql is set up correctly");
    }

    #[test]
    fn start() { let db = SQLDb::start().expect("Unable to connect to SQL database"); }
}
