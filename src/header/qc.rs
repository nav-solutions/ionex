use crate::prelude::Header;

use gnss_qc_traits::{Merge, MergeError};

impl Merge for Header {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }

    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        self.version = std::cmp::min(self.version, rhs.version);

        if self.program.is_none() {
            if let Some(program) = &rhs.program {
                self.program = Some(program.clone());
            }
        }

        if self.run_by.is_none() {
            if let Some(run_by) = &rhs.run_by {
                self.run_by = Some(run_by.clone());
            }
        }

        if self.date.is_none() {
            if let Some(date) = &rhs.date {
                self.date = Some(date.clone());
            }
        }

        if self.license.is_none() {
            if let Some(license) = &rhs.license {
                self.license = Some(license.clone());
            }
        }

        if self.doi.is_none() {
            if let Some(doi) = &rhs.doi {
                self.doi = Some(doi.clone());
            }
        }

        match self.description {
            Some(ref mut desc) => {
                if let Some(rhs) = &rhs.description {
                    desc.push_str(rhs);
                }
            },
            None => {
                if let Some(desc) = &rhs.description {
                    self.description = Some(desc.clone());
                }
            },
        }

        self.epoch_of_first_map = std::cmp::min(self.epoch_of_first_map, rhs.epoch_of_first_map);

        self.epoch_of_last_map = std::cmp::max(self.epoch_of_last_map, rhs.epoch_of_last_map);

        if self.reference_system != rhs.reference_system {
            // TODO: both must match
        }

        if self.mapf != rhs.mapf {
            // TODO: both must match
        }

        if self.map_dimension != rhs.map_dimension {
            // TODO: both must match
        }

        self.sampling_period = std::cmp::min(self.sampling_period, rhs.sampling_period);

        if rhs.elevation_cutoff > self.elevation_cutoff {
            self.elevation_cutoff = rhs.elevation_cutoff;
        }

        // TODO: merge grid def

        for comment in rhs.comments.iter() {
            if !self.comments.contains(&comment) {
                self.comments.push(comment.clone());
            }
        }

        // insert special comment
        let merge_comment = "FILE MERGE".to_string();

        if !self.comments.contains(&merge_comment) {
            self.comments.push(merge_comment.clone());
        }

        Ok(())
    }
}
