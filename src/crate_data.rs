//! A module for the data of the uploaded crates.

use std::cmp::*;
use std::collections::HashSet;
use std::hash::*;

/// A github repo
#[derive(Clone, PartialEq, Eq)]
pub struct GithubRepo {
    /// The username
    pub user: String,
    /// The name of the repo
    pub name: String,
}

/// A crate storage (host + username + name)
#[derive(Clone, PartialEq, Eq)]
pub enum CrateStorage {
    /// Crate on github
    Github(GithubRepo),
}
impl CrateStorage {
    pub fn name(&self) -> String {
        match self {
            &CrateStorage::Github(ref repo) => format!("github:{}:{}", repo.user, repo.name)
        }
    }
}

/// A hashed (for the sake of privacy) IP
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ip(u64);

impl Ip {
    pub fn new<T: Hash>(ip: T) -> Ip {
        // Hash the ip
        let mut hasher = SipHasher::new();
        ip.hash(&mut hasher);
        Ip(hasher.finish() % 15000)
    }
}

/// A crate
#[derive(Clone, PartialEq)]
pub struct Crate {
    /// The description of this crate
    pub description: String,
    /// The crate storage
    pub repo: CrateStorage,
    /// The time when the crate was uploaded
    pub uploaded: f64,
    /// The hash of the uploaders' IP
    pub uploader: Ip,
    /// The number of votes it have got
    pub votes: i32,
    /// People who upvoted this crate
    pub voters: HashSet<Ip>,
}

impl Crate {
    /// Get the score of the crate
    pub fn get_score(&self) -> f64 {
        // Calculate ln(votes). This is done to limit the duration of a crate being in the top.
        let log_votes = ((self.votes + 1) as f64).ln();

        // TODO: Change weight?
        let score = self.uploaded + 60.0 * 60.0 * 1.0 * log_votes;

        score
    }

    /// Upvote this crate.
    pub fn vote(&mut self, ip: Ip) {
        if !self.voters.contains(&ip) {
            // Register ip
            self.voters.insert(ip);
            // Update vote number
            self.votes += 1;
            print!("!");
        }
    }

    /// Convert this crate to a string of the format, "*:*:...", for example
    /// "rust-lang:rust:the rust compiler"
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.repo.name(), self.description)
    }
}

