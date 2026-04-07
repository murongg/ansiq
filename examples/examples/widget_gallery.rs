use ansiq_examples::{
    run_example_app,
    widget_gallery::{VIEWPORT_POLICY, WidgetGalleryApp, known_widgets},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut widget = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-w" | "--widget" => widget = args.next(),
            _ => {}
        }
    }

    let widget = widget.unwrap_or_else(|| {
        eprintln!(
            "Usage: cargo run -p ansiq-examples --example widget_gallery -- --widget <name>\nKnown widgets: {}",
            known_widgets().join(", ")
        );
        std::process::exit(2);
    });

    if !known_widgets().contains(&widget.as_str()) {
        eprintln!(
            "Unknown widget `{widget}`.\nKnown widgets: {}",
            known_widgets().join(", ")
        );
        std::process::exit(2);
    }

    run_example_app(WidgetGalleryApp::new(widget), VIEWPORT_POLICY).await
}
