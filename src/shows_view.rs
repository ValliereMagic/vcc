use crate::{
    show::{AdderShow, CategorizedShows, DisplayShow, ShowCategory},
    shows_db::ShowsDb,
};

use std::{collections::HashMap, str};
use trie_rs::TrieBuilder;

#[derive(Copy, Clone, PartialEq)]
pub enum UiShowCategory {
    Watching = 0,
    PlanToWatch = 1,
    Completed = 2,
    All = 3,
}

const SHOWS_PER_PAGE: usize = 10;

pub struct ShowsView {
    shows_db: ShowsDb,
    categorized_shows: CategorizedShows,
    ui_shows: Vec<DisplayShow>,
    search_term: String,
    current_category: UiShowCategory,
    page_number: usize,
    page_count: usize,
}

impl ShowsView {
    fn calculate_num_pages(ui_shows_len: usize) -> usize {
        usize::max(
            ((ui_shows_len as f64 / SHOWS_PER_PAGE as f64).ceil()) as usize,
            1,
        )
    }

    pub fn new() -> Self {
        let shows_db = ShowsDb::new();

        let mut categorized_shows = CategorizedShows::default();
        shows_db.load_all_shows().for_each(|show| {
            categorized_shows[show.category as usize].push(show);
        });

        // Present the user with the "Watching" category by default.
        let ui_shows = categorized_shows[ShowCategory::Watching as usize].to_owned();
        let ui_shows_len = ui_shows.len();

        ShowsView {
            shows_db,
            categorized_shows,
            ui_shows,
            search_term: Default::default(),
            current_category: UiShowCategory::Watching,
            page_number: 1,
            page_count: ShowsView::calculate_num_pages(ui_shows_len),
        }
    }

    // Use the ui shows as buffers for user input, and rendering the ui
    pub fn iter(&mut self) -> impl Iterator<Item = (usize, &mut DisplayShow)> + '_ {
        // ui_shows is empty
        if self.ui_shows.is_empty() {
            (0usize..).into_iter().zip(self.ui_shows.iter_mut())
        } else {
            let begin_inclusive = (self.page_number - 1) * SHOWS_PER_PAGE;
            let end_exclusive = usize::min(self.page_number * SHOWS_PER_PAGE, self.ui_shows.len());

            let offset_index = subslice_index::subslice_index(
                &self.ui_shows[..],
                &self.ui_shows[begin_inclusive..end_exclusive],
            );

            (offset_index..)
                .into_iter()
                .zip(self.ui_shows[begin_inclusive..end_exclusive].iter_mut())
        }
    }

    fn recalculate_ui_shows(&mut self) {
        self.ui_shows = match (self.current_category, self.search_term.is_empty()) {
            (UiShowCategory::All, true) => {
                self.categorized_shows.iter().flatten().cloned().collect()
            }
            (_, false) => {
                let mut trie_builder = TrieBuilder::new();
                let mut case_map = HashMap::new();

                type DisplayShowIter<'a> = Box<dyn Iterator<Item = &'a DisplayShow> + 'a>;
                match self.current_category {
                    UiShowCategory::All => {
                        Box::new(self.categorized_shows.iter().flatten()) as DisplayShowIter
                    }
                    _ => Box::new(self.categorized_shows[self.current_category as usize].iter())
                        as DisplayShowIter,
                }
                .for_each(|show| {
                    trie_builder.push(show.name.to_lowercase());
                    case_map.insert(show.name.to_lowercase(), &show.name);
                });

                let trie = trie_builder.build();
                let mut show = DisplayShow::default();
                trie.predictive_search(self.search_term.to_lowercase())
                    .into_iter()
                    .filter_map(|u8_rep| {
                        show.name = case_map[str::from_utf8(&u8_rep).unwrap()].to_owned();

                        match self.find_categorized_show(&show) {
                            None => None,
                            Some(show) => Some(show.1.to_owned()),
                        }
                    })
                    .collect()
            }
            (_, true) => self.categorized_shows[self.current_category as usize].to_owned(),
        };
        self.page_number = 1;
        self.page_count = ShowsView::calculate_num_pages(self.ui_shows.len());
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

    pub fn next_page(&mut self) {
        // User has gone past the last page
        if (self.page_number + 1) > self.page_count {
            return;
        } else {
            self.page_number += 1
        }
    }

    pub fn previous_page(&mut self) {
        // User has gone before the first page
        if (self.page_number - 1) < 1 {
            return;
        } else {
            self.page_number -= 1
        }
    }

    pub fn page(&self) -> usize {
        self.page_number
    }

    pub fn page_count(&self) -> usize {
        self.page_count
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
        let show_finder = |shows: &'a Vec<DisplayShow>| -> Option<(usize, &'a DisplayShow)> {
            match shows.binary_search(&show) {
                Ok(idx) => Some((idx, &shows[idx])),
                Err(_) => None,
            }
        };

        match self.current_category {
            UiShowCategory::All => {
                // The show could be in any of the 3 categories
                let mut categorized = None;
                for shows in self.categorized_shows.iter() {
                    categorized = show_finder(shows);

                    if categorized.is_some() {
                        break;
                    }
                }
                categorized
            }
            _ => show_finder(&self.categorized_shows[self.current_category as usize]),
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
