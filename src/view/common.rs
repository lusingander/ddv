use std::str::FromStr;

use ansi_to_tui::IntoText as _;
use once_cell::sync::Lazy;
use ratatui::{
    style::Stylize,
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{
        Color, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSet, ThemeSettings,
    },
    parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet, SyntaxSetBuilder},
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
    let span_widths: Vec<usize> = spans
        .iter()
        .map(|s| console::measure_text_width(&s.content))
        .collect();

    if span_widths.iter().sum::<usize>() <= max_width {
        return spans;
    }

    let ellipsis_width = console::measure_text_width(ellipsis);
    if ellipsis_width >= max_width {
        return vec![Span::from(ellipsis).fg(theme.cell_ellipsis_fg)];
    }

    let mut rest_w = max_width;
    rest_w -= ellipsis_width;

    let mut ret = Vec::new();
    let mut exceed = false;
    for (i, span) in spans.into_iter().enumerate() {
        let w = span_widths[i];
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

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    let mut builder = SyntaxSetBuilder::new();
    let syntax = SyntaxDefinition::load_from_str(CUSTOM_JSON_SYNTAX_DEFINITON, true, None).unwrap();
    builder.add(syntax);
    builder.build()
});

static JSON_SYNTAX: Lazy<&SyntaxReference> =
    Lazy::new(|| SYNTAX_SET.find_syntax_by_name("JSON").unwrap());

static THEME: Lazy<Theme> = Lazy::new(custom_json_theme);

fn custom_json_theme() -> Theme {
    let mut theme = Theme {
        settings: ThemeSettings {
            foreground: Some(syntect_color(255, 255, 255)),
            ..ThemeSettings::default()
        },
        ..Theme::default()
    };
    theme
        .scopes
        .push(theme_item("constant.numeric.json", 0, 255, 0));
    theme
        .scopes
        .push(theme_item("string.value.json", 255, 0, 255));
    theme
        .scopes
        .push(theme_item("constant.language.boolean.json", 0, 0, 255));
    theme
        .scopes
        .push(theme_item("constant.language.null.json", 0, 255, 255));
    theme
}

fn theme_item(scope: &str, r: u8, g: u8, b: u8) -> ThemeItem {
    ThemeItem {
        scope: ScopeSelectors::from_str(scope).unwrap(),
        style: StyleModifier {
            foreground: Some(syntect_color(r, g, b)),
            ..StyleModifier::default()
        },
    }
}

fn syntect_color(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

const CUSTOM_JSON_SYNTAX_DEFINITON: &str = r###"
%YAML 1.2
---
name: JSON
file_extensions:
  - json
scope: source.json

contexts:
  main:
    - match: '"'
      scope: punctuation.definition.string.begin.json
      push: string
    - match: '\b(true|false)\b'
      scope: constant.language.boolean.json
    - match: '\bnull\b'
      scope: constant.language.null.json
    - match: '[0-9]+(\.[0-9]+)?'
      scope: constant.numeric.json
    - match: '[{}\[\],:]'
      scope: punctuation.separator.json

  string:
    - meta_scope: string.quoted.double.json
    - match: '":'
      scope: punctuation.separator.keyvalue.json
      set: after_key
    - match: '"'
      scope: punctuation.definition.string.end.json
      pop: true
    - match: '\\.'
      scope: constant.character.escape.json

  after_key:
    - match: '"'
      scope: punctuation.definition.string.begin.json
      set: string_value
    - match: '\b(true|false)\b'
      scope: constant.language.boolean.json
      pop: true
    - match: '\bnull\b'
      scope: constant.language.null.json
      pop: true
    - match: '[0-9]+(\.[0-9]+)?'
      scope: constant.numeric.json
      pop: true
    - match: '[{}\[\],]'
      scope: punctuation.separator.json
      pop: true

  string_value:
    - meta_scope: string.value.json
    - match: '"'
      scope: punctuation.definition.string.end.json
      pop: true
    - match: '\\.'
      scope: constant.character.escape.json
"###;
