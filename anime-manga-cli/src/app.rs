use crate::anime_api::{Anime, Episode, StreamLink};
use crate::api::{Chapter, Manga};

pub enum Screen {
    MainMenu,
    Search,
    Results,
    Chapters,
    Episodes,
    Loading,
    QualityPick,
    Reader,
}

pub enum Mode {
    Manga,
    Anime,
}

pub struct Category {
    pub label: &'static str,
    pub icon: &'static str,
    pub query: &'static str,
}

pub enum SearchMode {
    Query,  // typing a search
    Browse, // browsing categories
}

pub const MANGA_CATEGORIES: &[Category] = &[
    Category {
        label: "Trending",
        icon: "🔥",
        query: "one piece",
    },
    Category {
        label: "Most Popular",
        icon: "⭐",
        query: "naruto",
    },
    Category {
        label: "Action",
        icon: "⚔️",
        query: "action",
    },
    Category {
        label: "Romance",
        icon: "💕",
        query: "romance",
    },
    Category {
        label: "Fantasy",
        icon: "🧙",
        query: "fantasy",
    },
    Category {
        label: "Horror",
        icon: "👻",
        query: "horror",
    },
    Category {
        label: "Sci-Fi",
        icon: "🚀",
        query: "sci-fi",
    },
    Category {
        label: "Slice of Life",
        icon: "🌸",
        query: "slice of life",
    },
    Category {
        label: "Sports",
        icon: "⚽",
        query: "sports",
    },
    Category {
        label: "Mecha",
        icon: "🤖",
        query: "mecha",
    },
    Category {
        label: "Mystery",
        icon: "🔍",
        query: "mystery",
    },
    Category {
        label: "Comedy",
        icon: "😂",
        query: "comedy",
    },
];

pub const ANIME_CATEGORIES: &[Category] = &[
    Category {
        label: "Trending",
        icon: "🔥",
        query: "attack on titan",
    },
    Category {
        label: "Most Popular",
        icon: "⭐",
        query: "naruto",
    },
    Category {
        label: "Top Rated",
        icon: "🏆",
        query: "fullmetal alchemist",
    },
    Category {
        label: "Action",
        icon: "⚔️",
        query: "action",
    },
    Category {
        label: "Romance",
        icon: "💕",
        query: "romance",
    },
    Category {
        label: "Fantasy",
        icon: "🧙",
        query: "fantasy",
    },
    Category {
        label: "Horror",
        icon: "👻",
        query: "horror",
    },
    Category {
        label: "Sci-Fi",
        icon: "🚀",
        query: "sci-fi",
    },
    Category {
        label: "Slice of Life",
        icon: "🌸",
        query: "slice of life",
    },
    Category {
        label: "Sports",
        icon: "⚽",
        query: "sports",
    },
    Category {
        label: "Mecha",
        icon: "🤖",
        query: "mecha",
    },
    Category {
        label: "Isekai",
        icon: "🌀",
        query: "isekai",
    },
];

pub struct App {
    pub screen: Screen,
    pub mode: Mode,
    pub search_input: String,
    pub results: Vec<Manga>,
    pub selected: usize,
    pub chapters: Vec<Chapter>,
    pub chapter_selected: usize,
    pub pages: Vec<String>,
    pub current_page: usize,
    pub loaded_pages: usize,
    pub total_pages: usize,
    pub tick: u64,
    pub status: String,
    pub should_quit: bool,
    pub search_mode: SearchMode,
    pub category_selected: usize,
    pub anime_results: Vec<Anime>,
    pub anime_selected: usize,
    pub episodes: Vec<Episode>,
    pub episode_selected: usize,
    pub stream_links: Vec<StreamLink>,
    pub quality_selected: usize,
}

impl App {
    pub fn new() -> Self {
        App {
            screen: Screen::MainMenu,
            mode: Mode::Manga,
            search_input: String::new(),
            results: vec![],
            selected: 0,
            chapters: vec![],
            chapter_selected: 0,
            pages: vec![],
            current_page: 0,
            loaded_pages: 0,
            total_pages: 0,
            tick: 0,
            status: String::from("Type to search, Enter to confirm"),
            should_quit: false,
            anime_results: vec![],
            anime_selected: 0,
            episodes: vec![],
            episode_selected: 0,
            search_mode: SearchMode::Browse,
            category_selected: 0,
            stream_links: vec![],
            quality_selected: 0,
        }
    }

    pub fn spinner(&self) -> &str {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        frames[(self.tick as usize) % frames.len()]
    }

    pub fn move_up(&mut self) {
        match self.screen {
            Screen::Results => match self.mode {
                Mode::Manga => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                Mode::Anime => {
                    if self.anime_selected > 0 {
                        self.anime_selected -= 1;
                    }
                }
            },
            Screen::Chapters => {
                if self.chapter_selected > 0 {
                    self.chapter_selected -= 1;
                }
            }
            Screen::Episodes => {
                if self.episode_selected > 0 {
                    self.episode_selected -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn move_down(&mut self) {
        match self.screen {
            Screen::Results => match self.mode {
                Mode::Manga => {
                    if self.selected + 1 < self.results.len() {
                        self.selected += 1;
                    }
                }
                Mode::Anime => {
                    if self.anime_selected + 1 < self.anime_results.len() {
                        self.anime_selected += 1;
                    }
                }
            },
            Screen::Chapters => {
                if self.chapter_selected + 1 < self.chapters.len() {
                    self.chapter_selected += 1;
                }
            }
            Screen::Episodes => {
                if self.episode_selected + 1 < self.episodes.len() {
                    self.episode_selected += 1;
                }
            }
            _ => {}
        }
    }

    pub fn selected_manga_id(&self) -> Option<&str> {
        self.results.get(self.selected).map(|m| m.id.as_str())
    }

    pub fn selected_chapter_id(&self) -> Option<&str> {
        self.chapters
            .get(self.chapter_selected)
            .map(|c| c.id.as_str())
    }

    pub fn selected_anime(&self) -> Option<&Anime> {
        self.anime_results.get(self.anime_selected)
    }

    pub fn selected_episode_id(&self) -> Option<&str> {
        self.episodes
            .get(self.episode_selected)
            .map(|e| e.episode_id.as_str())
    }

    pub fn categories(&self) -> &'static [Category] {
        match self.mode {
            Mode::Manga => MANGA_CATEGORIES,
            Mode::Anime => ANIME_CATEGORIES,
        }
    }

    pub fn selected_category(&self) -> &'static Category {
        let cats = self.categories();
        &cats[self.category_selected % cats.len()]
    }
}
