use std::{cmp::Ordering, rc::Rc};

#[derive(Copy, Clone, PartialEq)]
pub enum ShowCategory {
    Watching = 0,
    PlanToWatch = 1,
    Completed = 2,
}

impl Default for ShowCategory {
    fn default() -> Self {
        ShowCategory::PlanToWatch
    }
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

#[derive(Default, Clone)]
pub struct Show<S> {
    pub name: S,
    pub season_number: String,
    pub episodes_seen: String,
    pub category: ShowCategory,
}

pub type AdderShow = Show<String>;
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

impl Ord for DisplayShow {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for DisplayShow {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DisplayShow {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for DisplayShow {}

impl AdderShow {
    pub fn clear(&mut self) {
        self.name.clear();
        self.season_number.clear();
        self.episodes_seen.clear();
        self.category = ShowCategory::PlanToWatch;
    }
}

pub type CategorizedShows = [Vec<DisplayShow>; 3];
