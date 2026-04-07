use ansiq_examples::{
    run_example_app,
    scenarios::activity_monitor::{ActivityMonitorApp, VIEWPORT_POLICY},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    run_example_app(ActivityMonitorApp::default(), VIEWPORT_POLICY).await
}
