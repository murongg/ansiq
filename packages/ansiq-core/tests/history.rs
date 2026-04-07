use ansiq_core::{
    Color, HistoryBlock, HistoryEntry, HistoryLine, HistoryRun, Style, history_block_from_text,
};

fn flatten_block(block: &HistoryBlock) -> Vec<String> {
    block
        .lines
        .iter()
        .map(|line| line.runs.iter().map(|run| run.text.as_str()).collect())
        .collect()
}

#[test]
fn history_block_from_text_wraps_plain_text_into_history_lines() {
    let block = history_block_from_text("wrapped", 4);

    assert_eq!(
        flatten_block(&block),
        vec!["wrap".to_string(), "ped".to_string()]
    );
}

#[test]
fn history_block_from_text_preserves_explicit_blank_lines() {
    let block = history_block_from_text("top\n\nbottom", 80);

    assert_eq!(
        flatten_block(&block),
        vec!["top".to_string(), "".to_string(), "bottom".to_string()]
    );
}

#[test]
fn history_entry_text_and_block_share_the_same_commit_time_wrapping_model() {
    let wrapped = history_block_from_text("tail", 80);
    let block_entry = HistoryEntry::Block(HistoryBlock {
        lines: vec![HistoryLine {
            runs: vec![HistoryRun {
                text: "tail".to_string(),
                style: Style::default().fg(Color::Cyan),
            }],
        }],
    });

    let text_block = match HistoryEntry::Text("tail".to_string()) {
        HistoryEntry::Text(text) => history_block_from_text(&text, 80),
        HistoryEntry::Block(_) => unreachable!(),
    };

    assert_eq!(flatten_block(&text_block), flatten_block(&wrapped));
    match block_entry {
        HistoryEntry::Block(block) => assert_eq!(flatten_block(&block), vec!["tail".to_string()]),
        HistoryEntry::Text(_) => unreachable!(),
    }
}
