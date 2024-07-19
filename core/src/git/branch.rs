use super::repo;
use crate::prelude::*;
use chrono::{DateTime, Utc};
use color_eyre::{
    eyre::{Context, ContextCompat},
    Report,
};
use git2::BranchType;
use std::{fmt::Display, sync::Arc};

pub struct Branch {
    repo: Arc<repo::Inner>,
    name: String,
    typ: BranchType,
    commits: Vec<Commit>,
}

impl Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Branch {
    pub fn new(repo: Arc<repo::Inner>, name: impl AsRef<str>, typ: BranchType) -> Self {
        let commits = Vec::default();
        let name = name.as_ref().to_string();
        Self {
            repo,
            name,
            typ,
            commits,
        }
    }

    pub fn commits(&self) -> &[Commit] {
        self.commits.as_ref()
    }

    /// Loads the latest commits for this branch
    pub fn load(&mut self) -> Result<(), Report> {
        let branch = self
            .repo
            .find_branch(&self.name, self.typ)
            .wrap_err("load branch")?;
        let head = branch.get();
        let commit = head.peel_to_commit().wrap_err("get commit for ref")?;
        let mut revwalk = self.repo.revwalk().wrap_err("revwalk")?;
        revwalk.push(commit.id()).wrap_err("revwalk push commit")?;
        self.commits = revwalk
            .take(100)
            .map(|sha| {
                sha.wrap_err("revwalk sha")
                    .and_then(|sha| self.repo.find_commit(sha).wrap_err("find commit"))
                    .and_then(|cmt| cmt.try_into().wrap_err("get commit"))
            })
            .collect::<Result<Vec<_>, _>>()
            .wrap_err("get commits")?;
        Ok(())
    }
}

struct BranchId {
    name: String,
    typ: BranchType,
}

#[derive(Clone)]
pub struct Commit {
    pub summary: String,
    pub message: String,
    pub author: Author,
    pub timestamp: Timestamp,
}

impl TryFrom<git2::Commit<'_>> for Commit {
    type Error = Report;
    fn try_from(commit: git2::Commit<'_>) -> Result<Self, Self::Error> {
        let summary = commit.summary().map(ToOwned::to_owned).unwrap_or_default();
        let message = commit.message().map(ToOwned::to_owned).unwrap_or_default();
        let author = commit.author().into();
        let timestamp = commit.time().try_into()?;
        Ok(Self {
            summary,
            message,
            author,
            timestamp,
        })
    }
}

#[derive(Clone)]
pub struct Timestamp {
    epoch: i64,
    dt: DateTime<Utc>,
}

impl Timestamp {
    pub fn epoch(&self) -> i64 {
        self.epoch
    }
    fn format(&self) -> impl Display {
        self.dt.format("%m/%d/%Y %H:%M:%S")
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format().fmt(f)
    }
}

impl TryFrom<git2::Time> for Timestamp {
    type Error = Report;
    fn try_from(value: git2::Time) -> Result<Self, Self::Error> {
        let epoch = value.seconds();
        let offset = value.offset_minutes();
        // todo: how to deal with the offset when creating a chrono datetime?
        let dt = DateTime::from_timestamp(epoch, 0)
            .wrap_err_with(|| format!("no timestamp available for epoch {epoch}"))?;
        let ts = Self { epoch, dt };
        Ok(ts)
    }
}

#[derive(Clone)]
pub struct Author {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl From<git2::Signature<'_>> for Author {
    fn from(sig: git2::Signature<'_>) -> Self {
        Self {
            name: sig.name().map(ToOwned::to_owned),
            email: sig.email().map(ToOwned::to_owned),
        }
    }
}
