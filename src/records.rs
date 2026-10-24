use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::mode::{MODE_COUNT, Mode};

const RECORDS_VERSION: u32 = 1;
const RECORDS_FILE: &str = "records.toml";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    score: u64,
    lines: u32,
    level: u32,
    elapsed: Duration,
}

impl Record {
    pub const fn new(score: u64, lines: u32, level: u32, elapsed: Duration) -> Self {
        Self {
            score,
            lines,
            level,
            elapsed,
        }
    }

    pub const fn score(self) -> u64 {
        self.score
    }

    pub const fn elapsed(self) -> Duration {
        self.elapsed
    }

    fn is_better_than(self, mode: Mode, other: Self) -> bool {
        match mode {
            Mode::Sprint => self.elapsed < other.elapsed,
            Mode::Marathon | Mode::Ultra => self.score > other.score,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PersonalBests {
    entries: [Option<Record>; MODE_COUNT],
}

impl PersonalBests {
    pub fn best(&self, mode: Mode) -> Option<Record> {
        self.entries[mode.index()]
    }

    pub fn submit(&mut self, mode: Mode, record: Record) -> bool {
        let slot = &mut self.entries[mode.index()];
        if slot.is_some_and(|current| !record.is_better_than(mode, current)) {
            return false;
        }
        *slot = Some(record);
        true
    }
}

pub struct Records {
    path: Option<PathBuf>,
    bests: PersonalBests,
}

impl Records {
    pub fn load() -> Self {
        let path = records_path();
        let bests = path.as_deref().and_then(load_file).unwrap_or_default();
        Self { path, bests }
    }

    pub fn bests(&self) -> &PersonalBests {
        &self.bests
    }

    pub fn submit(&mut self, mode: Mode, record: Record) -> bool {
        if !self.bests.submit(mode, record) {
            return false;
        }
        self.persist();
        true
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self {
            path: None,
            bests: PersonalBests::default(),
        }
    }

    fn persist(&self) {
        if let Some(path) = &self.path {
            let _ = save_file(path, &self.bests);
        }
    }
}

fn records_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "uint82", "tetris").map(|dirs| dirs.data_dir().join(RECORDS_FILE))
}

fn load_file(path: &Path) -> Option<PersonalBests> {
    let contents = fs::read_to_string(path).ok()?;
    let stored: StoredRecords = toml::from_str(&contents).ok()?;
    if stored.version != RECORDS_VERSION {
        return None;
    }
    Some(PersonalBests::from(stored))
}

fn save_file(path: &Path, bests: &PersonalBests) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(&StoredRecords::from(bests)).map_err(io::Error::other)?;
    let temporary = path.with_extension("toml.tmp");
    fs::write(&temporary, contents)?;
    replace_file(&temporary, path)
}

fn replace_file(temporary: &Path, path: &Path) -> io::Result<()> {
    if cfg!(windows) && path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)
}

#[derive(Serialize, Deserialize)]
struct StoredRecords {
    version: u32,
    #[serde(default)]
    marathon: Option<StoredRecord>,
    #[serde(default)]
    sprint: Option<StoredRecord>,
    #[serde(default)]
    ultra: Option<StoredRecord>,
}

impl From<&PersonalBests> for StoredRecords {
    fn from(bests: &PersonalBests) -> Self {
        let [marathon, sprint, ultra] =
            Mode::ALL.map(|mode| bests.best(mode).map(StoredRecord::from));
        Self {
            version: RECORDS_VERSION,
            marathon,
            sprint,
            ultra,
        }
    }
}

impl From<StoredRecords> for PersonalBests {
    fn from(stored: StoredRecords) -> Self {
        Self {
            entries: [stored.marathon, stored.sprint, stored.ultra]
                .map(|record| record.map(Record::from)),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredRecord {
    score: u64,
    lines: u32,
    level: u32,
    millis: u64,
}

impl From<Record> for StoredRecord {
    fn from(record: Record) -> Self {
        Self {
            score: record.score,
            lines: record.lines,
            level: record.level,
            millis: u64::try_from(record.elapsed.as_millis()).unwrap_or(u64::MAX),
        }
    }
}

impl From<StoredRecord> for Record {
    fn from(record: StoredRecord) -> Self {
        Self::new(
            record.score,
            record.lines,
            record.level,
            Duration::from_millis(record.millis),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn record(score: u64, elapsed_secs: u64) -> Record {
        Record::new(score, 20, 2, Duration::from_secs(elapsed_secs))
    }

    #[test]
    fn first_result_becomes_the_best() {
        let mut bests = PersonalBests::default();
        assert!(bests.submit(Mode::Marathon, record(1000, 60)));
        assert_eq!(bests.best(Mode::Marathon), Some(record(1000, 60)));
    }

    #[test]
    fn marathon_and_ultra_records_compare_score() {
        let mut bests = PersonalBests::default();
        bests.submit(Mode::Marathon, record(1000, 60));
        assert!(!bests.submit(Mode::Marathon, record(900, 30)));
        assert!(bests.submit(Mode::Marathon, record(1100, 90)));
        assert!(bests.submit(Mode::Ultra, record(500, 120)));
        assert!(!bests.submit(Mode::Ultra, record(499, 120)));
    }

    #[test]
    fn sprint_records_compare_time() {
        let mut bests = PersonalBests::default();
        bests.submit(Mode::Sprint, record(500, 60));
        assert!(!bests.submit(Mode::Sprint, record(5000, 90)));
        assert!(bests.submit(Mode::Sprint, record(100, 45)));
        assert_eq!(
            bests.best(Mode::Sprint).map(Record::elapsed),
            Some(Duration::from_secs(45))
        );
    }

    #[test]
    fn records_round_trip_through_a_file() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join(RECORDS_FILE);
        let mut bests = PersonalBests::default();
        bests.submit(Mode::Marathon, record(1200, 80));
        bests.submit(Mode::Sprint, record(700, 55));
        save_file(&path, &bests).expect("records save");
        assert_eq!(load_file(&path), Some(bests));
    }

    #[test]
    fn missing_and_invalid_files_load_without_records() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join(RECORDS_FILE);
        assert_eq!(load_file(&path), None);
        fs::write(&path, "not valid toml").expect("write invalid file");
        assert_eq!(load_file(&path), None);
    }

    #[test]
    fn unsupported_versions_are_ignored() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join(RECORDS_FILE);
        fs::write(&path, "version = 99\n").expect("write future version");
        assert_eq!(load_file(&path), None);
    }
}
