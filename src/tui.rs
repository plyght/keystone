use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use crate::audit::AuditLogger;
use crate::daemon;
use crate::pool::{self, KeyPool};

struct DashboardState {
    last_update: Instant,
    daemon_status: Option<daemon::DaemonStatus>,
    recent_audits: Vec<crate::audit::AuditEntry>,
    pools: Vec<PoolInfo>,
    metrics: DashboardMetrics,
    scroll_offset: usize,
}

struct PoolInfo {
    name: String,
    total_keys: usize,
    available_keys: usize,
    active_keys: usize,
}

struct DashboardMetrics {
    total_rotations: usize,
    successful_rotations: usize,
    failed_rotations: usize,
}

impl DashboardState {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            daemon_status: None,
            recent_audits: Vec::new(),
            pools: Vec::new(),
            metrics: DashboardMetrics {
                total_rotations: 0,
                successful_rotations: 0,
                failed_rotations: 0,
            },
            scroll_offset: 0,
        }
    }

    fn refresh(&mut self) -> Result<()> {
        self.daemon_status = daemon::get_daemon_status().ok();

        let audit_logger = AuditLogger::new()?;
        self.recent_audits = audit_logger.read_logs(None, None, Some(50))?;

        self.metrics.total_rotations = self.recent_audits.len();
        self.metrics.successful_rotations = self.recent_audits.iter().filter(|e| e.success).count();
        self.metrics.failed_rotations = self.recent_audits.iter().filter(|e| !e.success).count();

        let pool_names = pool::list_all_pools()?;
        self.pools.clear();
        for name in pool_names {
            if let Ok(Some(pool)) = KeyPool::load(&name) {
                self.pools.push(PoolInfo {
                    name: name.clone(),
                    total_keys: pool.keys.len(),
                    available_keys: pool.count_available(),
                    active_keys: pool.count_active(),
                });
            }
        }

        self.last_update = Instant::now();
        Ok(())
    }
}

pub async fn run_dashboard() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = DashboardState::new();
    state.refresh()?;

    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut DashboardState,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => {
                        state.refresh()?;
                    }
                    KeyCode::Down => {
                        if state.scroll_offset < state.recent_audits.len().saturating_sub(1) {
                            state.scroll_offset += 1;
                        }
                    }
                    KeyCode::Up => {
                        state.scroll_offset = state.scroll_offset.saturating_sub(1);
                    }
                    KeyCode::PageDown => {
                        state.scroll_offset = (state.scroll_offset + 10)
                            .min(state.recent_audits.len().saturating_sub(1));
                    }
                    KeyCode::PageUp => {
                        state.scroll_offset = state.scroll_offset.saturating_sub(10);
                    }
                    _ => {}
                }
            }
        }

        if state.last_update.elapsed() > Duration::from_secs(5) {
            state.refresh()?;
        }
    }
}

fn ui(f: &mut Frame, state: &DashboardState) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(size);

    render_header(f, chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(body_chunks[0]);

    render_daemon_status(f, left_chunks[0], state);
    render_metrics(f, left_chunks[1], state);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(body_chunks[1]);

    render_audit_logs(f, right_chunks[0], state);
    render_pools(f, right_chunks[1], state);

    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "BIRCH",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("Dashboard", Style::default().fg(Color::Gray)),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(Color::White));

    f.render_widget(title, area);
}

fn render_daemon_status(f: &mut Frame, area: Rect, state: &DashboardState) {
    let status_text = if let Some(ref status) = state.daemon_status {
        if status.running {
            vec![
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled("Running", Style::default().fg(Color::Green)),
                ]),
                Line::from(format!("PID:    {}", status.pid.unwrap_or(0))),
                Line::from(format!("Bind:   {}", status.bind_address)),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled("Stopped", Style::default().fg(Color::Red)),
                ]),
                Line::from(format!("Bind:   {}", status.bind_address)),
            ]
        }
    } else {
        vec![Line::from(vec![
            Span::raw("Status: "),
            Span::styled("Unknown", Style::default().fg(Color::Yellow)),
        ])]
    };

    let paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Daemon Status"),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn render_metrics(f: &mut Frame, area: Rect, state: &DashboardState) {
    let success_rate = if state.metrics.total_rotations > 0 {
        (state.metrics.successful_rotations as f64 / state.metrics.total_rotations as f64) * 100.0
    } else {
        0.0
    };

    let metrics_text = vec![
        Line::from(format!(
            "Total Rotations: {}",
            state.metrics.total_rotations
        )),
        Line::from(vec![
            Span::raw("Success: "),
            Span::styled(
                format!("{}", state.metrics.successful_rotations),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("Failed:  "),
            Span::styled(
                format!("{}", state.metrics.failed_rotations),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(format!("Success Rate: {:.1}%", success_rate)),
    ];

    let paragraph = Paragraph::new(metrics_text)
        .block(Block::default().borders(Borders::ALL).title("Metrics"))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn render_audit_logs(f: &mut Frame, area: Rect, state: &DashboardState) {
    let items: Vec<ListItem> = state
        .recent_audits
        .iter()
        .skip(state.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|entry| {
            let (status_text, status_color) = if entry.success {
                ("[OK]", Color::Green)
            } else {
                ("[FAIL]", Color::Red)
            };

            let action_str = format!("{:?}", entry.action);
            let time_str = entry.timestamp.format("%m-%d %H:%M:%S");

            let content = vec![Line::from(vec![
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(time_str.to_string(), Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(action_str, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(&entry.secret_name, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", &entry.env),
                    Style::default().fg(Color::Yellow),
                ),
            ])];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Audit Logs"),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn render_pools(f: &mut Frame, area: Rect, state: &DashboardState) {
    if state.pools.is_empty() {
        let paragraph = Paragraph::new("No pools configured")
            .block(Block::default().borders(Borders::ALL).title("Key Pools"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = state
        .pools
        .iter()
        .map(|pool| {
            let (status_text, status_color) = if pool.available_keys == 0 {
                ("[EMPTY]", Color::Red)
            } else if pool.available_keys <= 2 {
                ("[LOW]", Color::Yellow)
            } else {
                ("[OK]", Color::Green)
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(&pool.name, Style::default().fg(Color::White)),
                Span::raw(" - "),
                Span::styled(
                    format!("{}", pool.total_keys),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" total, "),
                Span::styled(
                    format!("{}", pool.available_keys),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" available, "),
                Span::styled(
                    format!("{}", pool.active_keys),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" active"),
            ])];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Key Pools"))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(" quit  "),
        Span::styled("r", Style::default().fg(Color::Cyan)),
        Span::raw(" refresh  "),
        Span::styled("↑↓", Style::default().fg(Color::Cyan)),
        Span::raw(" scroll"),
    ]))
    .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, area);
}
