#[cfg(unix)]
use chrono::offset::TimeZone;
#[cfg(unix)]
use chrono::Local;
#[cfg(unix)]
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

#[cfg(unix)]
use std::{path, process};

#[cfg(unix)]
fn verify_against_date_command_local(path: &'static str, dt: NaiveDateTime) {
    let output = process::Command::new(path)
        .arg("-d")
        .arg(format!("{}-{:02}-{:02} {:02}:05:01", dt.year(), dt.month(), dt.day(), dt.hour()))
        .arg("+%Y-%m-%d %H:%M:%S %:z")
        .output()
        .unwrap();

    let date_command_str = String::from_utf8(output.stdout).unwrap();

    // The below would be preferred. At this stage neither earliest() or latest()
    // seems to be consistent with the output of the `date` command, so we simply
    // compare both.
    // let local = Local
    //     .with_ymd_and_hms(year, month, day, hour, 5, 1)
    //     // looks like the "date" command always returns a given time when it is ambiguous
    //     .earliest();

    // if let Some(local) = local {
    //     assert_eq!(format!("{}\n", local), date_command_str);
    // } else {
    //     // we are in a "Spring forward gap" due to DST, and so date also returns ""
    //     assert_eq!("", date_command_str);
    // }

    // This is used while a decision is made wheter the `date` output needs to
    // be exactly matched, or whether LocalResult::Ambigious should be handled
    // differently

    let date = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day()).unwrap();
    match Local.from_local_datetime(&date.and_hms_opt(dt.hour(), 5, 1).unwrap()) {
        chrono::LocalResult::Ambiguous(a, b) => assert!(
            format!("{}\n", a) == date_command_str || format!("{}\n", b) == date_command_str
        ),
        chrono::LocalResult::Single(a) => {
            assert_eq!(format!("{}\n", a), date_command_str);
        }
        chrono::LocalResult::None => {
            assert_eq!("", date_command_str);
        }
    }
}

/// path to Unix `date` command. Should work on most Linux and Unixes. Not the
/// path for MacOS (/bin/date) which uses a different version of `date` with
/// different arguments (so it won't run which is okay).
/// for testing only
#[allow(dead_code)]
#[cfg(not(target_os = "aix"))]
const DATE_PATH: &str = "/usr/bin/date";
#[allow(dead_code)]
#[cfg(target_os = "aix")]
const DATE_PATH: &str = "/opt/freeware/bin/date";

#[cfg(test)]
#[cfg(unix)]
/// test helper to sanity check the date command behaves as expected
/// asserts the command succeeded
fn assert_run_date_version() {
    // note environment variable `LANG`
    match std::env::var_os("LANG") {
        Some(lang) => eprintln!("LANG: {:?}", lang),
        None => eprintln!("LANG not set"),
    }
    let out = process::Command::new(DATE_PATH).arg("--version").output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    // note the `date` binary version
    eprintln!("command: {:?} --version\nstdout: {:?}\nstderr: {:?}", DATE_PATH, stdout, stderr);
    assert!(out.status.success(), "command failed: {:?} --version", DATE_PATH);
}

#[test]
#[cfg(unix)]
fn try_verify_against_date_command() {
    if !path::Path::new(DATE_PATH).exists() {
        eprintln!("date command {:?} not found, skipping", DATE_PATH);
        return;
    }
    assert_run_date_version();

    let mut date = NaiveDate::from_ymd_opt(1975, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();

    eprintln!(
        "Run command {:?} for every hour from {} to 2077, skipping some years...",
        DATE_PATH,
        date.year()
    );
    let mut count: u64 = 0;
    let mut year_at = date.year();
    while date.year() < 2078 {
        if (1975..=1977).contains(&date.year())
            || (2020..=2022).contains(&date.year())
            || (2073..=2077).contains(&date.year())
        {
            if date.year() != year_at {
                eprintln!("at year {}...", date.year());
                year_at = date.year();
            }
            verify_against_date_command_local(DATE_PATH, date);
            count += 1;
        }

        date += chrono::TimeDelta::hours(1);
    }
    eprintln!("Command {:?} was run {} times", DATE_PATH, count);
}

#[cfg(target_os = "linux")]
fn verify_against_date_command_format_local(path: &'static str, dt: NaiveDateTime) {
    let required_format =
        "d%d D%D F%F H%H I%I j%j k%k l%l m%m M%M S%S T%T u%u U%U w%w W%W X%X y%y Y%Y z%:z";
    // a%a - depends from localization
    // A%A - depends from localization
    // b%b - depends from localization
    // B%B - depends from localization
    // h%h - depends from localization
    // c%c - depends from localization
    // p%p - depends from localization
    // r%r - depends from localization
    // x%x - fails, date is dd/mm/yyyy, chrono is dd/mm/yy, same as %D
    // Z%Z - too many ways to represent it, will most likely fail

    let output = process::Command::new(path)
        .env("LANG", "c")
        .arg("-d")
        .arg(format!(
            "{}-{:02}-{:02} {:02}:{:02}:{:02}",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        ))
        .arg(format!("+{}", required_format))
        .output()
        .unwrap();

    let date_command_str = String::from_utf8(output.stdout).unwrap();
    let date = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day()).unwrap();
    let ldt = Local
        .from_local_datetime(&date.and_hms_opt(dt.hour(), dt.minute(), dt.second()).unwrap())
        .unwrap();
    let formated_date = format!("{}\n", ldt.format(required_format));
    assert_eq!(date_command_str, formated_date);
}

#[test]
#[cfg(target_os = "linux")]
fn try_verify_against_date_command_format() {
    if !path::Path::new(DATE_PATH).exists() {
        eprintln!("date command {:?} not found, skipping", DATE_PATH);
        return;
    }
    assert_run_date_version();

    let mut date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(12, 11, 13).unwrap();
    while date.year() < 2008 {
        verify_against_date_command_format_local(DATE_PATH, date);
        date += chrono::TimeDelta::days(55);
    }
}
