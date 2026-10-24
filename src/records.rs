use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::daily::DailyDate;
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
            Mode::Marathon | Mode::Time | Mode::Classic => self.score > other.score,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DailyBest {
    date: DailyDate,
    record: Record,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PersonalBests {
    entries: [Option<Record>; MODE_COUNT],
    daily_best: Option<DailyBest>,
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

    pub fn daily(&self, date: DailyDate) -> Option<Record> {
        self.daily_best
            .filter(|best| best.date == date)
            .map(|best| best.record)
    }

    pub fn submit_daily(&mut self, date: DailyDate, record: Record) -> bool {
        if let Some(best) = self.daily_best
            && best.date == date
            && record.score <= best.record.score
        {
            return false;
        }
        self.daily_best = Some(DailyBest { date, record });
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

    pub fn submit_daily(&mut self, date: DailyDate, record: Record) -> bool {
        if !self.bests.submit_daily(date, record) {
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
    #[serde(default, alias = "ultra")]
    time: Option<StoredRecord>,
    #[serde(default)]
    classic: Option<StoredRecord>,
    #[serde(default)]
    daily: Option<StoredDaily>,
}

impl From<&PersonalBests> for StoredRecords {
    fn from(bests: &PersonalBests) -> Self {
        let [marathon, sprint, time, classic] =
            Mode::ALL.map(|mode| bests.best(mode).map(StoredRecord::from));
        Self {
            version: RECORDS_VERSION,
            marathon,
            sprint,
            time,
            classic,
            daily: bests.daily_best.map(StoredDaily::from),
        }
    }
}

impl From<StoredRecords> for PersonalBests {
    fn from(stored: StoredRecords) -> Self {
        Self {
            entries: [stored.marathon, stored.sprint, stored.time, stored.classic]
                .map(|record| record.map(Record::from)),
            daily_best: stored.daily.map(DailyBest::from),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredDaily {
    day: i64,
    record: StoredRecord,
}

impl From<DailyBest> for StoredDaily {
    fn from(best: DailyBest) -> Self {
        Self {
            day: best.date.days(),
            record: StoredRecord::from(best.record),
        }
    }
}

impl From<StoredDaily> for DailyBest {
    fn from(stored: StoredDaily) -> Self {
        Self {
            date: DailyDate::from_days(stored.day),
            record: Record::from(stored.record),
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
    fn score_modes_compare_score() {
        let mut bests = PersonalBests::default();
        for mode in [Mode::Marathon, Mode::Time, Mode::Classic] {
            assert!(bests.submit(mode, record(1000, 60)));
            assert!(!bests.submit(mode, record(900, 30)));
            assert!(bests.submit(mode, record(1100, 90)));
        }
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
    fn daily_records_compare_score_within_a_date() {
        let date = DailyDate::from_days(20_751);
        let mut bests = PersonalBests::default();
        assert!(bests.submit_daily(date, record(500, 120)));
        assert!(!bests.submit_daily(date, record(400, 120)));
        assert!(bests.submit_daily(date, record(600, 120)));
        assert_eq!(bests.daily(date), Some(record(600, 120)));
    }

    #[test]
    fn daily_records_reset_on_a_new_date() {
        let mut bests = PersonalBests::default();
        let first = DailyDate::from_days(20_751);
        let next = DailyDate::from_days(20_752);
        bests.submit_daily(first, record(900, 120));
        assert_eq!(bests.daily(next), None);
        assert!(bests.submit_daily(next, record(100, 120)));
        assert_eq!(bests.daily(first), None);
        assert_eq!(bests.daily(next), Some(record(100, 120)));
    }

    #[test]
    fn records_round_trip_through_a_file() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join(RECORDS_FILE);
        let mut bests = PersonalBests::default();
        bests.submit(Mode::Marathon, record(1200, 80));
        bests.submit(Mode::Sprint, record(700, 55));
        bests.submit(Mode::Time, record(800, 120));
        bests.submit(Mode::Classic, record(900, 70));
        bests.submit_daily(DailyDate::from_days(20_751), record(1300, 120));
        save_file(&path, &bests).expect("records save");
        assert_eq!(load_file(&path), Some(bests));
    }

    #[test]
    fn records_without_daily_data_still_load() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join(RECORDS_FILE);
        fs::write(
            &path,
            "version = 1\n[marathon]\nscore = 900\nlines = 20\nlevel = 2\nmillis = 60000\n",
        )
        .expect("write legacy records");
        let bests = load_file(&path).expect("legacy records load");
        assert_eq!(bests.best(Mode::Marathon).map(Record::score), Some(900));
        assert_eq!(bests.daily(DailyDate::from_days(20_751)), None);
    }

    #[test]
    fn legacy_ultra_records_load_as_time() {
        let directory = tempdir().expect("temporary directory");
        let path = directory.path().join(RECORDS_FILE);
        fs::write(
            &path,
            "version = 1\n[ultra]\nscore = 1500\nlines = 20\nlevel = 2\nmillis = 120000\n",
        )
        .expect("write legacy records");
        let bests = load_file(&path).expect("legacy records load");
        assert_eq!(bests.best(Mode::Time).map(Record::score), Some(1500));
        assert_eq!(bests.best(Mode::Classic), None);
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
