use clap::{Args, Parser, Subcommand};
use chrono::NaiveDate;
use rusqlite::Connection;

#[derive(Args)]
struct Report {
    year: i32,
    month: u32,
}

#[derive(Args, Debug)]
struct Job {
    repo: String,
    branch: String,
}

#[derive(Subcommand)]
enum Command {
    /// Starts the working day
    Start,
    /// Ends the working day
    End,
    /// Adds a job to the timesheet
    Job(Job),
    /// Emits a monthly report
    Report(Report),
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command
}

#[derive(Clone)]
struct Entry {
    repo: String,
    branch: String,
    start: u64,
    duration: u64,
}

fn last_entry(conn: &Connection) -> Entry {
    let mut stmt = conn.prepare("select * from times order by time desc limit 1;").unwrap();
    let mut iter = stmt.query_map([], |row| {
        Ok(Entry {
            repo: row.get(1).unwrap_or_else(|_| "nothing".to_string()),
            branch: row.get(2).unwrap_or_else(|_| "nothing".to_string()),
            start: row.get(3).unwrap(),
            duration: row.get(4).unwrap_or(0),
        })
    }).unwrap();
    iter.next().unwrap().unwrap()
}

fn add_time(conn: &Connection, entry: Entry) {
    conn.execute(
        "insert into times (repo, branch, time, duration) values (?,?,?,?)",
        [entry.repo, entry.branch, format!("{}", entry.start), format!("{}", entry.duration)].map(|n| n.to_string()),
    ).unwrap();
}

fn main() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let cli = Cli::parse();
    let connection = rusqlite::Connection::open_with_flags("/home/git/timesheets.sql", Default::default()).unwrap();
    let query = "create table if not exists times (id integer primary key, repo text, branch text, time integer, duration integer);";
    connection.execute(query, ()).unwrap();

    match cli.command {
        Command::Start => {
            connection.execute(
            "insert into times (time) values (?);",
            (since_the_epoch,)
            ).unwrap();
        }
        Command::End => {
            let timestamp = last_entry(&connection);
            let mut entry = timestamp.clone();
            entry.start = since_the_epoch;
            entry.duration = since_the_epoch - timestamp.start;
            println!("Adding {} seconds to {:?}", entry.duration, Job{repo: entry.repo.clone(), branch: entry.branch.clone()});
            add_time(&connection, entry);
        }
        Command::Job(job) => {
            let timestamp = last_entry(&connection);
            let mut entry = timestamp.clone();
            let duration = since_the_epoch - timestamp.start;
            println!("Adding {} seconds to {:?}", duration, &job);
            entry.repo = job.repo;
            entry.branch = job.branch;
            entry.start = since_the_epoch;
            entry.duration = duration;
            add_time(&connection, entry);
        }
        Command::Report(Report { year, month }) => {
            let start_of_month = NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp();

            // Calculate the start of the following month
            let next_month = if month == 12 {
                // Wrap around to the next year if the month is December
                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
            } else {
                // Otherwise, just go to the next month
                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
            }
            .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp();

            let mut stmt = connection.prepare(&format!("select repo, branch, sum(duration) as total_duration from times where times.time > {start_of_month} and times.time < {next_month} group by repo, branch;")).unwrap();
            for entry in stmt.query_map([], |row| {
                Ok(Entry {
                    repo: row.get(0).unwrap_or_default(),
                    branch: row.get(1).unwrap_or_default(),
                    start: 0,
                    duration: row.get(2).unwrap_or_default(),
                })
            }).unwrap() {
                let entry = entry.unwrap();
            println!("{:?},{:?},{}", entry.repo, entry.branch, entry.duration);
            }
        }
    }
}
