use ansiq_core::{Element, ElementKind, Length, TranscriptEntry};
use ansiq_widgets::TranscriptView;

#[test]
fn transcript_view_builds_rich_text_from_entries() {
    let view: Element<()> = TranscriptView::new([
        TranscriptEntry::user("cargo test"),
        TranscriptEntry::assistant("Planning the change"),
    ])
    .build();

    match &view.kind {
        ElementKind::RichText(props) => {
            let rendered = props
                .block
                .lines
                .iter()
                .map(|line| {
                    line.runs
                        .iter()
                        .map(|run| run.text.as_str())
                        .collect::<String>()
                })
                .collect::<Vec<_>>();

            assert_eq!(
                rendered,
                vec![
                    "you  cargo test".to_string(),
                    "assistant  Planning the change".to_string(),
                ]
            );
        }
        other => panic!("expected rich text, got {other:?}"),
    }

    assert_eq!(view.layout.width, Length::Fill);
    assert_eq!(view.layout.height, Length::Auto);
}
