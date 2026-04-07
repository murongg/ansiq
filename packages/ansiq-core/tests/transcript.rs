use ansiq_core::{Color, TranscriptEntry, TranscriptRole, TranscriptSession, transcript_block};

fn flatten_block(block: &ansiq_core::HistoryBlock) -> Vec<String> {
    block
        .lines
        .iter()
        .map(|line| {
            line.runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>()
        })
        .collect()
}

#[test]
fn transcript_block_formats_user_and_assistant_entries() {
    let entries = vec![
        TranscriptEntry::user("write tests"),
        TranscriptEntry::assistant("Planning the change\nInspecting the workspace..."),
    ];

    let block = transcript_block(&entries);
    let lines = flatten_block(&block);

    assert_eq!(
        lines,
        vec![
            "you  write tests".to_string(),
            "assistant  Planning the change".to_string(),
            "Inspecting the workspace...".to_string(),
        ]
    );
}

#[test]
fn transcript_block_formats_status_entries_without_role_prefix() {
    let entries = vec![TranscriptEntry::new(
        TranscriptRole::Status,
        "Streaming complete.",
    )];

    let block = transcript_block(&entries);
    let lines = flatten_block(&block);

    assert_eq!(lines, vec!["Streaming complete.".to_string()]);
    assert_eq!(block.lines[0].runs[0].style.fg, Color::DarkGrey);
}

#[test]
fn transcript_session_rolls_the_previous_turn_into_history_when_starting_a_new_one() {
    let mut session = TranscriptSession::default();

    assert!(!session.started());
    assert!(session.begin_turn("cargo test").is_none());
    session.append_assistant("Planning the change");
    session.append_assistant("\nStreaming complete.");

    let committed = session
        .begin_turn("cargo check")
        .expect("previous turn should be committed");
    let lines = flatten_block(&committed);

    assert!(session.started());
    assert_eq!(
        lines,
        vec![
            "you  cargo test".to_string(),
            "assistant  Planning the change".to_string(),
            "Streaming complete.".to_string(),
        ]
    );
    assert_eq!(
        session.entries(),
        &[
            TranscriptEntry::user("cargo check"),
            TranscriptEntry::assistant(""),
        ]
    );
}
