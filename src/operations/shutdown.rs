use std::time::Duration;

use crate::app::AppContext;

pub async fn execute(app: &AppContext) {
    let duration = Duration::from_secs(1);
    while !app.states.is_shutting_down() {
        tokio::time::sleep(duration).await;
    }

    print!("Stopping the application");
}
