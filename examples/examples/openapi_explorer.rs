use ansiq_examples::{run_example_app, scenarios::openapi_explorer::OpenApiExplorerApp};
use ansiq_runtime::ViewportPolicy;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut input = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--input" => input = args.next(),
            _ => {}
        }
    }

    let input = input.unwrap_or_else(|| {
        eprintln!(
            "Usage: cargo run -p ansiq-examples --example openapi_explorer -- --input <PATH|URL>"
        );
        std::process::exit(2);
    });

    let (label, text) = ansiq_examples::openapi_explorer::load_source(&input)
        .await
        .map_err(std::io::Error::other)?;
    let app = OpenApiExplorerApp::from_spec_text(&label, &text).map_err(std::io::Error::other)?;

    run_example_app(app, ViewportPolicy::ReserveFitContent { min: 22, max: 32 }).await
}
