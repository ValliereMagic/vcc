use crate::show::*;

pub struct ShowsDb {
    connection: rusqlite::Connection,
}

impl ShowsDb {
    pub fn new() -> Self {
        let show_schema = "CREATE TABLE IF NOT EXISTS Shows (name TEXT, season_number INTEGER, episodes_seen INTEGER, category INTEGER)";

        let name_index = "CREATE INDEX IF NOT EXISTS idx_shows_name ON Shows (name)";

        let category_index = "CREATE INDEX IF NOT EXISTS idx_shows_category ON Shows (category)";

        let home_path = match std::env::var("HOME") {
            Ok(val) => val,
            _ => panic!("Unable to read HOME environment variable."),
        };

        let vcc_db_path = format!("{}/.local/share/vcc", home_path);
        std::fs::create_dir_all(&vcc_db_path).expect("Unable to create VCC Database directory.");

        let connection = rusqlite::Connection::open(format!("{}/shows.db", vcc_db_path)) //sqlite::open("./shows.db")
            .expect("Unable to Find show database.");

        connection
            .execute(show_schema, rusqlite::params![])
            .expect("Unable to create initial table.");

        connection
            .execute(name_index, rusqlite::params![])
            .expect("Unable to create name index.");

        connection
            .execute(category_index, rusqlite::params![])
            .expect("Unable to create category index.");

        ShowsDb { connection }
    }

    pub fn add(&self, show: &DisplayShow) {
        let add_query = "INSERT INTO Shows(name, season_number, episodes_seen, category) VALUES (?1, ?2, ?3, ?4)";

        let mut statement = self
            .connection
            .prepare(add_query)
            .expect("Unable to prepare add query.");

        statement
            .execute(rusqlite::params![
                show.name().as_str(),
                show.season_number,
                show.episodes_seen,
                show.category as i64
            ])
            .expect("Unable to insert show.");
    }

    pub fn remove(&self, show: &DisplayShow) {
        let remove_query = "DELETE from Shows WHERE name = ?1";

        let mut statement = self
            .connection
            .prepare(remove_query)
            .expect("Unable to prepare show delete query.");

        statement
            .execute(rusqlite::params![show.name().as_str()])
            .expect("Unable to delete show.");
    }

    pub fn update(&self, show: &DisplayShow) {
        let update_query = "UPDATE Shows SET season_number = ?1, episodes_seen = ?2, category = ?3 WHERE name = ?4";

        let mut statement = self
            .connection
            .prepare(update_query)
            .expect("Unable to prepare update query.");

        statement
            .execute(rusqlite::params![
                show.season_number,
                show.episodes_seen,
                show.category as i64,
                show.name().as_str()
            ])
            .expect("Unable to update show.");
    }

    pub fn load_all_shows(&self) -> impl Iterator<Item = DisplayShow> + '_ {
        let load_query = "SELECT * FROM Shows ORDER BY category, name COLLATE NOCASE";

        let mut statement = self
            .connection
            .prepare(load_query)
            .expect("Unable to prepare load query.");

        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get::<usize, i64>(3)?,
                ))
            })
            .expect("Unable to execute query.")
            .map(|result| result.expect("Unable to extract row."))
            .collect::<Vec<_>>();

        rows.into_iter()
            .map(|(name, season_number, episodes_seen, category)| {
                DisplayShow::new_numeric(
                    name,
                    season_number,
                    episodes_seen,
                    category
                        .try_into()
                        .expect("Unable to convert numeric category to ShowCategory."),
                )
            })
    }
}
