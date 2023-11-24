use crate::{
    show::{AdderShow, CategorizedShows, DisplayShow, ShowCategory},
    shows_db::ShowsDb,
};

use std::{collections::HashSet, str};
use trie_rs::TrieBuilder;

#[derive(Copy, Clone, PartialEq)]
pub enum UiShowCategory {
    Watching = 0,
    PlanToWatch = 1,
    Completed = 2,
    All = 3,
}

pub struct ShowsView {
    shows_db: ShowsDb,
    categorized_shows: CategorizedShows,
    ui_shows: Vec<DisplayShow>,
    search_term: String,
    current_category: UiShowCategory,
}

impl ShowsView {
    pub fn new() -> Self {
        let shows_db = ShowsDb::new();

        let mut categorized_shows = CategorizedShows::default();
        shows_db.load_all_shows().for_each(|show| {
            categorized_shows[show.category as usize].push(show);
        });

        // Present the user with the "Watching" category by default.
        let ui_shows = categorized_shows[ShowCategory::Watching as usize].to_owned();
        ShowsView {
            shows_db,
            categorized_shows,
            ui_shows,
            search_term: Default::default(),
            current_category: UiShowCategory::Watching,
        }
    }

    // Use the ui shows as buffers for user input, and rendering the ui
    pub fn iter(&mut self) -> impl Iterator<Item = (usize, &mut DisplayShow)> + '_ {
        self.ui_shows.iter_mut().enumerate()
    }

    fn recalculate_ui_shows(&mut self) {
        // Categorical
        if self.current_category == UiShowCategory::All {
            self.ui_shows = self.categorized_shows.iter().flatten().cloned().collect();
        } else {
            self.ui_shows = self.categorized_shows[self.current_category as usize]
                .iter()
                .cloned()
                .collect();
        }

        if self.search_term.is_empty() {
            return;
        }

        // Filter by search term
        let mut trie_builder = TrieBuilder::new();

        for show in self.ui_shows.iter() {
            trie_builder.push(&*show.name);
        }

        let trie = trie_builder.build();

        let results = trie
            .predictive_search(&self.search_term)
            .into_iter()
            .map(|u8s| str::from_utf8(&u8s).unwrap().to_owned())
            .collect::<HashSet<String>>();

        self.ui_shows = self
            .ui_shows
            .drain(..)
            .filter(|show| results.contains(show.name.as_str()))
            .collect();
    }

    pub fn search(&mut self) {
        self.recalculate_ui_shows();
    }

    pub fn search_box(&mut self) -> &mut String {
        &mut self.search_term
    }

    pub fn update_category(&mut self) {
        self.recalculate_ui_shows();
    }

    pub fn current_category(&mut self) -> &mut UiShowCategory {
        &mut self.current_category
    }

    pub fn add(&mut self, show: AdderShow) {
        let show = DisplayShow::new(
            show.name,
            show.season_number,
            show.episodes_seen,
            show.category,
        );

        let index = match self.categorized_shows[show.category as usize].binary_search(&show) {
            Ok(_) => return,
            Err(idx) => idx,
        };

        self.shows_db.add(&show);

        self.categorized_shows[show.category as usize].insert(index, show);

        self.recalculate_ui_shows();
    }

    fn find_categorized_show<'a>(&'a self, show: &DisplayShow) -> Option<(usize, &DisplayShow)> {
        let mut categorized = None;

        let show_finder = |shows: &'a Vec<DisplayShow>| -> Option<(usize, &'a DisplayShow)> {
            match shows.binary_search(&show) {
                Ok(idx) => Some((idx, &shows[idx])),
                Err(_) => None,
            }
        };

        if self.current_category == UiShowCategory::All {
            // The show could be in any of the 3 categories
            for shows in self.categorized_shows.iter() {
                categorized = show_finder(shows);

                if categorized.is_some() {
                    break;
                }
            }
            categorized
        } else {
            show_finder(&self.categorized_shows[self.current_category as usize])
        }
    }

    pub fn update(&mut self, ui_index: usize) {
        let show = &self.ui_shows[ui_index];
        let mut recalculate = false;

        if let Some((categorized_index, categorized_show)) = self.find_categorized_show(show) {
            if categorized_show.category != show.category {
                // Need to take the show out of the category it no longer
                // belongs to and put it into the one it's changed to.
                let new_index =
                    match self.categorized_shows[show.category as usize].binary_search(&show) {
                        Ok(_) => return,
                        Err(idx) => idx,
                    };

                self.categorized_shows[categorized_show.category as usize]
                    .remove(categorized_index);
                self.categorized_shows[show.category as usize].insert(new_index, show.to_owned());

                recalculate = true;
            } else {
                self.categorized_shows[categorized_show.category as usize][categorized_index] =
                    show.to_owned();
            }
            self.shows_db.update(&show);

            if recalculate {
                self.recalculate_ui_shows();
            }
        }
    }

    pub fn remove(&mut self, ui_index: usize) {
        let show = self.ui_shows.remove(ui_index);

        let (categorized_index, show) = match self.find_categorized_show(&show) {
            Some(categorized) => categorized,
            None => return,
        };

        let show = self.categorized_shows[show.category as usize].remove(categorized_index);

        self.shows_db.remove(&show);
    }
}
