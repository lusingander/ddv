use ansi_to_tui::IntoText as _;
use once_cell::sync::Lazy;
use ratatui::{
    style::Stylize,
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use crate::{color::ColorTheme, data::Attribute, widget::ScrollLinesState};

pub fn attribute_to_spans(attr: &Attribute, theme: &ColorTheme) -> Vec<Span<'static>> {
    match attr {
        Attribute::S(s) => {
            let text = format!("\"{s}\"");
            vec![Span::from(text).fg(theme.cell_string_fg)]
        }
        Attribute::N(n) => {
            let text = n.to_string();
            vec![Span::from(text).fg(theme.cell_number_fg)]
        }
        Attribute::B(b) => {
            let text = format!("Blob ({})", b.len());
            vec![Span::from(text).fg(theme.cell_binary_fg)]
        }
        Attribute::BOOL(b) => {
            let text = format!("{b}");
            vec![Span::from(text).fg(theme.cell_bool_fg)]
        }
        Attribute::NULL => {
            let text = "null";
            vec![Span::from(text).fg(theme.cell_null_fg)]
        }
        Attribute::L(attrs) => {
            let mut spans = Vec::new();
            spans.push(Span::from("["));
            for (i, ss) in attrs
                .iter()
                .map(|attr| attribute_to_spans(attr, theme))
                .enumerate()
            {
                spans.extend(ss);
                if i < attrs.len() - 1 {
                    spans.push(Span::from(", "));
                }
            }
            spans.push(Span::from("]"));
            spans
        }
        Attribute::M(map) => {
            let mut spans = Vec::new();
            spans.push(Span::from("{"));
            for (i, (k, v)) in map.iter().enumerate() {
                spans.push(Span::from(format!("{k}: ")));
                spans.extend(attribute_to_spans(v, theme));
                if i < map.len() - 1 {
                    spans.push(Span::from(", "));
                }
            }
            spans.push(Span::from("}"));
            spans
        }
        Attribute::SS(ss) => {
            let mut spans = Vec::new();
            spans.push(Span::from("["));
            for (i, s) in ss
                .iter()
                .map(|s| Span::from(format!("\"{s}\"")).fg(theme.cell_string_fg))
                .enumerate()
            {
                spans.push(s);
                if i < ss.len() - 1 {
                    spans.push(Span::from(", "));
                }
            }
            spans.push(Span::from("]"));
            spans
        }
        Attribute::NS(ns) => {
            let mut spans = Vec::new();
            spans.push(Span::from("["));
            for (i, n) in ns
                .iter()
                .map(|n| Span::from(n.to_string()).fg(theme.cell_number_fg))
                .enumerate()
            {
                spans.push(n);
                if i < ns.len() - 1 {
                    spans.push(Span::from(", "));
                }
            }
            spans.push(Span::from("]"));
            spans
        }
        Attribute::BS(bs) => {
            let mut spans = Vec::new();
            spans.push(Span::from("["));
            for (i, b) in bs
                .iter()
                .map(|b| Span::from(format!("Blob ({})", b.len())).fg(theme.cell_binary_fg))
                .enumerate()
            {
                spans.push(b);
                if i < bs.len() - 1 {
                    spans.push(Span::from(", "));
                }
            }
            spans.push(Span::from("]"));
            spans
        }
    }
}

pub fn raw_string_from_scroll_lines_state(state: &ScrollLinesState) -> String {
    state
        .lines()
        .iter()
        .map(|l| {
            l.iter()
                .map(|s| s.content.as_ref())
                .collect::<Vec<&str>>()
                .join("")
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn cut_spans_by_width<'a>(
    spans: Vec<Span<'a>>,
    max_width: usize,
    ellipsis: &'static str,
    theme: &ColorTheme,
) -> Vec<Span<'a>> {
    let total_spans = spans.len();

    let mut rest_w = max_width;
    rest_w -= console::measure_text_width(ellipsis);

    let mut ret = Vec::new();
    let mut exceed = false;
    for span in spans {
        let w = console::measure_text_width(&span.content);
        ret.push(span);
        if w > rest_w {
            exceed = true;
            break;
        }
        rest_w -= w;
    }

    if !exceed && ret.len() == total_spans {
        return ret;
    }

    let last_span = ret.pop().unwrap();
    let truncated = console::truncate_str(&last_span.content, rest_w, "").to_string();

    ret.push(Span::from(truncated).style(last_span.style));
    ret.push(Span::from(ellipsis).fg(theme.cell_ellipsis_fg));

    ret
}

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static DEFAULT_THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

static JSON_SYNTAX: Lazy<&SyntaxReference> =
    Lazy::new(|| SYNTAX_SET.find_syntax_by_name("JSON").unwrap());
static THEME: Lazy<&Theme> =
    Lazy::new(|| DEFAULT_THEME_SET.themes.get("base16-ocean.dark").unwrap());

pub fn to_highlighted_lines(json_str: &str) -> Vec<Line<'static>> {
    let mut h = HighlightLines::new(&JSON_SYNTAX, &THEME);
    let s = LinesWithEndings::from(json_str)
        .map(|line| {
            let ranges: Vec<(syntect::highlighting::Style, &str)> =
                h.highlight_line(line, &SYNTAX_SET).unwrap();
            as_24_bit_terminal_escaped(&ranges[..], false)
        })
        .collect::<Vec<String>>()
        .join("");
    s.into_text().unwrap().into_iter().collect()
}
