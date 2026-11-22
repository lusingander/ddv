mod support;

use anyhow::Result;

use support::{table_list_ready, Database, TestApp};

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
