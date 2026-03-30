use crate::app::{App, Mode, Screen};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

// ── Palette ───────────────────────────────────────────────────────────────────
const VOID: Color = Color::Rgb(2, 4, 16);
const DEEP: Color = Color::Rgb(5, 8, 30);
const BUILD1: Color = Color::Rgb(8, 12, 40);
const BUILD2: Color = Color::Rgb(12, 18, 55);
const BUILD3: Color = Color::Rgb(18, 25, 70);
const NEON_BLUE: Color = Color::Rgb(0, 180, 255);
const NEON_CYAN: Color = Color::Rgb(0, 255, 220);
const NEON_PINK: Color = Color::Rgb(255, 0, 150);
const NEON_PURP: Color = Color::Rgb(180, 0, 255);
const NEON_GREEN: Color = Color::Rgb(0, 255, 100);
const NEON_YELL: Color = Color::Rgb(255, 220, 0);
const WIN_ON: Color = Color::Rgb(255, 200, 80);
const WIN_BLUE: Color = Color::Rgb(80, 180, 255);
const WIN_PINK: Color = Color::Rgb(255, 80, 180);
const RAIN: Color = Color::Rgb(40, 100, 180);
const RAIN_DIM: Color = Color::Rgb(20, 50, 90);
const STAR_LO: Color = Color::Rgb(100, 120, 200);
const STAR_HI: Color = Color::Rgb(220, 230, 255);
const MOON_C: Color = Color::Rgb(240, 240, 200);
const DIM: Color = Color::Rgb(60, 70, 120);
const TEXT: Color = Color::Rgb(200, 210, 255);
const ACCENT: Color = Color::Rgb(0, 210, 255);

fn s(fg: Color, bg: Color) -> Style {
    Style::default().fg(fg).bg(bg)
}
fn fg(fg: Color) -> Style {
    Style::default().fg(fg)
}

// ── Animated helpers ──────────────────────────────────────────────────────────

fn star(tick: u64, phase: u64) -> (&'static str, Color) {
    match (tick / 8 + phase) % 4 {
        0 => ("✦", STAR_HI),
        1 => ("·", STAR_LO),
        2 => ("*", STAR_HI),
        _ => ("˙", STAR_LO),
    }
}

fn rain_char(tick: u64, col: u64) -> (&'static str, Color) {
    let phase = (tick + col * 7) % 20;
    match phase {
        0 => ("|", NEON_BLUE),
        1 => ("|", RAIN),
        2 => ("╎", RAIN),
        3 => ("╎", RAIN_DIM),
        4 => ("·", RAIN_DIM),
        _ => (" ", VOID),
    }
}

fn glitch_title(tick: u64) -> Vec<Span<'static>> {
    let glitch = (tick / 3) % 20 == 0;
    let lines = if glitch {
        vec![
            ("█▄─█ ", NEON_PINK),
            ("▄▀█ ", NEON_PURP),
            ("█▄─█ ", NEON_PINK),
            ("▄▀█ ", NEON_BLUE),
        ]
    } else {
        vec![
            ("ANIME", NEON_BLUE),
            (" ✦ ", NEON_CYAN),
            ("MANGA", NEON_PINK),
            (" CLI", NEON_PURP),
        ]
    };
    lines
        .into_iter()
        .map(|(text, color)| {
            Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )
        })
        .collect()
}

fn pulse_border(tick: u64) -> Color {
    match (tick / 5) % 4 {
        0 => NEON_BLUE,
        1 => NEON_CYAN,
        2 => NEON_PURP,
        _ => NEON_PINK,
    }
}

fn spinner_fancy(tick: u64) -> &'static str {
    let frames = ["◐", "◓", "◑", "◒"];
    frames[(tick as usize / 2) % frames.len()]
}

// ── Bordered block ────────────────────────────────────────────────────────────
fn bordered<'a>(title: &'a str, color: Color) -> Block<'a> {
    Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(fg(color))
        .style(Style::default().bg(DEEP))
}

