use std::rc::Rc;

#[derive(Default)]
pub struct Show<S> {
    pub name: S,
    pub season_number: String,
    pub episodes_seen: String,
}

impl Show<Rc<String>> {
    pub fn new(name: String, season_number: String, episodes_seen: String) -> Show<Rc<String>> {
        Show {
            name: Rc::new(name),
            season_number,
            episodes_seen,
        }
    }
    pub fn new_numeric(name: String, season_number: i64, episodes_seen: i64) -> Show<Rc<String>> {
        Show::new(
            name,
            format!("{}", season_number),
            format!("{}", episodes_seen),
        )
    }
}

impl Show<String> {
    pub fn clear(&mut self) {
        self.name.clear();
        self.season_number.clear();
        self.episodes_seen.clear();
    }
}

pub struct Shows {
    connection: sqlite::Connection,
    shows: Vec<Show<Rc<String>>>,
}

impl Shows {
    pub fn new() -> Self {
        let show_schema = "CREATE TABLE IF NOT EXISTS Shows (name TEXT, season_number INTEGER, episodes_seen INTEGER)";

        let home_path = match std::env::var("HOME") {
            Ok(val) => val,
            _ => panic!("Unable to read HOME environment variable."),
        };

        let vcc_db_path = format!("{}/.local/share/vcc", home_path);
        std::fs::create_dir_all(&vcc_db_path).expect("Unable to create VCC Database directory.");

        let connection = sqlite::open(format!("{}/shows.db", vcc_db_path))
            .expect("Unable to Find show database.");
        connection
            .execute(show_schema)
            .expect("Unable to create initial table.");

        let mut shows = Shows {
            connection,
            shows: Vec::default(),
        };
        shows.load_all_shows();
        shows
    }

    pub fn iter(&mut self) -> std::slice::IterMut<'_, Show<Rc<String>>> {
        self.shows.iter_mut()
    }

    pub fn add(&mut self, name: String, season_number: i64, episodes_seen: i64) {
        let add_query = "INSERT INTO Shows(name, season_number, episodes_seen) VALUES (?, ?, ?)";
        let mut statement = self
            .connection
            .prepare(add_query)
            .expect("Unable to prepare add query.");
        statement
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (1, name.to_owned().into()),
                    (2, season_number.into()),
                    (3, episodes_seen.into()),
                ][..],
            )
            .expect("Unable to bind added values to query.");
        while statement.next().expect("Error adding show.") != sqlite::State::Done {}

        self.shows
            .push(Show::new_numeric(name, season_number, episodes_seen));
    }

    pub fn remove(&mut self, name: &str) {
        let remove_query = "DELETE from Shows WHERE name = ?";

        let Some(show_idx) = self.shows.iter().position(|x| *x.name == name) else {
            return;
        };

        let mut statement = self
            .connection
            .prepare(remove_query)
            .expect("Unable to prepare show delete query.");
        statement
            .bind::<&[(_, sqlite::Value)]>(&[(1, name.into())][..])
            .expect("Unable to bind values to query.");
        while statement.next().expect("Error deleting show.") != sqlite::State::Done {}
        self.shows.remove(show_idx);
    }

    pub fn update(&mut self, name: &str, season_number: Option<i64>, episodes_seen: Option<i64>) {
        let update_season_number_query = "UPDATE Shows SET season_number = ? WHERE name = ?";
        let update_episodes_number_query = "UPDATE Shows SET episodes_seen = ? WHERE name = ?";

        let Some(show) = self.shows.iter_mut().find(|show| *show.name == name) else {
            return;
        };

        let update_field = |field: i64, name: &str, statement: &str| -> String {
            let mut statement = self
                .connection
                .prepare(statement)
                .expect("Unable to prepare season number update query.");
            statement
                .bind::<&[(_, sqlite::Value)]>(&[(1, field.into()), (2, name.into())][..])
                .expect("Unable to bind values to query.");
            while statement.next().expect("Error updating show.") != sqlite::State::Done {}
            format!("{}", field)
        };

        if let Some(season_number) = season_number {
            show.season_number = update_field(season_number, name, update_season_number_query);
        }

        if let Some(episodes_seen) = episodes_seen {
            show.episodes_seen = update_field(episodes_seen, name, update_episodes_number_query);
        }
    }

    fn load_all_shows(&mut self) {
        let load_query = "SELECT * FROM Shows";
        for row in self
            .connection
            .prepare(load_query)
            .unwrap()
            .into_iter()
            .map(|row| row.expect("Unable to read row."))
        {
            self.shows.push(Show::new_numeric(
                row.read::<&str, _>("name").to_owned(),
                row.read::<i64, _>("season_number"),
                row.read::<i64, _>("episodes_seen"),
            ))
        }
    }
}
