use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Expr, ExprLit, Ident, Lit, LitStr, Result, Token, braced,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as ViewRoot);
    root.expand().into()
}

struct ViewRoot {
    node: Node,
}

impl ViewRoot {
    fn expand(self) -> TokenStream2 {
        self.node.expand()
    }
}

impl Parse for ViewRoot {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            node: input.parse()?,
        })
    }
}

struct Node {
    name: Ident,
    attrs: Vec<Attribute>,
    children: Vec<Child>,
}

struct Attribute {
    name: Ident,
    value: Expr,
}

enum Child {
    Node(Node),
    Expr(Expr),
    Text(LitStr),
}

impl Parse for Node {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Token![<]>()?;
        let name: Ident = input.parse()?;
        let mut attrs = Vec::new();

        // Parse a deliberately small JSX-like surface: `name=value` pairs where
        // values are either normal Rust expressions or `{expr}` interpolations.
        while !(input.peek(Token![>]) || (input.peek(Token![/]) && input.peek2(Token![>]))) {
            let attr_name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                content.parse()?
            } else {
                Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: input.parse::<Lit>()?,
                })
            };
            attrs.push(Attribute {
                name: attr_name,
                value,
            });
        }

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(Self {
                name,
                attrs,
                children: Vec::new(),
            });
        }

        input.parse::<Token![>]>()?;
        let mut children = Vec::new();

        while !(input.peek(Token![<]) && input.peek2(Token![/])) {
            children.push(input.parse()?);
        }

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close_name: Ident = input.parse()?;
        if close_name != name {
            return Err(syn::Error::new(
                close_name.span(),
                format!("expected closing tag </{}>", name),
            ));
        }
        input.parse::<Token![>]>()?;

        Ok(Self {
            name,
            attrs,
            children,
        })
    }
}

impl Parse for Child {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Token![<]) {
            Ok(Self::Node(input.parse()?))
        } else if input.peek(LitStr) {
            Ok(Self::Text(input.parse()?))
        } else if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            Ok(Self::Expr(content.parse()?))
        } else {
            Err(input.error("expected a child element, string literal, or {expr}"))
        }
    }
}