// ── City background ───────────────────────────────────────────────────────────
fn draw_city(frame: &mut Frame, area: Rect, tick: u64) {
    let bg = Block::default().style(Style::default().bg(VOID));
    frame.render_widget(bg, area);

    let w = area.width as u64;

    // Build lines
    let mut lines: Vec<Line> = Vec::new();

    // ── Sky rows with stars and rain ─────────────────────────────────────────
    for row in 0..6u64 {
        let mut spans = vec![];
        for col in 0..w {
            let (rc, rcol) = rain_char(tick + row * 3, col);
            if rc != " " {
                spans.push(Span::styled(rc, s(rcol, VOID)));
            } else {
                let (sc, scol) = star(tick, col * 3 + row * 7);
                if col % 5 == 0 {
                    spans.push(Span::styled(sc, s(scol, VOID)));
                } else {
                    spans.push(Span::styled(" ", s(VOID, VOID)));
                }
            }
        }
        lines.push(Line::from(spans));
    }

    // ── Moon ─────────────────────────────────────────────────────────────────
    let moon_phase = match (tick / 20) % 4 {
        0 => "◐",
        1 => "○",
        2 => "◑",
        _ => "●",
    };
    let mut moon_line = vec![];
    for col in 0..w {
        let (rc, rcol) = rain_char(tick + 6, col);
        if rc != " " {
            moon_line.push(Span::styled(rc, s(rcol, VOID)));
        } else if col == w / 5 {
            moon_line.push(Span::styled(moon_phase, s(MOON_C, VOID)));
        } else if col % 7 == 0 {
            let (sc, scol) = star(tick, col * 11);
            moon_line.push(Span::styled(sc, s(scol, VOID)));
        } else {
            moon_line.push(Span::styled(" ", s(VOID, VOID)));
        }
    }
    lines.push(Line::from(moon_line));

    // ── Distant skyline ───────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("          ", s(VOID, VOID)),
        Span::styled("▄", s(BUILD1, VOID)),
        Span::styled("        ", s(VOID, VOID)),
        Span::styled("▄██▄", s(BUILD1, VOID)),
        Span::styled("       ", s(VOID, VOID)),
        Span::styled("▄█▄", s(BUILD1, VOID)),
        Span::styled("   ", s(VOID, VOID)),
        Span::styled("▄", s(BUILD1, VOID)),
        Span::styled("          ", s(VOID, VOID)),
        Span::styled("▄████▄", s(BUILD1, VOID)),
        Span::styled("      ", s(VOID, VOID)),
        Span::styled("▄▄██▄▄", s(BUILD1, VOID)),
        Span::styled("    ", s(VOID, VOID)),
        Span::styled("▄█▄", s(BUILD1, VOID)),
        Span::styled("    ", s(VOID, VOID)),
        Span::styled("▄███▄", s(BUILD1, VOID)),
    ]));

    // ── Mid skyline with neon signs ───────────────────────────────────────────
    let neon_a = if (tick / 8) % 2 == 0 {
        NEON_YELL
    } else {
        WIN_ON
    };
    let neon_b = if (tick / 6) % 2 == 0 {
        WIN_PINK
    } else {
        NEON_PINK
    };
    let neon_c = if (tick / 10) % 2 == 0 {
        WIN_BLUE
    } else {
        NEON_BLUE
    };

    lines.push(Line::from(vec![
        Span::styled("     ", s(VOID, VOID)),
        Span::styled("▄███▄", s(BUILD2, VOID)),
        Span::styled("   ", s(VOID, VOID)),
        Span::styled("██████", s(BUILD2, VOID)),
        Span::styled("  ", s(VOID, VOID)),
        Span::styled("█████", s(BUILD2, VOID)),
        Span::styled("  ", s(VOID, VOID)),
        Span::styled("███", s(BUILD2, VOID)),
        Span::styled("  ", s(VOID, VOID)),
        Span::styled("██████████", s(BUILD2, VOID)),
        Span::styled("   ", s(VOID, VOID)),
        Span::styled("████████", s(BUILD2, VOID)),
        Span::styled("  ", s(VOID, VOID)),
        Span::styled("█████", s(BUILD2, VOID)),
        Span::styled("  ", s(VOID, VOID)),
        Span::styled("███████", s(BUILD2, VOID)),
    ]));

    // ── Near building tops ────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("▄████▄", s(BUILD3, VOID)),
        Span::styled(" ", s(VOID, VOID)),
        Span::styled("▄", s(neon_a, VOID)),
        Span::styled("▄██████▄", s(BUILD3, VOID)),
        Span::styled(" ", s(VOID, VOID)),
        Span::styled("▄████████████▄", s(BUILD3, VOID)),
        Span::styled(" ", s(VOID, VOID)),
        Span::styled("▄▄", s(BUILD3, VOID)),
        Span::styled(" ", s(VOID, VOID)),
        Span::styled("▄", s(neon_b, VOID)),
        Span::styled("▄█████████▄", s(BUILD3, VOID)),
        Span::styled(" ", s(VOID, VOID)),
        Span::styled("▄", s(neon_c, VOID)),
        Span::styled("███████████████", s(BUILD3, VOID)),
    ]));

    // ── Near buildings row 1 — windows ────────────────────────────────────────
    let w1 = if (tick / 4 + 1) % 3 == 0 {
        VOID
    } else {
        WIN_ON
    };
    let w2 = if (tick / 5 + 2) % 3 == 0 {
        VOID
    } else {
        WIN_BLUE
    };
    let w3 = if (tick / 7 + 3) % 4 == 0 {
        VOID
    } else {
        WIN_PINK
    };
    let w4 = if (tick / 3 + 5) % 5 == 0 {
        VOID
    } else {
        WIN_ON
    };

    lines.push(Line::from(vec![
        Span::styled("██", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w1, BUILD3)),
        Span::styled("██", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w2, BUILD3)),
        Span::styled("████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w3, BUILD3)),
        Span::styled("█████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w1, BUILD3)),
        Span::styled("████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w4, BUILD3)),
        Span::styled("████████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w2, BUILD3)),
        Span::styled("██", s(BUILD3, BUILD3)),
        Span::styled("▪", s(w3, BUILD3)),
        Span::styled(
            "██████████████████████████████████████████████",
            s(BUILD3, BUILD3),
        ),
    ]));

    // ── Near buildings row 2 — windows ────────────────────────────────────────
    let x1 = if (tick / 6 + 7) % 3 == 0 {
        VOID
    } else {
        WIN_BLUE
    };
    let x2 = if (tick / 4 + 2) % 4 == 0 {
        VOID
    } else {
        WIN_ON
    };
    let x3 = if (tick / 5 + 9) % 3 == 0 {
        VOID
    } else {
        WIN_PINK
    };

    lines.push(Line::from(vec![
        Span::styled("██", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x2, BUILD3)),
        Span::styled("████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x1, BUILD3)),
        Span::styled("████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x3, BUILD3)),
        Span::styled("██████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x2, BUILD3)),
        Span::styled("████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x1, BUILD3)),
        Span::styled("████████████", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x3, BUILD3)),
        Span::styled("██", s(BUILD3, BUILD3)),
        Span::styled("▪", s(x2, BUILD3)),
        Span::styled(
            "██████████████████████████████████████████████",
            s(BUILD3, BUILD3),
        ),
    ]));

    // ── Ground ────────────────────────────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled(
            "████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████",
            s(BUILD3, BUILD3),
        ),
    ]));

    // Neon ground reflection
    let ref_color = match (tick / 5) % 3 {
        0 => NEON_BLUE,
        1 => NEON_CYAN,
        _ => NEON_PURP,
    };
    lines.push(Line::from(vec![
        Span::styled(
            "▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔",
            s(ref_color, VOID),
        ),
    ]));

    let paragraph = Paragraph::new(lines).style(Style::default().bg(VOID));
    frame.render_widget(paragraph, area);
}

