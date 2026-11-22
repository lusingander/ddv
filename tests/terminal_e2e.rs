mod support;

use anyhow::Result;
use rstest::rstest;
use termlens::Key;

use support::{
    at_item, at_table, filter_input, footer_contains, item_rows, selected_item, selected_table,
    table_list_ready, table_names, Database, TestApp,
};

// Keep runtime workers available while termlens waits synchronously for the PTY.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn starts_with_a_table_list_and_restores_the_terminal_on_quit() -> Result<()> {
    let database = Database::new(&["ddv-alpha", "ddv-beta"]).await?;
    let mut app = TestApp::spawn(&database.endpoint)?;

    let screen = app.terminal.snapshot_after(|screen| {
        table_list_ready(screen, "ddv-alpha") && screen.contains("ddv-beta")
    })?;
    assert!(screen.contains("pk (HASH)"), "{screen}");
    assert!(screen.contains("ACTIVE"), "{screen}");
    assert!(screen.alternate_screen(), "{screen}");

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn opens_the_selected_table_and_item_and_restores_selection_when_returning() -> Result<()> {
    let database = Database::new(&["ddv-alpha", "ddv-beta"]).await?;
    database
        .put_item("ddv-alpha", "alpha01", "alpha value")
        .await?;
    database
        .put_item("ddv-beta", "beta01", "first beta value")
        .await?;
    database
        .put_item("ddv-beta", "beta02", "chosen beta value")
        .await?;
    let mut app = TestApp::spawn(&database.endpoint)?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-alpha"))?;

    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-beta"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal
        .wait_until(|screen| at_table(screen, "ddv-beta") && selected_item(screen, "beta01"))?;

    app.terminal.send(Key::Char('j'))?;
    app.terminal
        .wait_until(|screen| selected_item(screen, "beta02"))?;
    app.terminal.send(Key::Enter)?;
    let item = app.terminal.snapshot_after(|screen| {
        at_item(screen, "ddv-beta", "beta02") && screen.contains("chosen beta value")
    })?;
    assert!(!item.contains("first beta value"), "{item}");
    assert!(!item.contains("alpha value"), "{item}");

    app.terminal.send(Key::Backspace)?;
    app.terminal.wait_until(|screen| {
        at_table(screen, "ddv-beta") && selected_item(screen, "beta02") && screen.contains("beta01")
    })?;
    app.terminal.send(Key::Backspace)?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-beta"))?;

    app.terminal.send(Key::Char('k'))?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-alpha"))?;
    app.terminal.send(Key::Enter)?;
    let other_table = app.terminal.snapshot_after(|screen| {
        at_table(screen, "ddv-alpha")
            && selected_item(screen, "alpha01")
            && screen.contains("alpha value")
    })?;
    assert!(!other_table.contains("beta01"), "{other_table}");
    assert!(!other_table.contains("beta02"), "{other_table}");

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filters_tables_opens_the_match_and_clears_the_filter_after_returning() -> Result<()> {
    let database = Database::new(&["ddv-alpha", "ddv-beta", "ddv-gamma"]).await?;
    database
        .put_item("ddv-alpha", "alpha01", "excluded table")
        .await?;
    database
        .put_item("ddv-beta", "beta01", "chosen table")
        .await?;
    let mut app = TestApp::spawn(&database.endpoint)?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-alpha"))?;

    app.terminal.send(Key::Char('/'))?;
    app.terminal.wait_until(|screen| filter_input(screen, ""))?;
    app.terminal.send(Key::Char('b'))?;
    // Filtering can fetch a newly selected table's description. Wait before typing on.
    app.terminal
        .wait_until(|screen| filter_input(screen, "b") && table_list_ready(screen, "ddv-beta"))?;
    app.terminal.send_str("eta")?;
    app.terminal.wait_until(|screen| {
        filter_input(screen, "beta") && table_list_ready(screen, "ddv-beta")
    })?;
    app.terminal.send(Key::Enter)?;
    let filtered = app.terminal.snapshot_after(|screen| {
        table_list_ready(screen, "ddv-beta") && footer_contains(screen, "Clear filter")
    })?;
    assert_eq!(table_names(&filtered), ["ddv-beta"], "{filtered}");

    app.terminal.send(Key::Enter)?;
    app.terminal.wait_until(|screen| {
        at_table(screen, "ddv-beta")
            && selected_item(screen, "beta01")
            && screen.contains("chosen table")
    })?;
    app.terminal.send(Key::Backspace)?;
    let restored = app.terminal.snapshot_after(|screen| {
        table_list_ready(screen, "ddv-beta") && footer_contains(screen, "Clear filter")
    })?;
    assert_eq!(table_names(&restored), ["ddv-beta"], "{restored}");

    app.terminal.send(Key::Esc)?;
    let cleared = app.terminal.snapshot_after(|screen| {
        table_list_ready(screen, "ddv-beta") && !footer_contains(screen, "Clear filter")
    })?;
    assert_eq!(
        table_names(&cleared),
        ["ddv-alpha", "ddv-beta", "ddv-gamma"],
        "{cleared}"
    );

    app.quit()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn filters_items_by_a_value_and_restores_the_filter_and_selection_after_details() -> Result<()>
{
    let database = Database::new(&["ddv-items"]).await?;
    for (key, value) in [
        ("row01", "first haystack"),
        ("row02", "needle"),
        ("row03", "last haystack"),
    ] {
        database.put_item("ddv-items", key, value).await?;
    }
    let mut app = TestApp::spawn(&database.endpoint)?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-items"))?;
    app.terminal.send(Key::Enter)?;
    app.terminal
        .wait_until(|screen| at_table(screen, "ddv-items") && selected_item(screen, "row01"))?;

    app.terminal.send(Key::Char('/'))?;
    app.terminal.wait_until(|screen| filter_input(screen, ""))?;
    app.terminal.send_str("needle")?;
    app.terminal
        .wait_until(|screen| filter_input(screen, "needle") && selected_item(screen, "row02"))?;
    app.terminal.send(Key::Enter)?;
    let filtered = app.terminal.snapshot_after(|screen| {
        at_table(screen, "ddv-items")
            && selected_item(screen, "row02")
            && footer_contains(screen, "Clear filter")
    })?;
    assert_eq!(item_rows(&filtered).len(), 1, "{filtered}");
    assert!(!filtered.contains("haystack"), "{filtered}");

    app.terminal.send(Key::Enter)?;
    app.terminal
        .wait_until(|screen| at_item(screen, "ddv-items", "row02") && screen.contains("needle"))?;
    app.terminal.send(Key::Backspace)?;
    let restored = app.terminal.snapshot_after(|screen| {
        at_table(screen, "ddv-items")
            && selected_item(screen, "row02")
            && footer_contains(screen, "Clear filter")
    })?;
    assert_eq!(item_rows(&restored), item_rows(&filtered), "{restored}");

    app.terminal.send(Key::Esc)?;
    let cleared = app.terminal.snapshot_after(|screen| {
        at_table(screen, "ddv-items")
            && selected_item(screen, "row02")
            && !footer_contains(screen, "Clear filter")
    })?;
    assert!(cleared.row_text(0).contains("2 / 3"), "{cleared}");
    assert!(cleared.contains("last haystack"), "{cleared}");

    // Clearing keeps the selected row at the same screen position.
    app.terminal.send(Key::Char('g'))?;
    let full_list = app.terminal.snapshot_after(|screen| {
        at_table(screen, "ddv-items")
            && selected_item(screen, "row01")
            && !footer_contains(screen, "Clear filter")
    })?;
    assert_eq!(item_rows(&full_list).len(), 3, "{full_list}");
    assert!(full_list.contains("first haystack"), "{full_list}");
    assert!(full_list.contains("needle"), "{full_list}");
    assert!(full_list.contains("last haystack"), "{full_list}");

    app.quit()
}

#[rstest]
#[case::table_list(false)]
#[case::items(true)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resets_a_filter_with_no_matches_on_confirm_and_can_open_an_item(
    #[case] items: bool,
) -> Result<()> {
    let database = Database::new(&["ddv-alpha", "ddv-beta"]).await?;
    database
        .put_item("ddv-alpha", "alpha01", "alpha value")
        .await?;
    database
        .put_item("ddv-beta", "beta01", "beta value")
        .await?;
    let mut app = TestApp::spawn(&database.endpoint)?;
    app.terminal
        .wait_until(|screen| table_list_ready(screen, "ddv-alpha"))?;
    if items {
        app.terminal.send(Key::Enter)?;
        app.terminal.wait_until(|screen| {
            at_table(screen, "ddv-alpha") && selected_item(screen, "alpha01")
        })?;
    }

    app.terminal.send(Key::Char('/'))?;
    app.terminal.wait_until(|screen| filter_input(screen, ""))?;
    app.terminal.send_str("no-match")?;
    let empty = app
        .terminal
        .snapshot_after(|screen| filter_input(screen, "no-match"))?;
    if items {
        assert!(item_rows(&empty).is_empty(), "{empty}");
    } else {
        assert!(table_names(&empty).is_empty(), "{empty}");
    }

    // DDV clears a filter with no matches when it is confirmed.
    app.terminal.send(Key::Enter)?;
    if items {
        app.terminal.wait_until(|screen| {
            at_table(screen, "ddv-alpha")
                && selected_item(screen, "alpha01")
                && footer_contains(screen, "Filter")
                && !footer_contains(screen, "Clear filter")
        })?;
        app.terminal.send(Key::Enter)?;
        app.terminal.wait_until(|screen| {
            at_item(screen, "ddv-alpha", "alpha01") && screen.contains("alpha value")
        })?;
    } else {
        app.terminal.wait_until(|screen| {
            selected_table(screen, "ddv-alpha")
                && table_names(screen) == ["ddv-alpha", "ddv-beta"]
                && footer_contains(screen, "Filter")
                && !footer_contains(screen, "Clear filter")
        })?;
        app.terminal.send(Key::Char('j'))?;
        app.terminal
            .wait_until(|screen| table_list_ready(screen, "ddv-beta"))?;
        app.terminal.send(Key::Enter)?;
        app.terminal
            .wait_until(|screen| at_table(screen, "ddv-beta") && selected_item(screen, "beta01"))?;
        app.terminal.send(Key::Enter)?;
        app.terminal.wait_until(|screen| {
            at_item(screen, "ddv-beta", "beta01") && screen.contains("beta value")
        })?;
    }

    app.quit()
}
