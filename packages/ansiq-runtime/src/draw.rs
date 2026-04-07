use ansiq_core::{Alignment, ElementKind, Node, Rect, Style};
use ansiq_render::FrameBuffer;

use crate::draw_border::draw_border;
use crate::draw_common::{
    border_style, fill_surface, fill_surface_in_regions, intersects_any, single_title, title_style,
};
use crate::draw_cursor::find_cursor_position;
use crate::draw_scrollbar::draw_scrollbar;
use crate::draw_table::draw_table;
use crate::draw_text::{draw_paragraph, draw_rich_text, draw_scroll_text, draw_text, text_content};
use crate::draw_widgets::{
    draw_bar_chart, draw_canvas, draw_chart, draw_gauge, draw_line_gauge, draw_list, draw_monthly,
    draw_sparkline, draw_tabs,
};

pub fn draw_tree<Message>(tree: &Node<Message>, focused: Option<usize>, buffer: &mut FrameBuffer) {
    buffer.fill_rect(
        Rect::new(0, 0, buffer.width(), buffer.height()),
        ' ',
        tree.element.style,
    );
    draw_node(tree, focused, buffer);
}

pub fn draw_tree_in_regions<Message>(
    tree: &Node<Message>,
    focused: Option<usize>,
    buffer: &mut FrameBuffer,
    regions: &[Rect],
) {
    if regions.is_empty() {
        return;
    }

    for region in regions {
        if let Some(clip) = region.intersection(Rect::new(0, 0, buffer.width(), buffer.height())) {
            buffer.fill_rect(clip, ' ', tree.element.style);
        }
    }

    draw_node_in_regions(tree, focused, buffer, regions);
}

pub fn cursor_position<Message>(
    tree: &Node<Message>,
    focused: Option<usize>,
) -> Option<(u16, u16)> {
    let focused = focused?;
    find_cursor_position(tree, focused)
}

fn draw_node<Message>(node: &Node<Message>, focused: Option<usize>, buffer: &mut FrameBuffer) {
    draw_node_impl(node, focused, buffer, None);
}

fn draw_node_in_regions<Message>(
    node: &Node<Message>,
    focused: Option<usize>,
    buffer: &mut FrameBuffer,
    regions: &[Rect],
) {
    draw_node_impl(node, focused, buffer, Some(regions));
}

fn draw_node_impl<Message>(
    node: &Node<Message>,
    focused: Option<usize>,
    buffer: &mut FrameBuffer,
    regions: Option<&[Rect]>,
) {
    if regions.is_some_and(|regions| !intersects_any(node.rect, regions)) {
        return;
    }

    match &node.element.kind {
        ElementKind::Box(_) | ElementKind::Shell(_) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_children(node, focused, buffer, regions);
        }
        ElementKind::Component(_) => {
            draw_children(node, focused, buffer, regions);
        }
        ElementKind::Pane(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            let titles = single_title(props.title.as_deref());
            draw_border(
                buffer,
                node.rect,
                &titles,
                Alignment::Left,
                ansiq_core::TitlePosition::Top,
                ansiq_core::Borders::ALL,
                ansiq_core::BorderType::Rounded,
                None,
                border_style(node.element.style, Style::default(), false),
                title_style(node.element.style, Style::default()),
            );
            draw_children(node, focused, buffer, regions);
        }
        ElementKind::Block(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            if !props.borders.is_empty() || !props.titles.is_empty() {
                draw_border(
                    buffer,
                    node.rect,
                    &props.titles,
                    props.title_alignment,
                    props.title_position,
                    props.borders,
                    props.border_type,
                    props.border_set,
                    border_style(node.element.style, props.border_style, false),
                    title_style(node.element.style, props.title_style),
                );
            }
            draw_children(node, focused, buffer, regions);
        }
        ElementKind::ScrollView(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            if let Some(child) = node.children.first() {
                if let Some((content, style)) = text_content(child) {
                    draw_scroll_text(
                        buffer,
                        node.rect,
                        &content,
                        style,
                        props.follow_bottom,
                        props.offset,
                    );
                    return;
                }
            }

            draw_children(node, focused, buffer, regions);
        }
        ElementKind::StreamingText(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_text(buffer, node.rect, &props.content, node.element.style);
        }
        ElementKind::Text(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_text(buffer, node.rect, &props.content, node.element.style);
        }
        ElementKind::Paragraph(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_paragraph(buffer, node.rect, props, node.element.style);
        }
        ElementKind::RichText(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_rich_text(buffer, node.rect, &props.block);
        }
        ElementKind::List(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_list(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Tabs(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_tabs(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Gauge(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_gauge(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Clear(_) => {
            clear_node_surface(buffer, node.rect, regions);
        }
        ElementKind::LineGauge(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_line_gauge(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Table(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_table(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Sparkline(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_sparkline(buffer, node.rect, props, node.element.style);
        }
        ElementKind::BarChart(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_bar_chart(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Chart(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_chart(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Canvas(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_canvas(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Monthly(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_monthly(buffer, node.rect, props, node.element.style);
        }
        ElementKind::Scrollbar(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_scrollbar(buffer, node.rect, props, node.element.style);
        }
        ElementKind::StatusBar(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            draw_text(buffer, node.rect, &props.content, node.element.style);
        }
        ElementKind::Input(props) => {
            fill_node_surface(buffer, node.rect, node.element.style, regions);
            let titles = Vec::new();
            draw_border(
                buffer,
                node.rect,
                &titles,
                Alignment::Left,
                ansiq_core::TitlePosition::Top,
                ansiq_core::Borders::ALL,
                ansiq_core::BorderType::Rounded,
                None,
                border_style(
                    node.element.style,
                    Style::default(),
                    focused == Some(node.id),
                ),
                node.element.style,
            );
            let inner = node.rect.shrink(1);
            let content = if props.value.is_empty() {
                &props.placeholder
            } else {
                &props.value
            };
            let content_style = if props.value.is_empty() {
                node.element.style.fg(ansiq_core::Color::Grey)
            } else {
                node.element.style
            };
            draw_text(buffer, inner, content, content_style);
        }
    }
}

fn draw_children<Message>(
    node: &Node<Message>,
    focused: Option<usize>,
    buffer: &mut FrameBuffer,
    regions: Option<&[Rect]>,
) {
    for child in &node.children {
        match regions {
            Some(regions) => draw_node_in_regions(child, focused, buffer, regions),
            None => draw_node(child, focused, buffer),
        }
    }
}

fn fill_node_surface(buffer: &mut FrameBuffer, rect: Rect, style: Style, regions: Option<&[Rect]>) {
    match regions {
        Some(regions) => fill_surface_in_regions(buffer, rect, style, regions),
        None => fill_surface(buffer, rect, style),
    }
}

fn clear_node_surface(buffer: &mut FrameBuffer, rect: Rect, regions: Option<&[Rect]>) {
    match regions {
        Some(regions) => {
            for region in regions {
                if let Some(clip) = rect.intersection(*region) {
                    buffer.fill_rect(clip, ' ', Style::default());
                }
            }
        }
        None => buffer.fill_rect(rect, ' ', Style::default()),
    }
}