// ── Router ────────────────────────────────────────────────────────────────────
pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::MainMenu => draw_main_menu(frame, app),
        Screen::Search => draw_search(frame, app),
        Screen::Results => match app.mode {
            Mode::Manga => draw_results(frame, app),
            Mode::Anime => draw_anime_results(frame, app),
        },
        Screen::Chapters => draw_chapters(frame, app),
        Screen::Episodes => draw_episodes(frame, app),
        Screen::Loading => draw_loading(frame, app),
        Screen::QualityPick => draw_quality_pick(frame, app),
        Screen::Reader => {}
    }
}

// ── Main menu ─────────────────────────────────────────────────────────────────
fn draw_quality_pick(frame: &mut Frame, app: &App) {
    let area = frame.area();
    draw_city(frame, area, app.tick);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Percentage(25),
            Constraint::Length(1),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "  select quality  ",
        Style::default().fg(NEON_PINK).add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(fg(pulse_border(app.tick)))
            .style(Style::default().bg(Color::Rgb(3, 5, 20))),
    );
    frame.render_widget(title, center_rect(40, chunks[1]));

    // Quality list
    let quality_icons = |q: &str| match q {
        "1080p" => "🔵 ",
        "720p" => "🟢 ",
        "480p" => "🟡 ",
        "360p" => "🔴 ",
        _ => "⚪ ",
    };

    let quality_desc = |q: &str| match q {
        "1080p" => "Full HD  — best quality",
        "720p" => "HD       — recommended",
        "480p" => "SD       — saves data",
        "360p" => "Low      — fast loading",
        _ => "Auto     — adaptive",
    };

    let items: Vec<ListItem> = app
        .stream_links
        .iter()
        .enumerate()
        .map(|(i, link)| {
            let is_sel = i == app.quality_selected;
            let icon = quality_icons(&link.quality);
            let desc = quality_desc(&link.quality);
            let color = match link.quality.as_str() {
                "1080p" => NEON_BLUE,
                "720p" => NEON_GREEN,
                "480p" => NEON_YELL,
                "360p" => NEON_PINK,
                _ => NEON_CYAN,
            };

            Line::from(vec![
                Span::styled(if is_sel { " ▶  " } else { "    " }, fg(color)),
                Span::styled(
                    format!("{}{:<8}", icon, link.quality),
                    Style::default()
                        .fg(if is_sel { VOID } else { color })
                        .bg(if is_sel { color } else { Color::Reset })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", desc),
                    Style::default()
                        .fg(if is_sel { VOID } else { TEXT })
                        .bg(if is_sel { color } else { Color::Reset }),
                ),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(bordered("↑↓ navigate  Enter play  Esc back", NEON_PINK));

    let mut state = ListState::default();
    state.select(Some(app.quality_selected));
    frame.render_stateful_widget(list, center_rect(60, chunks[2]), &mut state);

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(DIM).bg(VOID))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[4]);
}

fn draw_main_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();
    draw_city(frame, area, app.tick);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(18),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Percentage(5),
            Constraint::Length(14),
            Constraint::Percentage(5),
            Constraint::Length(1),
        ])
        .split(area);

    // Glitch title
    let center = center_rect(70, chunks[1]);
    let title_line = glitch_title(app.tick);
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(title_line),
        Line::from(vec![Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            fg(pulse_border(app.tick)),
        )]),
        Line::from(vec![Span::styled(
            "  terminal media player  //  anime & manga  ",
            fg(DIM),
        )]),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().bg(Color::Rgb(2, 4, 20)))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(fg(pulse_border(app.tick)))
            .style(Style::default().bg(Color::Rgb(2, 4, 20))),
    );
    frame.render_widget(title, center);

    // Subtitle with animated dots
    let dots = match (app.tick / 8) % 4 {
        0 => "   ",
        1 => ".  ",
        2 => ".. ",
        _ => "...",
    };
    let subtitle = Paragraph::new(format!("[ select mode{} ]", dots))
        .alignment(Alignment::Center)
        .style(Style::default().fg(DIM).bg(VOID));
    frame.render_widget(subtitle, chunks[3]);

    // Cards
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(8),
            Constraint::Percentage(38),
            Constraint::Percentage(8),
            Constraint::Percentage(38),
            Constraint::Percentage(8),
        ])
        .split(chunks[5]);

    draw_mode_card(frame, app, cards[1], false);
    draw_mode_card(frame, app, cards[3], true);

    // Bottom hint with animation
    let hint_color = if (app.tick / 10) % 2 == 0 {
        DIM
    } else {
        NEON_BLUE
    };
    let hint = Paragraph::new("◄ ► select    ↵ confirm    q quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(hint_color).bg(VOID));
    frame.render_widget(hint, chunks[7]);
}

