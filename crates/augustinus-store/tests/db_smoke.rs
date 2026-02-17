use chrono::Local;

use augustinus_store::db::Store;

#[test]
fn creates_schema_and_inserts_event() {
    let store = Store::open_in_memory().unwrap();
    store.insert_event("focus_start", "{}").unwrap();
    let n = store.count_events().unwrap();
    assert_eq!(n, 1);
}

#[test]
fn upserts_daily_focus_seconds() {
    let store = Store::open_in_memory().unwrap();
    let today = Local::now().date_naive();
    store.add_focus_seconds_for_day(today, 7).unwrap();
    store.add_focus_seconds_for_day(today, 3).unwrap();
    let seconds = store.focus_seconds_for_day(today).unwrap();
    assert_eq!(seconds, 10);
}

#[test]
fn streak_counts_consecutive_days_ending_today() {
    let store = Store::open_in_memory().unwrap();
    let today = Local::now().date_naive();
    let yesterday = today.pred_opt().unwrap();
    let two_days_ago = yesterday.pred_opt().unwrap();

    store.add_focus_seconds_for_day(today, 1).unwrap();
    store.add_focus_seconds_for_day(yesterday, 1).unwrap();
    store.add_focus_seconds_for_day(two_days_ago, 0).unwrap();

    let streak = store.streak_days_ending_today().unwrap();
    assert_eq!(streak, 2);
}
