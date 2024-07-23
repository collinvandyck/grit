use color_eyre::{eyre::Context, Report};
use git2::BranchType;
use std::{ops::Deref, sync::Arc};

use super::branch::Branch;

#[derive(Debug, Clone)]
pub struct Repository {
    pub(super) inner: Arc<Inner>,
}

pub struct Inner {
    repo: git2::Repository,
}

impl Deref for Inner {
    type Target = git2::Repository;
    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<repo>>")
    }
}

impl Repository {
    pub fn current() -> Result<Self, Report> {
        let cwd = std::env::current_dir().wrap_err("get current dir")?;
        let flags = git2::RepositoryOpenFlags::FROM_ENV;
        let ceiling = &[] as &[&std::ffi::OsStr];
        let repo = git2::Repository::open_ext(cwd, flags, ceiling).wrap_err("open repo")?;
        let inner = Arc::new(Inner { repo });
        Ok(Self { inner })
    }

    pub fn branches(&self, typ: Option<BranchType>) -> Result<Vec<Branch>, Report> {
        Ok(self
            .inner
            .repo
            .branches(typ)
            .wrap_err("repo branches")
            .and_then(|iter| {
                iter.map(|br_res| {
                    let (branch, typ) = br_res.wrap_err("branch")?;
                    let name = branch.name().wrap_err("branch name")?;
                    if let Some(name) = name {
                        let branch = Branch::new(self, name, typ);
                        Ok(Some(branch))
                    } else {
                        Ok(None)
                    }
                })
                .collect::<Result<Vec<_>, _>>()
            })?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>())
    }
}
