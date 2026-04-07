use ansiq_examples::{
    run_example_app,
    scenarios::scroll_sync::{ScrollSyncApp, VIEWPORT_POLICY},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    run_example_app(ScrollSyncApp, VIEWPORT_POLICY).await
}
