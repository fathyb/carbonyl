use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, InputMode};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Create main layout with tabs, URL bar, browser pane, and status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Length(3), // URL bar
            Constraint::Min(0),    // Browser pane
            Constraint::Length(1), // Status bar
        ])
        .split(f.size());

    // Draw tabs
    draw_tabs(f, app, chunks[0]);

    // Draw URL bar
    draw_url_bar(f, app, chunks[1]);

    // Draw browser pane (placeholder for now)
    draw_browser_pane(f, app, chunks[2]);

    // Draw status bar
    draw_status_bar(f, app, chunks[3]);

    // Draw command palette if open
    if app.command_palette_open {
        draw_command_palette(f, app);
    }
}

fn draw_tabs<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let title = if tab.title.len() > 20 {
                format!("{}...", &tab.title[..17])
            } else {
                tab.title.clone()
            };

            let style = if i == app.active_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(Span::styled(format!(" {} ", title), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.active_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn draw_url_bar<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let (text, style) = match app.input_mode {
        InputMode::Insert => (
            &app.url_input,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        _ => {
            if let Some(tab) = app.get_current_tab() {
                (&tab.url, Style::default().fg(Color::White))
            } else {
                ("", Style::default().fg(Color::White))
            }
        }
    };

    let input = Paragraph::new(text.as_str())
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("URL (Ctrl+L to edit)"),
        );

    f.render_widget(input, area);

    // Show cursor in insert mode
    if app.input_mode == InputMode::Insert {
        f.set_cursor(
            area.x + text.len() as u16 + 1,
            area.y + 1,
        );
    }
}

fn draw_browser_pane<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // Placeholder browser rendering
    // In the future, this will fetch the pre-rendered buffer from libmongoose
    // and display it directly

    let mode_indicator = if app.is_ascii_mode {
        "ASCII Art Mode"
    } else {
        "Block Mode (60fps)"
    };

    let placeholder_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "┌─────────────────────────────────────┐",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "│                                     │",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "│        mongoose Browser v0.0.3       │",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "│                                     │",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "│  Terminal-Native • Privacy-First    │",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "│  Hybrid Rendering • FOSS Stack      │",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "│                                     │",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            "└─────────────────────────────────────┘",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Current Mode: {}", mode_indicator),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("Keybindings:"),
        Line::from("  Ctrl+L  - Focus URL bar"),
        Line::from("  Ctrl+T  - New tab"),
        Line::from("  Ctrl+W  - Close tab"),
        Line::from("  Ctrl+A  - Toggle ASCII art mode"),
        Line::from("  Ctrl+Shift+P - Command palette"),
        Line::from("  Tab/Shift+Tab - Next/Previous tab"),
        Line::from("  q       - Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Browser engine integration: Coming soon",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(placeholder_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Browser Pane"),
        )
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_status_bar<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let status = Paragraph::new(app.status_message.as_str())
        .style(Style::default().fg(Color::White));

    f.render_widget(status, area);
}

fn draw_command_palette<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Create a centered popup
    let area = centered_rect(60, 40, f.size());

    let block = Block::default()
        .title("Command Palette")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into input and suggestions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    // Input field
    let input = Paragraph::new(app.command_input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Command"))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(input, chunks[0]);

    // Command suggestions
    let commands = vec![
        "quit - Exit mongoose",
        "new tab - Create a new tab",
        "close tab - Close current tab",
        "toggle ascii - Toggle ASCII art mode",
        "clear cache - Clear browser cache",
    ];

    let items: Vec<ListItem> = commands
        .iter()
        .map(|cmd| ListItem::new(*cmd))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available Commands"))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, chunks[1]);

    // Show cursor
    f.set_cursor(
        chunks[0].x + app.command_input.len() as u16 + 1,
        chunks[0].y + 1,
    );
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
