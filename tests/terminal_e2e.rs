mod support;

use anyhow::Result;
use termlens::Key;

use support::{at_item, at_table, selected_item, table_list_ready, Database, TestApp};

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
