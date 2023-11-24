use crate::show::*;

pub struct ShowsDb {
    connection: sqlite::Connection,
}

impl ShowsDb {
    pub fn new() -> Self {
        let show_schema = "CREATE TABLE IF NOT EXISTS Shows (name TEXT, season_number INTEGER, episodes_seen INTEGER, category INTEGER)";

        let home_path = match std::env::var("HOME") {
            Ok(val) => val,
            _ => panic!("Unable to read HOME environment variable."),
        };

        let vcc_db_path = format!("{}/.local/share/vcc", home_path);
        std::fs::create_dir_all(&vcc_db_path).expect("Unable to create VCC Database directory.");

        let connection = sqlite::open(format!("{}/shows.db", vcc_db_path)) //sqlite::open("./shows.db")
            .expect("Unable to Find show database.");
        connection
            .execute(show_schema)
            .expect("Unable to create initial table.");

        ShowsDb { connection }
    }

    pub fn add(&self, show: &DisplayShow) {
        let add_query =
            "INSERT INTO Shows(name, season_number, episodes_seen, category) VALUES (?, ?, ?, ?)";

        let mut statement = self
            .connection
            .prepare(add_query)
            .expect("Unable to prepare add query.");

        statement
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (1, (*(show.name)).to_owned().into()),
                    (2, show.season_number.to_owned().into()),
                    (3, show.episodes_seen.to_owned().into()),
                    (4, (show.category as i64).into()),
                ][..],
            )
            .expect("Unable to bind added values to query.");

        while statement.next().expect("Error adding show.") != sqlite::State::Done {}
    }

    pub fn remove(&self, show: &DisplayShow) {
        let remove_query = "DELETE from Shows WHERE name = ?";

        let mut statement = self
            .connection
            .prepare(remove_query)
            .expect("Unable to prepare show delete query.");
        statement
            .bind::<&[(_, sqlite::Value)]>(&[(1, (*(show.name)).to_owned().into())][..])
            .expect("Unable to bind values to query.");

        while statement.next().expect("Error deleting show.") != sqlite::State::Done {}
    }

    pub fn update(&self, show: &DisplayShow) {
        let update_query =
            "UPDATE Shows SET season_number = ?, episodes_seen = ?, category = ? WHERE name = ?";

        let mut statement = self
            .connection
            .prepare(update_query)
            .expect("Unable to prepare update query.");
        statement
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (1, show.season_number.to_owned().into()),
                    (2, show.episodes_seen.to_owned().into()),
                    (3, (show.category as i64).into()),
                    (4, (*(show.name)).to_owned().into()),
                ][..],
            )
            .expect("Unable to bind values to query.");

        while statement.next().expect("Error updating show.") != sqlite::State::Done {}
    }

    pub fn load_all_shows(&self) -> impl Iterator<Item = DisplayShow> + '_ {
        let load_query = "SELECT * FROM Shows ORDER BY category, name";

        self.connection
            .prepare(load_query)
            .unwrap()
            .into_iter()
            .map(|row| row.expect("Unable to read row."))
            .map(|row| {
                DisplayShow::new_numeric(
                    row.read::<&str, _>("name").to_owned(),
                    row.read::<i64, _>("season_number"),
                    row.read::<i64, _>("episodes_seen"),
                    row.read::<i64, _>("category")
                        .try_into()
                        .expect("Unable to extract category from the database."),
                )
            })
    }
}
