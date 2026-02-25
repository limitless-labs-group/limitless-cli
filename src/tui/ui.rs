use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use super::app::App;

/// USDC has 6 decimals.
const USDC_SCALE: f64 = 1_000_000.0;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // header
        Constraint::Length(3),  // stats bar
        Constraint::Min(8),    // orderbook (flexible)
        Constraint::Length(8),  // VWAP table
        Constraint::Length(1),  // footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_stats(frame, app, chunks[1]);
    render_orderbook(frame, app, chunks[2]);
    render_vwap(frame, app, chunks[3]);
    render_footer(frame, app, chunks[4]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let status_indicator = if app.connected {
        Span::styled(" ● ", Style::default().fg(Color::Green).bold())
    } else {
        Span::styled(" ● ", Style::default().fg(Color::Red).bold())
    };

    let elapsed = app
        .last_update
        .map(|t| {
            let secs = t.elapsed().as_secs_f64();
            if secs < 1.0 {
                format!("{:.0}ms", secs * 1000.0)
            } else {
                format!("{:.1}s", secs)
            }
        })
        .unwrap_or_else(|| "-".to_string());

    let line1 = Line::from(vec![
        Span::styled(
            " LIMITLESS ORDERBOOK MONITOR ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ "),
        Span::styled(&app.slug, Style::default().fg(Color::Yellow).bold()),
    ]);

    let line2 = Line::from(vec![
        Span::raw(" "),
        status_indicator,
        Span::styled(&app.status_msg, Style::default().fg(Color::DarkGray)),
        Span::raw(" │ "),
        Span::raw(format!("Updates: {}", app.update_count)),
        Span::raw(" │ "),
        Span::raw(format!("Last: {} ago", elapsed)),
    ]);

    let header = Paragraph::new(vec![line1, line2])
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_stats(frame: &mut Frame, app: &App, area: Rect) {
    let midpoint_str = app
        .midpoint
        .map(|m| format!("${:.4}", m))
        .unwrap_or_else(|| "-".to_string());

    let spread_str = app
        .spread
        .map(|s| format!("{:.4}", s))
        .unwrap_or_else(|| "-".to_string());

    let best_bid_str = app
        .best_bid
        .map(|b| format!("${:.4}", b))
        .unwrap_or_else(|| "-".to_string());

    let best_ask_str = app
        .best_ask
        .map(|a| format!("${:.4}", a))
        .unwrap_or_else(|| "-".to_string());

    let bid_size_str = app
        .bids
        .first()
        .map(|(_, s)| format_size(*s))
        .unwrap_or_else(|| "-".to_string());

    let ask_size_str = app
        .asks
        .first()
        .map(|(_, s)| format_size(*s))
        .unwrap_or_else(|| "-".to_string());

    let line = Line::from(vec![
        Span::raw("  Midpoint: "),
        Span::styled(&midpoint_str, Style::default().fg(Color::White).bold()),
        Span::raw("  │  Spread: "),
        Span::styled(&spread_str, Style::default().fg(Color::White).bold()),
        Span::raw("  │  Bid: "),
        Span::styled(&best_bid_str, Style::default().fg(Color::Green)),
        Span::styled(format!(" ({})", bid_size_str), Style::default().fg(Color::DarkGray)),
        Span::raw("  Ask: "),
        Span::styled(&best_ask_str, Style::default().fg(Color::Red)),
        Span::styled(format!(" ({})", ask_size_str), Style::default().fg(Color::DarkGray)),
    ]);

    let stats = Paragraph::new(line)
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(stats, area);
}

fn render_orderbook(frame: &mut Frame, app: &App, area: Rect) {
    let halves = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_bids(frame, app, halves[0]);
    render_asks(frame, app, halves[1]);
}

fn render_bids(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Price").style(Style::default().bold()),
        Cell::from("Size").style(Style::default().bold()),
        Cell::from("Total").style(Style::default().bold()),
    ]);

    let mut cumulative = 0.0;
    let rows: Vec<Row> = app
        .bids
        .iter()
        .take(max_orderbook_rows(area))
        .map(|(price, raw_size)| {
            let scaled = *raw_size / USDC_SCALE;
            cumulative += scaled;
            Row::new(vec![
                Cell::from(format!("${:.4}", price))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(format_size(*raw_size))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.2}", cumulative))
                    .style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" BIDS (Buy) ")
            .title_style(Style::default().fg(Color::Green).bold())
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
}

fn render_asks(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Price").style(Style::default().bold()),
        Cell::from("Size").style(Style::default().bold()),
        Cell::from("Total").style(Style::default().bold()),
    ]);

    let mut cumulative = 0.0;
    let rows: Vec<Row> = app
        .asks
        .iter()
        .take(max_orderbook_rows(area))
        .map(|(price, raw_size)| {
            let scaled = *raw_size / USDC_SCALE;
            cumulative += scaled;
            Row::new(vec![
                Cell::from(format!("${:.4}", price))
                    .style(Style::default().fg(Color::Red)),
                Cell::from(format_size(*raw_size))
                    .style(Style::default().fg(Color::Red)),
                Cell::from(format!("{:.2}", cumulative))
                    .style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" ASKS (Sell) ")
            .title_style(Style::default().fg(Color::Red).bold())
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
}

fn render_vwap(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Depth").style(Style::default().bold()),
        Cell::from("Buy VWAP").style(Style::default().fg(Color::Red).bold()),
        Cell::from("Buy Shares").style(Style::default().bold()),
        Cell::from("Sell VWAP").style(Style::default().fg(Color::Green).bold()),
        Cell::from("Sell Shares").style(Style::default().bold()),
    ]);

    let rows: Vec<Row> = app
        .vwaps
        .iter()
        .map(|v| {
            let buy_vwap = v
                .vwap_buy
                .map(|p| format!("${:.4}", p))
                .unwrap_or_else(|| "—".to_string());
            let sell_vwap = v
                .vwap_sell
                .map(|p| format!("${:.4}", p))
                .unwrap_or_else(|| "—".to_string());
            let buy_shares = if v.buy_shares > 0.0 {
                format!("{:.2}", v.buy_shares)
            } else {
                "—".to_string()
            };
            let sell_shares = if v.sell_shares > 0.0 {
                format!("{:.2}", v.sell_shares)
            } else {
                "—".to_string()
            };

            Row::new(vec![
                Cell::from(format!("${:.0}", v.depth_usd))
                    .style(Style::default().fg(Color::Yellow)),
                Cell::from(buy_vwap).style(Style::default().fg(Color::Red)),
                Cell::from(buy_shares).style(Style::default().fg(Color::DarkGray)),
                Cell::from(sell_vwap).style(Style::default().fg(Color::Green)),
                Cell::from(sell_shares).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(22),
            Constraint::Percentage(18),
            Constraint::Percentage(22),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" VWAP Analysis ")
            .title_style(Style::default().fg(Color::Cyan).bold())
            .borders(Borders::ALL),
    );

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, _app: &App, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " q",
            Style::default().fg(Color::Yellow).bold(),
        ),
        Span::raw("/"),
        Span::styled(
            "Esc",
            Style::default().fg(Color::Yellow).bold(),
        ),
        Span::raw(" quit  "),
        Span::styled(
            "Ctrl+C",
            Style::default().fg(Color::Yellow).bold(),
        ),
        Span::raw(" exit"),
    ]));

    frame.render_widget(footer, area);
}

// ── Helpers ──────────────────────────────────────────────────────────

fn format_size(raw: f64) -> String {
    let scaled = raw / USDC_SCALE;
    if scaled >= 1000.0 {
        format!("{:.1}K", scaled / 1000.0)
    } else {
        format!("{:.2}", scaled)
    }
}

fn max_orderbook_rows(area: Rect) -> usize {
    // Account for block borders (2) and header row (1)
    area.height.saturating_sub(3) as usize
}