fn draw_mode_card(frame: &mut Frame, app: &App, area: Rect, is_manga: bool) {
    let selected = match (&app.mode, is_manga) {
        (Mode::Manga, true) | (Mode::Anime, false) => true,
        _ => false,
    };

    let (icon, label, color, desc1, desc2) = if is_manga {
        (
            "📖",
            "M A N G A",
            NEON_CYAN,
            "search & read manga",
            "powered by MangaDex",
        )
    } else {
        (
            "📺",
            "A N I M E",
            NEON_PINK,
            "search & stream anime",
            "powered by AllAnime",
        )
    };

    let border_color = if selected { color } else { DIM };
    let border_type = if selected {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    // Animated selection indicator
    let selector = if selected {
        match (app.tick / 4) % 4 {
            0 => "▶  ",
            1 => " ▶ ",
            2 => "  ▶",
            _ => " ▶ ",
        }
    } else {
        "   "
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", icon),
            Style::default().fg(color),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}{}", selector, label),
            Style::default()
                .fg(if selected { color } else { TEXT })
                .add_modifier(if selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", desc1),
            Style::default().fg(DIM),
        )),
        Line::from(Span::styled(
            format!("  {}", desc2),
            Style::default().fg(DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            if selected {
                "  ◆ ENTER TO SELECT ◆"
            } else {
                "  ◇ navigate with ◄ ►"
            },
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        // Mini scanline effect
        Line::from(Span::styled(
            "  ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁",
            Style::default().fg(if selected {
                color
            } else {
                Color::Rgb(20, 20, 40)
            }),
        )),
    ];

    let card = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(fg(border_color))
            .style(Style::default().bg(if selected {
                Color::Rgb(5, 10, 35)
            } else {
                Color::Rgb(3, 5, 18)
            })),
    );
    frame.render_widget(card, area);
}

// ── Search screen ─────────────────────────────────────────────────────────────
fn draw_search(frame: &mut Frame, app: &App) {
    let area = frame.area();
    draw_city(frame, area, app.tick);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let (mode_label, mode_color) = match app.mode {
        Mode::Manga => ("MANGA", NEON_CYAN),
        Mode::Anime => ("ANIME", NEON_PINK),
    };

    // Title bar
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  ╔══ ", fg(mode_color)),
        Span::styled(
            mode_label,
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ══╗  ", fg(mode_color)),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(VOID));
    frame.render_widget(title, center_rect(60, chunks[1]));

    // Search input
    let is_query_mode = matches!(app.search_mode, crate::app::SearchMode::Query);
    let cursor = if is_query_mode && app.tick % 8 < 4 {
        "█"
    } else {
        " "
    };
    let input_text = format!("  ❯ {}{}", app.search_input, cursor);
    let input_color = if is_query_mode {
        pulse_border(app.tick)
    } else {
        DIM
    };

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(TEXT).bg(Color::Rgb(4, 8, 28)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(fg(input_color))
                .title(Span::styled(
                    if is_query_mode {
                        " 🔍 type to search — Enter confirm — Tab browse categories "
                    } else {
                        " 🔍 search — press Tab to type — Enter on category "
                    },
                    Style::default()
                        .fg(input_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(Color::Rgb(4, 8, 28))),
        );
    frame.render_widget(input, center_rect(72, chunks[2]));

    // Section label
    let section_label = Paragraph::new(Line::from(vec![
        Span::styled("  ── ", fg(DIM)),
        Span::styled(
            "Browse Categories",
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ──────────────────────────────────────────────────────",
            fg(DIM),
        ),
    ]))
    .style(Style::default().bg(VOID));
    frame.render_widget(section_label, chunks[3]);

    // Category grid — 4 per row
    let cats = app.categories();
    let cat_area = chunks[4];
    let cols = 4usize;
    let rows = (cats.len() + cols - 1) / cols;
    let row_height = (cat_area.height as usize / rows).max(1) as u16;

    for (i, cat) in cats.iter().enumerate() {
        let row = i / cols;
        let col = i % cols;
        let is_sel = i == app.category_selected && !is_query_mode;

        let cell = Rect {
            x: cat_area.x + (col as u16 * (cat_area.width / cols as u16)),
            y: cat_area.y + (row as u16 * row_height),
            width: cat_area.width / cols as u16,
            height: row_height.min(cat_area.height.saturating_sub(row as u16 * row_height)),
        };

        let border_color = if is_sel {
            mode_color
        } else {
            Color::Rgb(30, 40, 80)
        };
        let text_color = if is_sel { mode_color } else { DIM };

        let content = Paragraph::new(vec![Line::from(Span::styled(
            format!(" {} {}", cat.icon, cat.label),
            Style::default()
                .fg(text_color)
                .bg(if is_sel {
                    Color::Rgb(5, 10, 35)
                } else {
                    Color::Rgb(3, 5, 18)
                })
                .add_modifier(if is_sel {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(if is_sel {
                    BorderType::Double
                } else {
                    BorderType::Plain
                })
                .border_style(fg(border_color))
                .style(Style::default().bg(if is_sel {
                    Color::Rgb(5, 10, 35)
                } else {
                    Color::Rgb(3, 5, 18)
                })),
        );
        frame.render_widget(content, cell);
    }

    // Status bar
    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(DIM).bg(VOID))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[5]);
}

// ── Results ───────────────────────────────────────────────────────────────────
fn draw_results(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(chunks[0]);

    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, manga)| {
            let is_sel = i == app.selected;
            let status = manga.attributes.status.as_deref().unwrap_or("?");
            let sc = match status {
                "completed" => NEON_GREEN,
                "ongoing" => NEON_YELL,
                _ => NEON_PINK,
            };
            let icon = match status {
                "completed" => "✓",
                "ongoing" => "●",
                _ => "⏸",
            };

            Line::from(vec![
                Span::styled(if is_sel { " ▶ " } else { "   " }, fg(NEON_CYAN)),
                Span::styled(
                    format!("{:<35}", manga.title()),
                    if is_sel {
                        Style::default()
                            .fg(VOID)
                            .bg(NEON_CYAN)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(TEXT)
                    },
                ),
                Span::styled(format!(" {}{} ", icon, status), fg(sc)),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(bordered(
        "Results  ↑↓ navigate  Enter select  Esc back",
        NEON_CYAN,
    ));
    let mut state = ListState::default();
    state.select(Some(app.selected));
    frame.render_stateful_widget(list, panels[0], &mut state);

    draw_manga_detail(frame, app, panels[1]);

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(DIM).bg(DEEP))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[1]);
}

fn draw_manga_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(manga) = app.results.get(app.selected) else {
        frame.render_widget(
            Paragraph::new("\n  ◇ select a manga to preview")
                .style(Style::default().fg(DIM))
                .block(bordered("Preview", DIM)),
            area,
        );
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let status_str = manga.attributes.status.as_deref().unwrap_or("unknown");
    let (sc, si) = match status_str {
        "completed" => (NEON_GREEN, "✓ completed"),
        "ongoing" => (NEON_YELL, "● ongoing"),
        "hiatus" => (NEON_PINK, "⏸ hiatus"),
        _ => (DIM, "? unknown"),
    };

    frame.render_widget(
        Paragraph::new(format!("  {}", manga.title()))
            .style(Style::default().fg(NEON_CYAN).add_modifier(Modifier::BOLD))
            .block(bordered("Title", NEON_CYAN))
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new(format!("  {}", si))
            .style(Style::default().fg(sc).add_modifier(Modifier::BOLD))
            .block(bordered("Status", sc)),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new(format!("\n  {}", manga.description()))
            .style(Style::default().fg(TEXT))
            .block(bordered("Synopsis", NEON_PURP))
            .wrap(Wrap { trim: true }),
        chunks[2],
    );
}

// ── Anime results ─────────────────────────────────────────────────────────────
fn draw_anime_results(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(chunks[0]);

    let items: Vec<ListItem> = app
        .anime_results
        .iter()
        .enumerate()
        .map(|(i, anime)| {
            let is_sel = i == app.anime_selected;
            Line::from(vec![
                Span::styled(if is_sel { " ▶ " } else { "   " }, fg(NEON_PINK)),
                Span::styled(
                    format!("{:<38}", anime.title()),
                    if is_sel {
                        Style::default()
                            .fg(VOID)
                            .bg(NEON_PINK)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(TEXT)
                    },
                ),
                Span::styled(format!(" {} ep", anime.episodes), fg(DIM)),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(bordered(
        "Anime Results  ↑↓ navigate  Enter select  Esc back",
        NEON_PINK,
    ));
    let mut state = ListState::default();
    state.select(Some(app.anime_selected));
    frame.render_stateful_widget(list, panels[0], &mut state);

    // Detail panel
    if let Some(anime) = app.anime_results.get(app.anime_selected) {
        let dchunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(panels[1]);

        frame.render_widget(
            Paragraph::new(format!("  {}", anime.title()))
                .style(Style::default().fg(NEON_PINK).add_modifier(Modifier::BOLD))
                .block(bordered("Title", NEON_PINK))
                .wrap(Wrap { trim: true }),
            dchunks[0],
        );

        frame.render_widget(
            Paragraph::new(format!("\n  {} episodes available", anime.episodes))
                .style(Style::default().fg(NEON_YELL))
                .block(bordered("Info", NEON_PURP)),
            dchunks[1],
        );
    }

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(DIM).bg(DEEP))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[1]);
}

// ── Chapters ──────────────────────────────────────────────────────────────────
fn draw_chapters(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let items: Vec<ListItem> = app
        .chapters
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let is_sel = i == app.chapter_selected;
            let num = ch.attributes.chapter.as_deref().unwrap_or("?");
            let title = ch.attributes.title.as_deref().unwrap_or("Untitled");
            let pages = ch.attributes.pages.unwrap_or(0);

            Line::from(vec![
                Span::styled(if is_sel { " ▶ " } else { "   " }, fg(NEON_CYAN)),
                Span::styled(
                    format!("Ch.{:<5}", num),
                    Style::default()
                        .fg(if is_sel { VOID } else { NEON_CYAN })
                        .bg(if is_sel { NEON_CYAN } else { Color::Reset })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {:<45}", title),
                    Style::default()
                        .fg(if is_sel { VOID } else { TEXT })
                        .bg(if is_sel { NEON_CYAN } else { Color::Reset }),
                ),
                Span::styled(
                    format!(" {}pg", pages),
                    Style::default()
                        .fg(if is_sel { VOID } else { DIM })
                        .bg(if is_sel { NEON_CYAN } else { Color::Reset }),
                ),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(bordered(
        "Chapters  ↑↓ navigate  Enter read  Esc back",
        NEON_CYAN,
    ));
    let mut state = ListState::default();
    state.select(Some(app.chapter_selected));
    frame.render_stateful_widget(list, chunks[0], &mut state);

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(DIM).bg(DEEP))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[1]);
}

// ── Episodes ──────────────────────────────────────────────────────────────────
fn draw_episodes(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .enumerate()
        .map(|(i, ep)| {
            let is_sel = i == app.episode_selected;
            let filler = ep.filler.unwrap_or(false);

            Line::from(vec![
                Span::styled(if is_sel { " ▶ " } else { "   " }, fg(NEON_PINK)),
                Span::styled(
                    format!("{:<55}", ep.label()),
                    if is_sel {
                        Style::default()
                            .fg(VOID)
                            .bg(NEON_PINK)
                            .add_modifier(Modifier::BOLD)
                    } else if filler {
                        Style::default().fg(DIM)
                    } else {
                        Style::default().fg(TEXT)
                    },
                ),
            ])
            .into()
        })
        .collect();

    let list = List::new(items).block(bordered(
        "Episodes  ↑↓ navigate  Enter watch  Esc back",
        NEON_PINK,
    ));
    let mut state = ListState::default();
    state.select(Some(app.episode_selected));
    frame.render_stateful_widget(list, chunks[0], &mut state);

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(DIM).bg(DEEP))
        .alignment(Alignment::Center);
    frame.render_widget(status, chunks[1]);
}

// ── Loading ───────────────────────────────────────────────────────────────────
fn draw_loading(frame: &mut Frame, app: &App) {
    let area = frame.area();
    draw_city(frame, area, app.tick);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Percentage(38),
        ])
        .split(area);

    let center = center_rect(60, chunks[1]);

    // Spinner with animated border
    let spin_color = pulse_border(app.tick);
    let spinner_text = format!(
        "  {}  {}  {} ",
        spinner_fancy(app.tick),
        app.status,
        spinner_fancy(app.tick + 2),
    );
    let spinner = Paragraph::new(spinner_text)
        .style(
            Style::default()
                .fg(spin_color)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(3, 6, 22)),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(fg(spin_color))
                .style(Style::default().bg(Color::Rgb(3, 6, 22))),
        );
    frame.render_widget(spinner, center);

    // Progress bar
    let progress = if app.total_pages > 0 {
        app.loaded_pages as f64 / app.total_pages as f64
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(bordered("Downloading Pages", NEON_CYAN))
        .gauge_style(
            Style::default()
                .fg(NEON_CYAN)
                .bg(Color::Rgb(10, 20, 50))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(progress)
        .label(format!("{} / {} pages", app.loaded_pages, app.total_pages));
    frame.render_widget(gauge, center_rect(60, chunks[3]));

    // Data stream effect
    let stream: String = (0..50)
        .map(|i| {
            let chars = "01アイウエオカキクケコ";
            let idx = ((app.tick + i * 7) % chars.chars().count() as u64) as usize;
            chars.chars().nth(idx).unwrap_or('0')
        })
        .collect();

    let data_stream = Paragraph::new(stream.as_str())
        .style(Style::default().fg(Color::Rgb(0, 60, 30)).bg(VOID))
        .alignment(Alignment::Center);
    frame.render_widget(data_stream, center_rect(60, chunks[4]));
}

// ── Helpers ───────────────────────────────────────────────────────────────────
fn center_rect(percent_x: u16, area: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(area);
    layout[1]
}