impl Child {
    fn expand(self) -> TokenStream2 {
        match self {
            Self::Node(node) => node.expand(),
            Self::Expr(expr) => {
                let core = core_path();
                quote! { #core::IntoElement::into_element(#expr) }
            }
            Self::Text(text) => {
                let core = core_path();
                quote! { #core::Element::new_text(#text) }
            }
        }
    }
}

fn dependency_path(package: &str) -> TokenStream2 {
    match crate_name(package) {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => {
            let ident = Ident::new(&package.replace('-', "_"), Span::call_site());
            quote! { ::#ident }
        }
    }
}

fn ansiq_root() -> Option<TokenStream2> {
    match crate_name("ansiq") {
        Ok(FoundCrate::Itself) => Some(quote! { crate }),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Some(quote! { ::#ident })
        }
        Err(_) => None,
    }
}

fn core_path() -> TokenStream2 {
    if let Some(ansiq) = ansiq_root() {
        quote! { #ansiq::core }
    } else {
        dependency_path("ansiq-core")
    }
}

fn widgets_path() -> TokenStream2 {
    if let Some(ansiq) = ansiq_root() {
        quote! { #ansiq::widgets }
    } else {
        dependency_path("ansiq-widgets")
    }
}

impl Node {
    fn expand(self) -> TokenStream2 {
        match self.name.to_string().as_str() {
            "Box" => self.expand_box(),
            "Text" => self.expand_text(),
            "Paragraph" => self.expand_paragraph(),
            "RichText" => self.expand_rich_text(),
            "Pane" => self.expand_pane(),
            "Block" => self.expand_block(),
            "List" => self.expand_list(),
            "Tabs" => self.expand_tabs(),
            "Gauge" => self.expand_gauge(),
            "Clear" => self.expand_clear(),
            "LineGauge" => self.expand_line_gauge(),
            "Table" => self.expand_table(),
            "Sparkline" => self.expand_sparkline(),
            "BarChart" => self.expand_bar_chart(),
            "Chart" => self.expand_chart(),
            "Canvas" => self.expand_canvas(),
            "Monthly" => self.expand_monthly(),
            "ScrollView" => self.expand_scroll_view(),
            "Scrollbar" => self.expand_scrollbar(),
            "StreamingText" => self.expand_streaming_text(),
            "Input" => self.expand_input(),
            "StatusBar" => self.expand_status_bar(),
            _ => self.expand_custom_component(),
        }
    }

    fn expand_box(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let direction = attr_expr(&attrs, "direction")
            .and_then(expr_as_string)
            .unwrap_or_else(|| "column".to_string());
        let mut builder = match direction.as_str() {
            "row" => quote! { #widgets::Box::row() },
            _ => quote! { #widgets::Box::column() },
        };

        if let Some(gap) = attr_expr(&attrs, "gap") {
            builder = quote! { #builder .gap(#gap) };
        }

        for child in children {
            let child = child.expand();
            builder = quote! { #builder .child(#child) };
        }

        finish_element(builder, &attrs)
    }

    fn expand_text(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let content = content_expr(&attrs, &children).unwrap_or_else(|| quote! { "" });
        finish_element(quote! { #widgets::Text::new(#content) }, &attrs)
    }

    fn expand_paragraph(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let content = content_expr(&attrs, &children).unwrap_or_else(|| quote! { "" });
        let mut builder = quote! { #widgets::Paragraph::new(#content) };
        if let Some(alignment) = attr_expr(&attrs, "alignment") {
            builder = quote! { #builder .alignment(#alignment) };
        }
        if let Some(wrap) = attr_expr(&attrs, "wrap") {
            builder = quote! { #builder .wrap(#wrap) };
        }
        if let Some(block) = attr_expr(&attrs, "block") {
            builder = quote! { #builder .block(#block) };
        }
        if let Some(scroll) = attr_expr(&attrs, "scroll") {
            builder = quote! { #builder .scroll(#scroll) };
        } else if attr_expr(&attrs, "scroll_y").is_some() || attr_expr(&attrs, "scroll_x").is_some()
        {
            let scroll_y = attr_expr(&attrs, "scroll_y")
                .map(|expr| quote! { #expr })
                .unwrap_or_else(|| quote! { 0u16 });
            let scroll_x = attr_expr(&attrs, "scroll_x")
                .map(|expr| quote! { #expr })
                .unwrap_or_else(|| quote! { 0u16 });
            builder = quote! { #builder .scroll((#scroll_y, #scroll_x)) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_rich_text(self) -> TokenStream2 {
        let attrs = self.attrs;
        let core = core_path();
        let widgets = widgets_path();
        let block = attr_expr(&attrs, "block")
            .map(|expr| quote! { #expr })
            .unwrap_or_else(|| quote! { #core::HistoryBlock { lines: ::std::vec::Vec::new() } });
        finish_element(quote! { #widgets::RichText::new(#block) }, &attrs)
    }

    fn expand_pane(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::Pane::new() };
        if let Some(title) = attr_expr(&attrs, "title") {
            builder = quote! { #builder .title(#title) };
        }
        for child in children {
            let child = child.expand();
            builder = quote! { #builder .child(#child) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_block(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::Block::new() };
        if let Some(title) = attr_expr(&attrs, "title") {
            builder = quote! { #builder .title(#title) };
        }
        if let Some(title_top) = attr_expr(&attrs, "title_top") {
            builder = quote! { #builder .title_top(#title_top) };
        }
        if let Some(title_bottom) = attr_expr(&attrs, "title_bottom") {
            builder = quote! { #builder .title_bottom(#title_bottom) };
        }
        if let Some(title_alignment) = attr_expr(&attrs, "title_alignment") {
            builder = quote! { #builder .title_alignment(#title_alignment) };
        }
        if let Some(title_position) = attr_expr(&attrs, "title_position") {
            builder = quote! { #builder .title_position(#title_position) };
        }
        if let Some(bordered) = attr_expr(&attrs, "bordered") {
            builder = quote! { #builder .bordered_flag(#bordered) };
        }
        if let Some(borders) = attr_expr(&attrs, "borders") {
            builder = quote! { #builder .borders(#borders) };
        }
        if let Some(border_type) = attr_expr(&attrs, "border_type") {
            builder = quote! { #builder .border_type(#border_type) };
        }
        if let Some(border_set) = attr_expr(&attrs, "border_set") {
            builder = quote! { #builder .border_set(#border_set) };
        }
        if let Some(padding) = attr_expr(&attrs, "padding") {
            builder = quote! { #builder .padding(#padding) };
        }
        if let Some(border_style) = attr_expr(&attrs, "border_style") {
            builder = quote! { #builder .border_style(#border_style) };
        }
        if let Some(title_style) = attr_expr(&attrs, "title_style") {
            builder = quote! { #builder .title_style(#title_style) };
        }
        for child in children {
            let child = child.expand();
            builder = quote! { #builder .child(#child) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_list(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let items = attr_expr(&attrs, "items")
            .map(|expr| quote! { #expr })
            .unwrap_or_else(|| quote! { ::std::vec::Vec::<::std::string::String>::new() });
        let mut builder = quote! { #widgets::List::new(#items) };
        if let Some(items) = attr_expr(&attrs, "items") {
            builder = quote! { #builder .items(#items) };
        }
        if let Some(block) = attr_expr(&attrs, "block") {
            builder = quote! { #builder .block(#block) };
        }
        if let Some(selected) = attr_expr(&attrs, "selected") {
            builder = quote! { #builder .selected(#selected) };
        }
        if let Some(on_select) = attr_expr(&attrs, "on_select") {
            builder = quote! { #builder .on_select(#on_select) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_tabs(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let titles = attr_expr(&attrs, "titles")
            .map(|expr| quote! { #expr })
            .unwrap_or_else(|| quote! { ::std::vec::Vec::<::std::string::String>::new() });
        let mut builder = quote! { #widgets::Tabs::new(#titles) };
        if let Some(block) = attr_expr(&attrs, "block") {
            builder = quote! { #builder .block(#block) };
        }
        if let Some(titles) = attr_expr(&attrs, "titles") {
            builder = quote! { #builder .titles(#titles) };
        }
        if let Some(selected) = attr_expr(&attrs, "selected") {
            builder = quote! { #builder .selected(#selected) };
        }
        if let Some(highlight_style) = attr_expr(&attrs, "highlight_style") {
            builder = quote! { #builder .highlight_style(#highlight_style) };
        }
        if let Some(divider) = attr_expr(&attrs, "divider") {
            builder = quote! { #builder .divider(#divider) };
        }
        if let Some(padding_left) = attr_expr(&attrs, "padding_left") {
            builder = quote! { #builder .padding_left(#padding_left) };
        }
        if let Some(padding_right) = attr_expr(&attrs, "padding_right") {
            builder = quote! { #builder .padding_right(#padding_right) };
        }
        if let Some(on_select) = attr_expr(&attrs, "on_select") {
            builder = quote! { #builder .on_select(#on_select) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_gauge(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::Gauge::new() };
        if let Some(block) = attr_expr(&attrs, "block") {
            builder = quote! { #builder .block(#block) };
        }
        if let Some(ratio) = attr_expr(&attrs, "ratio") {
            builder = quote! { #builder .ratio(#ratio) };
        }
        if let Some(percent) = attr_expr(&attrs, "percent") {
            builder = quote! { #builder .percent(#percent) };
        }
        if let Some(label) = attr_expr(&attrs, "label") {
            builder = quote! { #builder .label(#label) };
        }
        if let Some(use_unicode) = attr_expr(&attrs, "use_unicode") {
            builder = quote! { #builder .use_unicode(#use_unicode) };
        }
        if let Some(gauge_style) = attr_expr(&attrs, "gauge_style") {
            builder = quote! { #builder .gauge_style(#gauge_style) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_clear(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        finish_element(quote! { #widgets::Clear::new() }, &attrs)
    }

    fn expand_line_gauge(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::LineGauge::new() };
        if let Some(block) = attr_expr(&attrs, "block") {
            builder = quote! { #builder .block(#block) };
        }
        if let Some(ratio) = attr_expr(&attrs, "ratio") {
            builder = quote! { #builder .ratio(#ratio) };
        }
        if let Some(percent) = attr_expr(&attrs, "percent") {
            builder = quote! { #builder .percent(#percent) };
        }
        if let Some(label) = attr_expr(&attrs, "label") {
            builder = quote! { #builder .label(#label) };
        }
        if let Some(line_set) = attr_expr(&attrs, "line_set") {
            builder = quote! { #builder .line_set(#line_set) };
        }
        if let Some(filled_symbol) = attr_expr(&attrs, "filled_symbol") {
            builder = quote! { #builder .filled_symbol(#filled_symbol) };
        }
        if let Some(unfilled_symbol) = attr_expr(&attrs, "unfilled_symbol") {
            builder = quote! { #builder .unfilled_symbol(#unfilled_symbol) };
        }
        if let Some(filled_style) = attr_expr(&attrs, "filled_style") {
            builder = quote! { #builder .filled_style(#filled_style) };
        }
        if let Some(unfilled_style) = attr_expr(&attrs, "unfilled_style") {
            builder = quote! { #builder .unfilled_style(#unfilled_style) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_table(self) -> TokenStream2 {
        let attrs = self.attrs;
        let core = core_path();
        let widgets = widgets_path();
        let rows = attr_expr(&attrs, "rows")
            .map(|expr| quote! { #expr })
            .unwrap_or_else(
                || quote! { ::std::vec::Vec::<::std::vec::Vec<::std::string::String>>::new() },
            );
        let widths = attr_expr(&attrs, "widths")
            .map(|expr| quote! { #expr })
            .unwrap_or_else(|| quote! { ::std::vec::Vec::<#core::Constraint>::new() });
        let mut builder = quote! { #widgets::Table::new(#rows, #widths) };
        if let Some(block) = attr_expr(&attrs, "block") {
            builder = quote! { #builder .block(#block) };
        }
        if let Some(header) = attr_expr(&attrs, "header") {
            builder = quote! { #builder .header(#header) };
        } else if let Some(headers) = attr_expr(&attrs, "headers") {
            builder = quote! { #builder .headers(#headers) };
        }
        if let Some(footer) = attr_expr(&attrs, "footer") {
            builder = quote! { #builder .footer(#footer) };
        }
        if let Some(widths) = attr_expr(&attrs, "widths") {
            builder = quote! { #builder .widths(#widths) };
        }
        if let Some(column_spacing) = attr_expr(&attrs, "column_spacing") {
            builder = quote! { #builder .column_spacing(#column_spacing) };
        }
        if let Some(flex) = attr_expr(&attrs, "flex") {
            builder = quote! { #builder .flex(#flex) };
        }
        if let Some(alignments) = attr_expr(&attrs, "alignments") {
            builder = quote! { #builder .alignments(#alignments) };
        }
        if let Some(selected) = attr_expr(&attrs, "selected") {
            builder = quote! { #builder .selected(#selected) };
        }
        if let Some(highlight_symbol) = attr_expr(&attrs, "highlight_symbol") {
            builder = quote! { #builder .highlight_symbol(#highlight_symbol) };
        }
        if let Some(on_select) = attr_expr(&attrs, "on_select") {
            builder = quote! { #builder .on_select(#on_select) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_sparkline(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::Sparkline::new() };
        if let Some(values) = attr_expr(&attrs, "values") {
            builder = quote! { #builder .values(#values) };
        }
        if let Some(max) = attr_expr(&attrs, "max") {
            builder = quote! { #builder .max(#max) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_bar_chart(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::BarChart::new() };
        if let Some(bars) = attr_expr(&attrs, "bars") {
            builder = quote! { #builder .bars(#bars) };
        }
        if let Some(max) = attr_expr(&attrs, "max") {
            builder = quote! { #builder .max(#max) };
        }
        if let Some(bar_width) = attr_expr(&attrs, "bar_width") {
            builder = quote! { #builder .bar_width(#bar_width) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_chart(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let datasets = attr_expr(&attrs, "datasets");
        let mut builder = quote! { #widgets::Chart::new() };
        if let Some(min_y) = attr_expr(&attrs, "min_y") {
            builder = quote! { #builder .min_y(#min_y) };
        }
        if let Some(max_y) = attr_expr(&attrs, "max_y") {
            builder = quote! { #builder .max_y(#max_y) };
        }
        let builder = if let Some(datasets) = datasets {
            quote! {{
                let mut chart = #builder;
                for dataset in #datasets {
                    chart = if let Some(label) = dataset.label {
                        chart.named_dataset(label, dataset.points)
                    } else {
                        chart.dataset(dataset.points)
                    };
                }
                chart
            }}
        } else {
            builder
        };
        finish_element(builder, &attrs)
    }

    fn expand_canvas(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let cells = attr_expr(&attrs, "cells");
        let mut builder = quote! { #widgets::Canvas::new() };
        if let Some(width) = attr_expr(&attrs, "width") {
            let height = attr_expr(&attrs, "height")
                .map(|expr| quote! { #expr })
                .unwrap_or_else(|| quote! { 8u16 });
            builder = quote! { #builder .size(#width, #height) };
        }
        let builder = if let Some(cells) = cells {
            quote! {{
                let mut canvas = #builder;
                for cell in #cells {
                    canvas = canvas.point(cell.x, cell.y, cell.symbol);
                }
                canvas
            }}
        } else {
            builder
        };
        finish_element(builder, &attrs)
    }

    fn expand_monthly(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::Monthly::new() };
        if let Some(year) = attr_expr(&attrs, "year") {
            builder = quote! { #builder .year(#year) };
        }
        if let Some(month) = attr_expr(&attrs, "month") {
            builder = quote! { #builder .month(#month) };
        }
        if let Some(selected_day) = attr_expr(&attrs, "selected_day") {
            builder = quote! { #builder .selected_day(#selected_day) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_scrollbar(self) -> TokenStream2 {
        let attrs = self.attrs;
        let core = core_path();
        let widgets = widgets_path();
        let orientation = attr_expr(&attrs, "orientation")
            .map(|expr| quote! { #expr })
            .unwrap_or_else(|| quote! { #core::ScrollbarOrientation::VerticalRight });
        let mut builder = quote! { #widgets::Scrollbar::new(#orientation) };
        if let Some(position) = attr_expr(&attrs, "position") {
            builder = quote! { #builder .position(#position) };
        }
        if let Some(content_length) = attr_expr(&attrs, "content_length") {
            builder = quote! { #builder .content_length(#content_length) };
        }
        if let Some(viewport_length) = attr_expr(&attrs, "viewport_length") {
            builder = quote! { #builder .viewport_length(#viewport_length) };
        }
        if let Some(viewport_content_length) = attr_expr(&attrs, "viewport_content_length") {
            builder = quote! { #builder .viewport_content_length(#viewport_content_length) };
        }
        if let Some(symbols) = attr_expr(&attrs, "symbols") {
            builder = quote! { #builder .symbols(#symbols) };
        }
        if let Some(thumb_symbol) = attr_expr(&attrs, "thumb_symbol") {
            builder = quote! { #builder .thumb_symbol(#thumb_symbol) };
        }
        if let Some(track_symbol) = attr_expr(&attrs, "track_symbol") {
            builder = quote! { #builder .track_symbol(#track_symbol) };
        }
        if let Some(begin_symbol) = attr_expr(&attrs, "begin_symbol") {
            builder = quote! { #builder .begin_symbol(#begin_symbol) };
        }
        if let Some(end_symbol) = attr_expr(&attrs, "end_symbol") {
            builder = quote! { #builder .end_symbol(#end_symbol) };
        }
        if let Some(thumb_style) = attr_expr(&attrs, "thumb_style") {
            builder = quote! { #builder .thumb_style(#thumb_style) };
        }
        if let Some(track_style) = attr_expr(&attrs, "track_style") {
            builder = quote! { #builder .track_style(#track_style) };
        }
        if let Some(begin_style) = attr_expr(&attrs, "begin_style") {
            builder = quote! { #builder .begin_style(#begin_style) };
        }
        if let Some(end_style) = attr_expr(&attrs, "end_style") {
            builder = quote! { #builder .end_style(#end_style) };
        }
        if let Some(on_scroll) = attr_expr(&attrs, "on_scroll") {
            builder = quote! { #builder .on_scroll(#on_scroll) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_scroll_view(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::ScrollView::new() };
        if let Some(follow_bottom) = attr_expr(&attrs, "follow_bottom") {
            builder = quote! { #builder .follow_bottom(#follow_bottom) };
        }
        if let Some(offset) = attr_expr(&attrs, "offset") {
            builder = quote! { #builder .offset(#offset) };
        }
        if let Some(on_scroll) = attr_expr(&attrs, "on_scroll") {
            builder = quote! { #builder .on_scroll(#on_scroll) };
        }
        for child in children {
            let child = child.expand();
            builder = quote! { #builder .child(#child) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_streaming_text(self) -> TokenStream2 {
        let attrs = self.attrs;
        let children = self.children;
        let widgets = widgets_path();
        let content = content_expr(&attrs, &children).unwrap_or_else(|| quote! { "" });
        finish_element(quote! { #widgets::StreamingText::new(#content) }, &attrs)
    }

    fn expand_input(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let mut builder = quote! { #widgets::Input::new() };
        if let Some(value) = attr_expr(&attrs, "value") {
            builder = quote! { #builder .value(#value) };
        }
        if let Some(placeholder) = attr_expr(&attrs, "placeholder") {
            builder = quote! { #builder .placeholder(#placeholder) };
        }
        if let Some(on_change) = attr_expr(&attrs, "on_change") {
            builder = quote! { #builder .on_change(#on_change) };
        }
        if let Some(on_submit) = attr_expr(&attrs, "on_submit") {
            builder = quote! { #builder .on_submit(#on_submit) };
        }
        finish_element(builder, &attrs)
    }

    fn expand_status_bar(self) -> TokenStream2 {
        let attrs = self.attrs;
        let widgets = widgets_path();
        let content = attr_expr(&attrs, "text")
            .or_else(|| attr_expr(&attrs, "content"))
            .map(|expr| quote! { #expr })
            .unwrap_or_else(|| quote! { "" });
        finish_element(quote! { #widgets::StatusBar::new(#content) }, &attrs)
    }

    fn expand_custom_component(self) -> TokenStream2 {
        if !self.attrs.is_empty() || !self.children.is_empty() {
            return compile_error(
                self.name.span(),
                "custom components with props or children are not supported yet",
            );
        }

        let name = self.name;
        let core = core_path();
        quote! { #core::component_with_cx(stringify!(#name), #name) }
    }
}

fn finish_element(builder: TokenStream2, attrs: &[Attribute]) -> TokenStream2 {
    let mut element = quote! { #builder .build() };

    if let Some(layout) = attr_expr(attrs, "layout") {
        element = quote! { #element .with_layout(#layout) };
    }
    if let Some(style) = attr_expr(attrs, "style") {
        element = quote! { #element .with_style(#style) };
    }
    if let Some(focusable) = attr_expr(attrs, "focusable") {
        element = quote! { #element .with_focusable(#focusable) };
    }

    element
}

fn attr_expr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Expr> {
    attrs
        .iter()
        .find(|attr| attr.name == Ident::new(name, Span::call_site()))
        .map(|attr| &attr.value)
}

fn content_expr(attrs: &[Attribute], children: &[Child]) -> Option<TokenStream2> {
    attr_expr(attrs, "content")
        .or_else(|| attr_expr(attrs, "text"))
        .map(|expr| quote! { #expr })
        .or_else(|| {
            if children.len() == 1 {
                match &children[0] {
                    Child::Text(text) => Some(quote! { #text }),
                    Child::Expr(expr) => Some(quote! { #expr }),
                    Child::Node(_) => None,
                }
            } else {
                None
            }
        })
}

fn expr_as_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(value) => Some(value.value()),
            _ => None,
        },
        _ => None,
    }
}

fn compile_error(span: Span, message: &str) -> TokenStream2 {
    syn::Error::new(span, message).to_compile_error()
}
