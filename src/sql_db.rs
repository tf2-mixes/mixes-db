use postgres as sql;

use crate::class::Class;
use crate::database::Database;
use crate::performance::Performance;
use crate::steam_id::SteamID;

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
                log_id int,
                date int,
                map varchar(50),
                length_secs int,
                PRIMARY KEY (log_id)
            );
            CREATE TABLE IF NOT EXISTS stats (
                log_id int,
                steam_id varchar(20),
                class int,
                won_rounds int,
                lost_rounds int,
                damage int,
                healing int,
                damage_taken int,
                kills int,
                deaths int,
                time_played_secs int
            );",
        )
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

    fn update(&mut self, min_ratio: f32) -> Result<(), Self::Error> { todo!() }

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
