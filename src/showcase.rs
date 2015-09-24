//! A module for the `showcase` (the collection of uploaded crates).

use time::precise_time_s as now;
use crate_data::*;
use std::collections::{HashMap, HashSet};

// TODO: Make ::new() function instead
/// The showcase
#[derive(Clone)]
pub struct Showcase {
    /// The uploaded crates
    pub crates: Vec<Crate>,
    /// The sorted, currently shown crates
    pub shown_crates: Vec<u64>,
    /// Additions since last update (to keep track of changes).
    pub additions: u16,
    /// Timestamps and counts to prevent spamming
    pub latest_upload: HashMap<Ip, (f64, u32)>,
    /// For checking for duplicates
    pub uploads: HashSet<String>,
}

/// Upload errors
#[derive(Clone)]
pub enum UploadError {
    /// The upload limit is reached
    LimitReached,
    /// The upload is a duplicate
    Duplicate,
}

impl Showcase {
    /// Check for updates
    fn check_update(&mut self) {
       // Update if necessary
       if self.additions >= 2 {
           self.update();
       }
    }
    /// Update the shown crates
    pub fn update(&mut self) {

        let crates = &self.crates;

        self.shown_crates = (crates.len().saturating_sub(500) as u64
                             .. crates.len() as u64).collect();

        // Sort the crates by score
        self.shown_crates.sort_by(|&a, &b| {
            use std::cmp::Ordering;

            // Calculate scores
            let x = crates[a as usize].get_score();
            let y = crates[b as usize].get_score();

            // Why? Because we want the highest scores first.
            if x < y {
                Ordering::Greater
            } else if x > y {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        // Reset the number of additions since last updated
        self.additions = 0;

        println!("Showcase updated");
    }
    /// Add new crate to showcase
    pub fn add(&mut self, new: Crate) -> Result<(), UploadError> {
        let data = new.to_string();
        if !self.uploads.contains(&data) {
            let now = now();
            // Make spam check
            let (time, rep) = *self.latest_upload.get(&new.uploader.clone()).unwrap_or(&(0.0, 0));
            if time - now > 60.0 * 60.0 || rep < 10 {

                // Add this new crate
                self.crates.push(new.clone()); // TODO: Is there a better way, without cloning?

                let new_rep = if time > 60.0 * 60.0 {
                    3 // If limit is finished set the 1 hour upload number to 5
                      // (user gains N new uploads next hour)
                } else {
                    rep + 1
                };

                // Register latest upload
                self.latest_upload.insert(new.uploader, (now, new_rep));

                // Update number of additions since last update
                self.additions += 5;
                println!("[{}] New upload: {}", new.uploaded.round(), new.repo.name());

                self.check_update();

                self.uploads.insert(data);

                Ok(())
            } else {
                Err(UploadError::LimitReached)
            }
        } else {
            Err(UploadError::Duplicate)
        }
    }
    /// Upvote
    pub fn vote(&mut self, id: u64, ip: Ip) {
        if let Some(cand) = self.crates.get_mut(id as usize) { // Is it bad practice to use .get() over Index?
            cand.vote(ip);
        }
        self.additions += 1;
        self.check_update();
    }
}
