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
                steam_id varchar(20),
                discord_id varchar(80)
            );
            CREATE TABLE IF NOT EXISTS logs (
                log_id int,
                date int,
                map varchar(50),
                length_secs int
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

    fn add_user(&mut self, steam_id: SteamID, discord_id: String) -> Result<bool, Self::Error>
    {
        todo!()
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
