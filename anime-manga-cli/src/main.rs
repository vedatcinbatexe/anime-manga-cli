mod anime_api;
mod api;
mod app;
mod ui;

use anyhow::Result;
use app::{App, Mode, Screen};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = reqwest::Client::builder()
        .user_agent("anime-manga-cli/0.1")
        .build()?;

    let mut app = App::new();

    loop {
        app.tick += 1;
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.screen {
                    Screen::MainMenu => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Left | KeyCode::Right => {
                            app.mode = match app.mode {
                                Mode::Manga => Mode::Anime,
                                Mode::Anime => Mode::Manga,
                            };
                        }
                        KeyCode::Enter => {
                            app.screen = Screen::Search;
                            app.search_input.clear();
                            app.status = String::from("Type to search, Enter to confirm");
                        }
                        _ => {}
                    },
                    Screen::Search => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::MainMenu;
                            app.search_mode = app::SearchMode::Browse;
                            app.search_input.clear();
                        }
                        KeyCode::Tab => {
                            // Toggle between typing and browsing
                            app.search_mode = match app.search_mode {
                                app::SearchMode::Query => app::SearchMode::Browse,
                                app::SearchMode::Browse => app::SearchMode::Query,
                            };
                        }
                        KeyCode::Char(c) => {
                            // Any character press switches to query mode and types
                            app.search_mode = app::SearchMode::Query;
                            app.search_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_mode = app::SearchMode::Query;
                            app.search_input.pop();
                        }
                        KeyCode::Left => {
                            if matches!(app.search_mode, app::SearchMode::Browse) {
                                if app.category_selected > 0 {
                                    app.category_selected -= 1;
                                }
                            }
                        }
                        KeyCode::Right => {
                            if matches!(app.search_mode, app::SearchMode::Browse) {
                                if app.category_selected + 1 < app.categories().len() {
                                    app.category_selected += 1;
                                }
                            }
                        }
                        KeyCode::Up => {
                            if matches!(app.search_mode, app::SearchMode::Browse) {
                                if app.category_selected >= 4 {
                                    app.category_selected -= 4;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if matches!(app.search_mode, app::SearchMode::Browse) {
                                if app.category_selected + 4 < app.categories().len() {
                                    app.category_selected += 4;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            let query = match app.search_mode {
                                app::SearchMode::Query => app.search_input.clone(),
                                app::SearchMode::Browse => {
                                    app.selected_category().query.to_string()
                                }
                            };

                            if query.is_empty() {
                                return Ok(());
                            }

                            app.status = String::from("Searching...");
                            terminal.draw(|frame| ui::draw(frame, &app))?;

                            match app.mode {
                                Mode::Manga => match api::search_manga(&client, &query).await {
                                    Ok(results) => {
                                        app.results = results;
                                        app.selected = 0;
                                        app.screen = Screen::Results;
                                        app.status =
                                            String::from("↑↓ navigate  Enter select  Esc back");
                                    }
                                    Err(e) => app.status = format!("Error: {}", e),
                                },
                                Mode::Anime => {
                                    match anime_api::search_anime(&client, &query).await {
                                        Ok(results) => {
                                            app.anime_results = results;
                                            app.anime_selected = 0;
                                            app.screen = Screen::Results;
                                            app.status =
                                                String::from("↑↓ navigate  Enter select  Esc back");
                                        }
                                        Err(e) => app.status = format!("Error: {}", e),
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Screen::Results => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::Search;
                            app.status = String::from("Type to search, Enter to confirm");
                        }
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Enter => match app.mode {
                            Mode::Manga => {
                                if let Some(manga_id) = app.selected_manga_id() {
                                    let manga_id = manga_id.to_string();
                                    app.status = String::from("Loading chapters...");
                                    terminal.draw(|frame| ui::draw(frame, &app))?;

                                    match api::fetch_chapters(&client, &manga_id).await {
                                        Ok(chapters) => {
                                            app.chapters = chapters;
                                            app.chapter_selected = 0;
                                            app.screen = Screen::Chapters;
                                            app.status = String::from(
                                                "↑↓ to move, Enter to read, Esc to go back",
                                            );
                                        }
                                        Err(e) => app.status = format!("Error: {}", e),
                                    }
                                }
                            }
                            Mode::Anime => {
                                if let Some(anime) = app.selected_anime() {
                                    let anime = anime.clone();
                                    app.status = String::from("Loading episodes...");
                                    terminal.draw(|frame| ui::draw(frame, &app))?;

                                    match anime_api::fetch_episodes(&client, &anime).await {
                                        Ok(episodes) => {
                                            app.episodes = episodes;
                                            app.episode_selected = 0;
                                            app.screen = Screen::Episodes;
                                            app.status = String::from(
                                                "↑↓ to move, Enter to watch, Esc to go back",
                                            );
                                        }
                                        Err(e) => app.status = format!("Error: {}", e),
                                    }
                                }
                            }
                        },
                        _ => {}
                    },
                    Screen::Chapters => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::Results;
                            app.status =
                                String::from("↑↓ to move, Enter to select, Esc to go back");
                        }
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Enter => {
                            if let Some(chapter_id) = app.selected_chapter_id() {
                                let chapter_id = chapter_id.to_string();
                                app.status = String::from("Fetching page list...");
                                app.screen = Screen::Loading;
                                terminal.draw(|frame| ui::draw(frame, &app))?;

                                match api::fetch_pages(&client, &chapter_id).await {
                                    Ok(pages) => {
                                        app.total_pages = pages.len();
                                        app.pages = pages;
                                        app.current_page = 0;
                                        app.loaded_pages = 0;

                                        // Clean and create temp dir
                                        let _ = std::fs::remove_dir_all("/tmp/manga-pages");
                                        std::fs::create_dir_all("/tmp/manga-pages")?;

                                        // Download all pages with progress
                                        for (i, url) in app.pages.iter().enumerate() {
                                            app.status = format!(
                                                "Downloading page {}/{}...",
                                                i + 1,
                                                app.pages.len()
                                            );
                                            terminal.draw(|frame| ui::draw(frame, &app))?;

                                            match api::download_page_bytes(&client, url).await {
                                                Ok(bytes) => {
                                                    let path = format!(
                                                        "/tmp/manga-pages/{:03}.jpg",
                                                        i + 1
                                                    );
                                                    std::fs::write(&path, &bytes)?;
                                                    app.loaded_pages += 1;
                                                }
                                                Err(e) => {
                                                    app.status =
                                                        format!("Error on page {}: {}", i + 1, e);
                                                    terminal.draw(|frame| ui::draw(frame, &app))?;
                                                }
                                            }
                                        }

                                        // Launch feh with all pages
                                        app.status = String::from("Opening feh...");
                                        terminal.draw(|frame| ui::draw(frame, &app))?;

                                        std::process::Command::new("feh")
                                            .args([
                                                "--geometry",
                                                "1920x1080",
                                                "--sort",
                                                "name",
                                                "--image-bg",
                                                "black",
                                                "--title",
                                                "Manga Reader",
                                                "--zoom",
                                                "max",
                                                "--scale-down",
                                                "/tmp/manga-pages/",
                                            ])
                                            .status()?;

                                        // Back to chapters after feh closes
                                        app.screen = Screen::Chapters;
                                        app.status = String::from(
                                            "↑↓ to move, Enter to read, Esc to go back",
                                        );
                                    }
                                    Err(e) => {
                                        app.screen = Screen::Chapters;
                                        app.status = format!("Error: {}", e);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Screen::Episodes => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::Results;
                            app.status =
                                String::from("↑↓ to move, Enter to select, Esc to go back");
                        }
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Enter => {
                            if let Some(episode) = app.episodes.get(app.episode_selected) {
                                let ep_num = episode.episode_id.clone();
                                let show_id = app
                                    .anime_results
                                    .get(app.anime_selected)
                                    .map(|a| a.id.clone())
                                    .unwrap_or_default();

                                app.status =
                                    format!("Fetching qualities for episode {}...", ep_num);
                                terminal.draw(|frame| ui::draw(frame, &app))?;

                                match anime_api::fetch_links(&client, &show_id, &ep_num).await {
                                    Ok(links) => {
                                        if links.len() == 1 {
                                            // Only one quality — play directly
                                            let url = links[0].url.clone();
                                            std::process::Command::new("/mnt/c/mpv/mpv.exe")
                                                .args([
                                                    "--fullscreen",
                                                    "--title=Anime Player",
                                                    &url,
                                                ])
                                                .stdout(std::process::Stdio::null())
                                                .stderr(std::process::Stdio::null())
                                                .spawn()?;
                                            app.status = format!(
                                                "▶ Playing at {} — only quality available",
                                                links[0].quality
                                            );
                                        } else {
                                            app.stream_links = links;
                                            app.quality_selected = 0;
                                            app.screen = Screen::QualityPick;
                                            app.status = String::from(
                                                "↑↓ select quality  Enter watch  Esc back",
                                            );
                                        }
                                    }
                                    Err(e) => app.status = format!("Error: {}", e),
                                }
                            }
                        }
                        _ => {}
                    },
                    Screen::QualityPick => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::Episodes;
                            app.status = String::from("↑↓ navigate  Enter watch  Esc back");
                        }
                        KeyCode::Up => {
                            if app.quality_selected > 0 {
                                app.quality_selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.quality_selected + 1 < app.stream_links.len() {
                                app.quality_selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(link) = app.stream_links.get(app.quality_selected) {
                                let url = link.url.clone();
                                let quality = link.quality.clone();

                                std::process::Command::new("/mnt/c/mpv/mpv.exe")
                                    .args(["--fullscreen", "--title=Anime Player", &url])
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .spawn()?;

                                app.screen = Screen::Episodes;
                                app.status =
                                    format!("▶ Playing at {} — select another episode", quality);
                            }
                        }
                        _ => {}
                    },
                    Screen::Reader => {}  // feh handles everything now
                    Screen::Loading => {} // just displays progress, no key handling
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
