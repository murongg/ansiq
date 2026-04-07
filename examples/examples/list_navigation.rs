use ansiq_examples::{
    run_example_app,
    scenarios::list_navigation::{ListNavigationApp, VIEWPORT_POLICY},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    run_example_app(ListNavigationApp, VIEWPORT_POLICY).await
}
