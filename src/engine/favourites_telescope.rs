use std::{
    collections::HashMap,
    fmt::Display,
    fs::read_to_string,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use blaze_explorer_lib::{
    action::{Action, AppAction},
    app::App,
    components::explorer_table::{explorer_utils::FileConfig, ExplorerTable},
};

use crate::engine::{TelescopeResult, TelescopeSearch};

#[derive(Debug, Clone, PartialEq)]
pub struct FavouritesSearch {
    favourites: HashMap<usize, PathBuf>,
    results: Vec<SearchFavouritesResult>,
    last_search_timing: Option<Duration>,
}

impl FavouritesSearch {
    pub fn new(ctx: App) -> Self {
        let fav = ctx.config.favourites.clone();
        // attach a number in order to each element in favourites
        let fav_map = fav
            .iter()
            .enumerate()
            .map(|(i, path)| (i, path.clone()))
            .collect::<HashMap<usize, PathBuf>>();
        Self {
            favourites: fav_map,
            results: Vec::new(),
            last_search_timing: None,
        }
    }
}

impl TelescopeSearch for FavouritesSearch {
    fn search(&mut self, query: String) {
        let start = Instant::now();
        self.results = self
            .favourites
            .clone()
            .into_iter()
            .filter(|(id, path)| {
                id.to_string().contains(&query) || path.to_string_lossy().contains(&query)
            })
            .map(|(_id, path)| SearchFavouritesResult::new(path.to_str().unwrap().to_string()))
            .collect::<Vec<SearchFavouritesResult>>();
        self.last_search_timing = Some(start.elapsed());
    }

    fn get_results_list(&self) -> Vec<String> {
        self.results
            .iter()
            .map(|r| r.display())
            .collect::<Vec<String>>()
    }

    fn confirm_result(&mut self, id: usize) -> Option<Action> {
        let result = &self.results[id];
        let path = Path::new(&result.path).to_path_buf();
        Some(Action::AppAct(AppAction::ShowInFolder(path)))
    }

    fn preview_result(&self, id: Option<usize>, frame: &mut Frame, area: Rect) -> Result<()> {
        let preview_block = Block::default().borders(Borders::ALL).title("Preview");
        match id {
            Some(id) => {
                if let Some(result) = self.results.get(id) {
                    return result.preview(frame, area, preview_block);
                }
            }
            None => {
                frame.render_widget(Paragraph::default().block(preview_block), area);
            }
        }
        Ok(())
    }

    fn display(&self) -> String {
        let elapsed = match &self.last_search_timing {
            Some(d) => (d.as_millis() as f64 / 1000.0).to_string(),
            None => "".to_string(),
        };
        format!("Favourites - {}", elapsed)
    }

    fn n_results(&self) -> usize {
        self.results.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SearchFavouritesResult {
    path: String,
}

impl SearchFavouritesResult {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn render_folder_preview(&self, frame: &mut Frame, area: Rect) -> Result<()> {
        let mut explorer_table = ExplorerTable::new();
        let path = PathBuf::from(&self.path);
        explorer_table.update_path(path, None);
        let dummy_file_config = FileConfig {
            favourites: vec![],
            string_sequence: "".into(),
        };
        explorer_table.draw(frame, area, &dummy_file_config)
    }
}

impl TelescopeResult for SearchFavouritesResult {
    fn display(&self) -> String {
        self.path.clone()
    }

    fn preview(&self, frame: &mut Frame, area: Rect, preview_block: Block) -> Result<()> {
        self.render_folder_preview(frame, area)
    }

    fn from<S: ToString + Display>(s: S) -> Self {
        Self {
            path: s.to_string(),
        }
    }
}
