use ratatui::style::Color;

#[derive(Clone, Copy)]
pub struct ColorTheme {
    pub fg: Color,
    pub bg: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub selected_axis_bg: Color,
    pub quick_filter_matched_fg: Color,
    pub quick_filter_matched_bg: Color,

    pub disabled: Color,
    pub short_help: Color,
    pub notification_success: Color,
    pub notification_warning: Color,
    pub notification_error: Color,

    pub cell_number_fg: Color,
    pub cell_string_fg: Color,
    pub cell_binary_fg: Color,
    pub cell_bool_fg: Color,
    pub cell_null_fg: Color,
    pub cell_undefined_fg: Color,
    pub cell_ellipsis_fg: Color,

    pub item_attribute_type_fg: Color,

    pub insight_attribute_name_fg: Color,
    pub insight_attribute_value_fg: Color,

    pub help_key_fg: Color,
    pub help_link_fg: Color,

    pub line_number_fg: Color,
    pub divier_fg: Color,
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme {
            fg: Color::Reset,
            bg: Color::Reset,
            selected_fg: Color::Black,
            selected_bg: Color::LightGreen,
            selected_axis_bg: Color::DarkGray,
            quick_filter_matched_fg: Color::Black,
            quick_filter_matched_bg: Color::Yellow,

            disabled: Color::DarkGray,
            short_help: Color::DarkGray,
            notification_success: Color::Green,
            notification_warning: Color::Yellow,
            notification_error: Color::Red,

            cell_number_fg: Color::Blue,
            cell_string_fg: Color::Green,
            cell_binary_fg: Color::Cyan,
            cell_bool_fg: Color::Red,
            cell_null_fg: Color::Magenta,
            cell_undefined_fg: Color::DarkGray,
            cell_ellipsis_fg: Color::Reset,

            item_attribute_type_fg: Color::DarkGray,

            insight_attribute_name_fg: Color::Green,
            insight_attribute_value_fg: Color::DarkGray,

            help_key_fg: Color::Yellow,
            help_link_fg: Color::Blue,

            line_number_fg: Color::DarkGray,
            divier_fg: Color::DarkGray,
        }
    }
}
