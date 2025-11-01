use crate::prelude::Record;
use gnss_qc_traits::{Merge, MergeError};

impl Merge for Record {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }

    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        for (rhs_k, rhs_v) in rhs.map.iter() {
            if let Some(lhs_v) = self.map.get_mut(&rhs_k) {
                if let Some(rhs_rms) = rhs_v.rms {
                    if lhs_v.rms.is_none() {
                        lhs_v.rms = Some(rhs_rms);
                    }
                }
                if let Some(rhs_height) = rhs_v.height {
                    if lhs_v.height.is_none() {
                        lhs_v.height = Some(rhs_height);
                    }
                }
            } else {
                self.map.insert(*rhs_k, *rhs_v);
            }
        }

        Ok(())
    }
}
