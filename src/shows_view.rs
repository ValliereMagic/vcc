use std::{iter::Zip, ops::RangeFrom, slice::IterMut};

use memchr::memmem;
use strumbra::UniqueString;

use crate::{
    show::{AdderShow, CategorizedShows, DisplayShow, ShowCategory},
    shows_db::ShowsDb,
};

#[derive(Copy, Clone, PartialEq)]
pub enum UiShowCategory {
    Watching = 0,
    PlanToWatch = 1,
    Completed = 2,
    All = 3,
}

impl From<ShowCategory> for UiShowCategory {
    fn from(value: ShowCategory) -> Self {
        match value {
            ShowCategory::Watching => UiShowCategory::Watching,
            ShowCategory::PlanToWatch => UiShowCategory::PlanToWatch,
            ShowCategory::Completed => UiShowCategory::Completed,
        }
    }
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
    pub fn iter_mut(&mut self) -> Zip<RangeFrom<usize>, IterMut<'_, DisplayShow>> {
        // ui_shows is empty
        if self.ui_shows.is_empty() {
            return (0usize..).zip(self.ui_shows.iter_mut());
        }

        let begin_inclusive = (self.page_number - 1) * SHOWS_PER_PAGE;
        let end_exclusive = usize::min(self.page_number * SHOWS_PER_PAGE, self.ui_shows.len());

        let offset_index = subslice_index::subslice_index(
            &self.ui_shows[..],
            &self.ui_shows[begin_inclusive..end_exclusive],
        );

        (offset_index..).zip(self.ui_shows[begin_inclusive..end_exclusive].iter_mut())
    }

    fn recalculate_ui_shows(&mut self) {
        self.ui_shows = match (self.current_category, self.search_term.is_empty()) {
            (UiShowCategory::All, true) => {
                self.categorized_shows.iter().flatten().cloned().collect()
            }
            (_, false) => {
                let lower_search_term =
                    UniqueString::try_from(self.search_term.to_lowercase()).unwrap();
                let searcher = memmem::Finder::new(lower_search_term.as_bytes());

                type DisplayShowIter<'a> = Box<dyn Iterator<Item = &'a DisplayShow> + 'a>;
                match self.current_category {
                    UiShowCategory::All => {
                        Box::new(self.categorized_shows.iter().flatten()) as DisplayShowIter
                    }
                    _ => Box::new(self.categorized_shows[self.current_category as usize].iter())
                        as DisplayShowIter,
                }
                .filter_map(|show| {
                    searcher
                        .find(show.lower_name().as_bytes())
                        .map(|_| show.to_owned())
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
        }
        self.page_number += 1
    }

    pub fn previous_page(&mut self) {
        // User has gone before the first page
        if (self.page_number - 1) < 1 {
            return;
        }
        self.page_number -= 1
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

        // Make sure someone isn't adding a show that already exists.
        let insert_index = match self.find_categorized_show(&show, Some(show.category as usize)) {
            Ok((_, existing_show)) => {
                let existing_show = existing_show.to_owned();

                self.current_category = UiShowCategory::All;
                self.search_term = existing_show.name().as_str().to_owned();

                self.recalculate_ui_shows();
                return;
            }
            Err(insert_index) => insert_index,
        };

        self.shows_db.add(&show);

        self.current_category = show.category.into();
        self.categorized_shows[show.category as usize].insert(insert_index, show);

        self.recalculate_ui_shows();
    }

    fn find_categorized_show<'a>(
        &'a self,
        show: &DisplayShow,
        add_category: Option<usize>,
    ) -> Result<(usize, &'a DisplayShow), usize> {
        let show_finder = |shows: &'a Vec<DisplayShow>| -> Result<(usize, &DisplayShow), usize> {
            match shows.binary_search(show) {
                Ok(idx) => Ok((idx, &shows[idx])),
                Err(idx) => Err(idx),
            }
        };

        match (self.current_category, add_category) {
            (_, Some(_)) | (UiShowCategory::All, None) => {
                let mut result = Err(usize::MAX);
                for current_category in
                    (ShowCategory::Watching as usize)..(ShowCategory::Completed as usize + 1)
                {
                    match show_finder(&self.categorized_shows[current_category]) {
                        // Found the show, game over.
                        Ok((index, show)) => {
                            result = Ok((index, show));
                            break;
                        }
                        // Found the index to insert the show at in add_category.
                        // Set result to it, but understand that it could be
                        // overwritten by a found on the next iteration.
                        Err(index) if matches!(add_category, Some(add_category) if add_category == current_category) => {
                            result = Err(index)
                        }
                        // Not found, and not in a category we care about.
                        Err(_) => (),
                    }
                }
                result
            }
            _ => show_finder(&self.categorized_shows[self.current_category as usize]),
        }
    }

    pub fn update(&mut self, ui_index: usize) {
        let show = self.ui_shows[ui_index].to_owned();

        let Ok((categorized_index, categorized_show)) = self.find_categorized_show(&show, None)
        else {
            return;
        };

        self.shows_db.update(&show);

        if categorized_show.category == show.category {
            self.categorized_shows[categorized_show.category as usize][categorized_index] = show;
            return;
        }

        let Err(new_index) = self.categorized_shows[show.category as usize].binary_search(&show)
        else {
            return;
        };

        self.categorized_shows[categorized_show.category as usize].remove(categorized_index);
        self.current_category = show.category.into();
        self.categorized_shows[show.category as usize].insert(new_index, show);

        self.recalculate_ui_shows();
    }

    pub fn remove(&mut self, ui_index: usize) {
        let show = self.ui_shows.remove(ui_index);

        if let Ok((categorized_index, show)) = self.find_categorized_show(&show, None) {
            let show = self.categorized_shows[show.category as usize].remove(categorized_index);
            self.shows_db.remove(&show);
        }
    }
}
