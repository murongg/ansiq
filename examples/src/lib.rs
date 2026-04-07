use std::io;

use ansiq_runtime::{App, ViewportPolicy, run_app_with_policy};

pub mod activity_monitor;
pub mod openapi_explorer;
pub mod scenarios;
pub mod widget_gallery;

pub async fn run_example_app<A: App>(app: A, policy: ViewportPolicy) -> io::Result<()> {
    run_app_with_policy(app, policy).await
}
