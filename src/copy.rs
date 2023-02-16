use std::path::PathBuf;

use fs_extra::dir::{copy_with_progress, get_size, CopyOptions};

use crate::Args;

#[derive(Clone)]
pub struct CopyQueue {
    source: PathBuf,
    destinations: Vec<PathBuf>,
}

impl From<&Args> for CopyQueue {
    fn from(a: &Args) -> Self {
        Self {
            source: a.copy_from.clone(),
            destinations: a.drives.clone(),
        }
    }
}

impl CopyQueue {
    ///
    /// Starts the copy process using CopyQueue's source and destination variables
    ///
    /// Callbacks:
    /// * `onpercentage` - `|percentage: usize, source_dir: PathBuf| -> ()`
    /// * `oncomplete`   - `|| -> ()`
    ///
    pub fn start_copy(
        &self,
        onpercentage: Box<impl Fn(usize, PathBuf)>,
        oncomplete: Box<impl FnOnce()>,
    ) {
        let total_bytes = get_size(self.source.clone()).unwrap();
        for dest in self.destinations.clone() {
            let opt = CopyOptions {
                overwrite: true,
                content_only: true,
                ..CopyOptions::new()
            };
            copy_with_progress(self.source.clone(), dest.clone(), &opt, |proc_info| {
                let percentage = (proc_info.copied_bytes as f64 / total_bytes as f64) * 100.;
                onpercentage(percentage as usize, dest.clone());
                fs_extra::dir::TransitProcessResult::ContinueOrAbort
            })
            .unwrap();
        }

        oncomplete();
    }
}
