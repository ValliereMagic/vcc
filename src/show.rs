use std::{cmp::Ordering, rc::Rc};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum ShowCategory {
    Watching = 0,
    #[default]
    PlanToWatch = 1,
    Completed = 2,
}

impl TryFrom<i64> for ShowCategory {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            v if v == ShowCategory::Watching as i64 => Ok(ShowCategory::Watching),
            v if v == ShowCategory::PlanToWatch as i64 => Ok(ShowCategory::PlanToWatch),
            v if v == ShowCategory::Completed as i64 => Ok(ShowCategory::Completed),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
pub struct Show<S> {
    pub name: S,
    pub season_number: String,
    pub episodes_seen: String,
    pub category: ShowCategory,
}

pub type AdderShow = Show<String>;

impl AdderShow {
    fn default_numeric_string() -> String {
        "0".to_owned()
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.season_number = AdderShow::default_numeric_string();
        self.episodes_seen = AdderShow::default_numeric_string();
        self.category = ShowCategory::PlanToWatch;
    }
}

impl Default for AdderShow {
    fn default() -> Self {
        Self {
            name: Default::default(),
            season_number: AdderShow::default_numeric_string(),
            episodes_seen: AdderShow::default_numeric_string(),
            category: Default::default(),
        }
    }
}

pub type DisplayShow = Show<Rc<String>>;

impl DisplayShow {
    pub fn new(
        name: String,
        season_number: String,
        episodes_seen: String,
        category: ShowCategory,
    ) -> DisplayShow {
        Show {
            name: Rc::new(name),
            season_number,
            episodes_seen,
            category,
        }
    }

    pub fn new_numeric(
        name: String,
        season_number: i64,
        episodes_seen: i64,
        category: ShowCategory,
    ) -> DisplayShow {
        Show::new(
            name,
            format!("{}", season_number),
            format!("{}", episodes_seen),
            category,
        )
    }
}

impl Default for DisplayShow {
    fn default() -> Self {
        Self {
            name: Default::default(),
            season_number: Default::default(),
            episodes_seen: Default::default(),
            category: Default::default(),
        }
    }
}

impl Ord for DisplayShow {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for DisplayShow {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DisplayShow {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for DisplayShow {}

pub type CategorizedShows = [Vec<DisplayShow>; 3];
