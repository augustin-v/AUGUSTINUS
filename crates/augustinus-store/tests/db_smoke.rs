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

