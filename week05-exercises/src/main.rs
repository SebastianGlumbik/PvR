use ratatui::crossterm::event::KeyEventKind;
use ratatui::text::Line;
use ratatui::widgets::{BorderType, Borders, Paragraph};
use ratatui::{
    crossterm::event::{self},
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    symbols::{self},
    text::Span,
    widgets::{Axis, Block, Chart, Dataset},
    DefaultTerminal,
};
use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;

// 1 (warm-up): Figure out how many cores does your PC have (with Rust code :)) and print it
// You don't need an external crate for it, there is a function for it in the stdlib

// 2: Measure the CPU utilization of each core on your computer, e.g. using the
// [psutil](https://docs.rs/psutil) crate.
// Then print the utilization every second.
// You can use the `stress -c <number-of-cores>` command to generate some heat on your PC.

// 3: Implement a simple "progress bar" that will render the usage for each core in a
// visual way.
// E.g. [xxxxx.....]

// 4: Make the progress bar colored, based on the utilization.
// Utilization < 30% should be green, < 70% should be yellow, and higher utilization should be
// red. Try to find a crate that will help you with coloring strings.

// 5: create a TUI application that will draw a historical chart of average CPU usage.
// Display the current per-core CPU usage next to the chart.
// Use the [ratatui](https://ratatui.rs/tutorials/hello-world/) crate.
// Warning: if you want to draw the per-cope "htop progress bars" with ratatui, don't combine
// other coloring crates with Ratatui; use Ratatui's colors and styles instead.

pub struct App {
    collector: psutil::cpu::CpuPercentCollector,
    terminal: DefaultTerminal,
    usages: VecDeque<(f64, f64)>,
}

impl App {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            collector: psutil::cpu::CpuPercentCollector::new()?,
            terminal: ratatui::init(),
            usages: VecDeque::with_capacity(120),
        })
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut low_usage: Vec<(f64, f64)> = Vec::with_capacity(120);
        let mut medium_usage: Vec<(f64, f64)> = Vec::with_capacity(120);
        let mut high_usage: Vec<(f64, f64)> = Vec::with_capacity(120);
        let cpu_count = std::thread::available_parallelism()?;

        loop {
            let cpu_percent = self.collector.cpu_percent()? as f64;
            let cpu_percent_percpu = self.collector.cpu_percent_percpu()?;
            low_usage.clear();
            medium_usage.clear();
            high_usage.clear();
            if self.usages.len() >= 120 {
                self.usages.pop_front();
            }
            self.usages.push_back((1.0, cpu_percent));
            self.usages.iter_mut().for_each(|(x, y)| {
                *x -= 1.0;
                match *y {
                    0f64..30f64 => {
                        low_usage.push((*x, *y));
                    }
                    30f64..=80f64 => {
                        medium_usage.push((*x, *y));
                    }
                    _ => {
                        high_usage.push((*x, *y));
                    }
                }
            });

            self.terminal.clear()?;
            self.terminal.draw(|frame| {
                let [left, right] =
                    Layout::horizontal([Constraint::Length(120 + 7), Constraint::Length(40)])
                        .areas(frame.area());

                let x_labels = vec![
                    Span::styled(format!("{}s", left.width - 7), Style::default()),
                    Span::styled(
                        format!("{}s", 0),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ];
                let datasets = vec![
                    Dataset::default()
                        .marker(symbols::Marker::HalfBlock)
                        .style(Style::default().fg(Color::Green))
                        .data(&low_usage),
                    Dataset::default()
                        .marker(symbols::Marker::HalfBlock)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&medium_usage),
                    Dataset::default()
                        .marker(symbols::Marker::HalfBlock)
                        .style(Style::default().fg(Color::Red))
                        .data(&high_usage),
                ];

                let chart = Chart::new(datasets)
                    .block(
                        Block::bordered()
                            .border_type(BorderType::Rounded)
                            .white()
                            .title_alignment(Alignment::Center)
                            .title_bottom(format!(" CPU usage: {:.2} % ", cpu_percent)),
                    )
                    .x_axis(
                        Axis::default()
                            .labels_alignment(Alignment::Right)
                            .style(Style::default().fg(Color::White))
                            .labels(x_labels)
                            .bounds([8f64 - (left.width as f64), 0f64]),
                    )
                    .y_axis(
                        Axis::default()
                            .labels_alignment(Alignment::Right)
                            .style(Style::default().fg(Color::White))
                            .labels(["0%".bold(), "50%".into(), "100%".bold()])
                            .bounds([0.0, 100.0]),
                    );
                frame.render_widget(chart, left);

                /* Right */
                let mut lines = vec![];
                for (i, util) in cpu_percent_percpu.iter().enumerate() {
                    let ratio = (util / 10.0) as usize;
                    let inner = "x".repeat(ratio);
                    let inner = match *util as i32 {
                        ..30 => inner.green(),
                        30..70 => inner.yellow(),
                        _ => inner.red(),
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("CPU{i}: {util:6.2} %"),
                            Style::default().fg(Color::White),
                        ),
                        Span::styled(" [".to_string(), Style::default().fg(Color::White)),
                        inner,
                        Span::styled(
                            format!("{}]\n", ".".repeat(10 - ratio)),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }

                let p = Paragraph::new(lines).alignment(Alignment::Center).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White))
                        .title_bottom(format!(" {} CPUs ", cpu_count))
                        .title_alignment(Alignment::Center)
                        .border_type(BorderType::Rounded),
                );
                frame.render_widget(p, right);
            })?;
            sleep(Duration::from_secs(1));
            if event::poll(Duration::ZERO)? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn main() {
    if let Err(e) = App::new().and_then(|mut app| app.run()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
