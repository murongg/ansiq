use ansiq_examples::{
    run_example_app,
    scenarios::table_interaction::{TableInteractionApp, VIEWPORT_POLICY},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    run_example_app(TableInteractionApp, VIEWPORT_POLICY).await
}
