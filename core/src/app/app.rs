use crate::{git, opts::Opts, prelude::*};
use color_eyre::eyre::Context;

use super::branch;

const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Report(#[from] color_eyre::eyre::Report),

    #[error(transparent)]
    IO(#[from] io::Error),
}

pub struct App {
    repo: git::Repository,
    branch_list: branch::List,
    exit: bool,
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf)
    }
}

impl App {
    pub fn new(opts: &Opts) -> EResult<Self> {
        if let Some(dir) = &opts.dir {
            std::env::set_current_dir(dir).wrap_err("change dir")?;
        }
        let repo = git::Repository::current().wrap_err("read repo")?;
        let branches = branch::List::default();
        let exit = false;
        let mut app = Self {
            repo,
            branch_list: branches,
            exit,
        };
        app.load_branches()?;
        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut crate::bootstrap::Tui) -> EResult<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    pub fn load_branches(&mut self) -> EResult<()> {
        let filter = self.branch_list.filter.clone();
        let branches: Vec<git::Branch> = self
            .repo
            .branches(filter.typ())
            .wrap_err("get branches")?
            .into_iter()
            .collect();
        self.branch_list = branch::List::build(branches, filter);
        Ok(())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [header, main, footer] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);
        let [list, item] = Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main);
        self.render_header(header, buf);
        self.render_branch_list(list, buf);
        self.render_selected(item, buf);
        App::render_footer(footer, buf);
    }

    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let sort = match self.branch_list.sort {
            branch::Sort::NameAscending => "name asc",
            branch::Sort::NameDescending => "name desc",
            branch::Sort::DateAscending => "date asc",
            branch::Sort::DateDescending => "date desc",
        };
        let header = format!("j/k/g/G: move [,]: sort ({sort})");
        Paragraph::new(header)
            .bold()
            .left_aligned()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("footer stuff").centered().render(area, buf);
    }

    fn render_branch_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Branches").left_aligned())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);
        let items: Vec<ListItem> = self
            .branch_list
            .items
            .iter()
            .map(|item| ListItem::from(item))
            .collect();
        let list = List::new(items.into_iter())
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.branch_list.state)
    }

    fn render_selected(&mut self, area: Rect, buf: &mut Buffer) {
        let Some(branch) = self.branch_list.current() else {
            return;
        };
        let _block = Block::new()
            .title(Line::raw("Details").left_aligned())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);
        let commits = branch
            .commits()
            .iter()
            .map(|c| {
                let summary = c.summary.as_str();
                let author = c.author.name.as_deref().unwrap_or("<none>");
                let timestamp = &c.timestamp;
                format!("{timestamp}: {author}: {summary}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        Paragraph::new(commits).render(area, buf);
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> EResult<(), Error> {
        match event::read()? {
            Event::Key(key_event) => self
                .handle_key(key_event)
                .wrap_err("handle key failed")
                .wrap_err_with(|| format!("{key_event:#?}"))?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> EResult<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        match key.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('h') | KeyCode::Left => self.select_none()?,
            KeyCode::Char('j') | KeyCode::Down => self.select_next()?,
            KeyCode::Char('k') | KeyCode::Up => self.select_previous()?,
            KeyCode::Char('g') | KeyCode::Home => self.select_first()?,
            KeyCode::Char('G') | KeyCode::End => self.select_last()?,
            KeyCode::Char('s') => self.cycle_sort()?,
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.toggle_branch()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn cycle_sort(&mut self) -> EResult<()> {
        self.branch_list.sort = match self.branch_list.sort {
            branch::Sort::NameAscending => branch::Sort::NameDescending,
            branch::Sort::NameDescending => branch::Sort::DateAscending,
            branch::Sort::DateAscending => branch::Sort::DateDescending,
            branch::Sort::DateDescending => branch::Sort::NameAscending,
        };
        self.branch_list.sort();
        self.branch_list.state.select_first();
        Ok(())
    }

    fn select_none(&mut self) -> EResult<()> {
        self.branch_list.state.select(None);
        Ok(())
    }

    fn select_next(&mut self) -> EResult<()> {
        self.branch_list.state.select_next();
        Ok(())
    }

    fn select_previous(&mut self) -> EResult<()> {
        self.branch_list.state.select_previous();
        Ok(())
    }

    fn select_first(&mut self) -> EResult<()> {
        self.branch_list.state.select_first();
        Ok(())
    }

    fn select_last(&mut self) -> EResult<()> {
        self.branch_list.state.select_last();
        Ok(())
    }

    fn toggle_branch(&mut self) -> EResult<()> {
        if let Some(i) = self.branch_list.state.selected() {
            let _branch = &self.branch_list.items[i];
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
