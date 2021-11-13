use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table},
};

pub struct Widgets {
    outer_border: Block<'static>,

    // decryption panel
    forged_cypher_text_view: Table<'static>,
    intermediate_view: Table<'static>,
    plaintext_view: Table<'static>,

    // status panel
    status_panel_border: Block<'static>,
    progress_bar: Gauge<'static>,
    logs_view: Block<'static>,
}

impl Widgets {
    pub fn build(title_style: Style) -> Widgets {
        Widgets {
            outer_border: build_outer_border(title_style),

            forged_cypher_text_view: build_forged_cypher_text_view(title_style, vec![]),
            intermediate_view: build_intermediate_view(
                title_style,
                vec![Row::new(vec!["hi proper data here"])],
            ),
            plaintext_view: build_plaintext_view(
                title_style,
                vec![Row::new(vec![
                    "baaaaaaaaaaaaaaaaaaaaaaaaaaaaaab",
                    "baaaaaaaaaaaaaab",
                ])],
            ),

            status_panel_border: build_status_panel_border(title_style),
            progress_bar: build_progress_bar(),
            logs_view: build_log_view(title_style),
        }
    }

    pub fn outer_border(&self) -> &Block<'static> {
        &self.outer_border
    }
    pub fn forged_block_view(&self) -> &Table<'static> {
        &self.forged_cypher_text_view
    }
    pub fn intermediate_block_view(&self) -> &Table<'static> {
        &self.intermediate_view
    }
    pub fn plaintext_view(&self) -> &Table<'static> {
        &self.plaintext_view
    }
    pub fn status_panel_border(&self) -> &Block<'static> {
        &self.status_panel_border
    }
    pub fn progress_bar(&self) -> &Gauge<'static> {
        &self.progress_bar
    }
    pub fn logs_view(&self) -> &Block<'static> {
        &self.logs_view
    }
}

fn build_outer_border(title_style: Style) -> Block<'static> {
    let title = {
        let mut title = Span::from("rustpad");
        title.style = title_style;
        title
    };

    Block::default().title(title).borders(Borders::NONE)
}

fn build_forged_cypher_text_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Forged block");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_intermediate_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Intermediate block");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .widths(&[Constraint::Ratio(1, 1)])
}

fn build_plaintext_view(title_style: Style, rows: Vec<Row>) -> Table {
    let title = {
        let mut title = Span::from("Plain text");
        title.style = title_style;
        title
    };

    Table::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .column_spacing(1)
        .widths(&[Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
}

fn build_status_panel_border(title_style: Style) -> Block<'static> {
    let title = {
        let mut title = Span::from("Status");
        title.style = title_style;
        title
    };

    Block::default().title(title).borders(Borders::ALL)
}

fn build_progress_bar() -> Gauge<'static> {
    let label = {
        let mut label = Span::from("TODO: 72");
        label.style = Style::default().fg(Color::DarkGray);
        label
    };

    Gauge::default()
        .gauge_style(Style::default().fg(Color::LightCyan))
        .percent(72)
        .label(label)
        .use_unicode(true)
}

fn build_log_view(title_style: Style) -> Block<'static> {
    let title = {
        let mut title = Span::from("Log");
        title.style = title_style;
        title
    };

    Block::default().title(title).borders(Borders::NONE)
}
