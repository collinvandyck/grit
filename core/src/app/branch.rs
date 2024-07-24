use crate::git;
use crate::prelude::*;
use git2::BranchType;
use ratatui::widgets::{ListItem, ListState};

const LOCAL_BRANCH_COLOR: Color = SLATE.c200;
const REMOTE_BRANCH_COLOR: Color = RED.c200;

#[derive(Default)]
pub struct List {
    pub items: Vec<git::Branch>,
    pub state: ListState,
    pub sort: Sort,
    pub filter: Filter,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Sort {
    NameAscending,
    NameDescending,
    DateAscending,
    #[default]
    DateDescending,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Filter(Option<git2::BranchType>);

impl List {
    pub fn current(&self) -> Option<&git::Branch> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
    pub fn build(branches: Vec<git::Branch>, filter: Filter) -> Self {
        let sort = Sort::default();
        let items = branches;
        let state = ListState::default();
        let mut list = List {
            items,
            state,
            sort,
            filter,
        };
        list.sort();
        list.state.select_first();
        list
    }

    pub fn sort(&mut self) {
        match self.sort {
            Sort::NameAscending => self.items.sort_by(|b1, b2| b1.name.cmp(&b2.name)),
            Sort::NameDescending => self.items.sort_by(|b1, b2| b2.name.cmp(&b1.name)),
            Sort::DateAscending => self.items.sort_by(|b1, b2| {
                let b1 = b1.commits.first().as_ref().map(|c| c.timestamp.epoch());
                let b2 = b2.commits.first().as_ref().map(|c| c.timestamp.epoch());
                b1.cmp(&b2)
            }),
            Sort::DateDescending => self.items.sort_by(|b1, b2| {
                let i1 = b1.commits.first().as_ref().map(|c| c.timestamp.epoch());
                let i2 = b2.commits.first().as_ref().map(|c| c.timestamp.epoch());
                i2.cmp(&i1)
            }),
        };
    }
}

impl From<&git::Branch> for ListItem<'_> {
    fn from(value: &git::Branch) -> Self {
        let name = value.name.to_string();
        let line = match value.typ {
            BranchType::Local => {
                Line::styled(name, LOCAL_BRANCH_COLOR).add_modifier(Modifier::BOLD)
            }
            BranchType::Remote => {
                Line::styled(name, REMOTE_BRANCH_COLOR).add_modifier(Modifier::DIM)
            }
        };
        ListItem::new(line)
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self(Some(BranchType::Local))
    }
}

impl Filter {
    pub fn typ(&self) -> Option<BranchType> {
        self.0.clone()
    }

    #[allow(unused)]
    pub fn cycle(&mut self) {
        self.0 = match self.0 {
            None => Some(BranchType::Local),
            Some(BranchType::Local) => Some(BranchType::Remote),
            Some(BranchType::Remote) => None,
        };
    }
}
