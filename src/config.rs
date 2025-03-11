use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeType {
    Custom,
    Preset,
}

impl ThemeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeType::Custom => "custom",
            ThemeType::Preset => "preset",
        }
    }

    // fn from_str(s: &str) -> Option<Self> {
    //     match s.to_lowercase().as_str() {
    //         "custom" => Some(ThemeType::Custom),
    //         "preset" => Some(ThemeType::Preset),
    //         _ => None,
    //     }
    // }
}

#[derive(Deserialize)]
pub struct Config {
    pub theme: ThemeConfig,
}

#[derive(Deserialize)]
pub struct ThemeConfig {
    pub theme_type: ThemeType,
    pub preset: Option<String>,
    pub custom: Option<CustomTheme>,
}

#[derive(Deserialize)]
pub struct CustomTheme {
    pub light: HashMap<String, String>,
    pub dark: HashMap<String, String>,
}

pub fn get_preset_themes() -> HashMap<String, (HashMap<String, String>, HashMap<String, String>)> {
    let mut presets = HashMap::new();

    // Catppuccin Preset
    let mut catppuccin_light = HashMap::new();
    catppuccin_light.insert("background_color".to_string(), "#ffffff".to_string());
    catppuccin_light.insert("text_color".to_string(), "#4c4f69".to_string());
    catppuccin_light.insert("link_color".to_string(), "#1e66f5".to_string());
    catppuccin_light.insert("heading_color".to_string(), "#8839ef".to_string());
    catppuccin_light.insert("code_background".to_string(), "#e6e9ef".to_string());
    catppuccin_light.insert("code_text".to_string(), "#4c4f69".to_string());
    catppuccin_light.insert("border_color".to_string(), "#ccd0da".to_string());
    catppuccin_light.insert("accent_color".to_string(), "#1e66f5".to_string()); // Same as link_color
    catppuccin_light.insert("blockquote_color".to_string(), "#6c7086".to_string());
    catppuccin_light.insert("secondary_background".to_string(), "#eff1f5".to_string());
    catppuccin_light.insert("secondary_accent".to_string(), "#dd7878".to_string());
    // Syntax highlighting colors
    catppuccin_light.insert("highlight_add".to_string(), "rgba(87, 160, 112, 0.3)".to_string());
    catppuccin_light.insert("highlight_del".to_string(), "rgba(210, 77, 87, 0.3)".to_string());
    catppuccin_light.insert("highlight".to_string(), "rgba(30, 102, 245, 0.3)".to_string());
    catppuccin_light.insert("type".to_string(), "#1e66f5".to_string());
    catppuccin_light.insert("constant".to_string(), "#fe640b".to_string());
    catppuccin_light.insert("string".to_string(), "#40a02b".to_string());
    catppuccin_light.insert("comment".to_string(), "#8b949e".to_string());
    catppuccin_light.insert("keyword".to_string(), "#8839ef".to_string());
    catppuccin_light.insert("function".to_string(), "#d20f39".to_string());
    catppuccin_light.insert("variable".to_string(), "#7287fd".to_string());
    catppuccin_light.insert("punctuation".to_string(), "#6c7086".to_string());
    catppuccin_light.insert("markup_heading".to_string(), "#d20f39".to_string());
    catppuccin_light.insert("diff_plus".to_string(), "#d4f1d7".to_string());
    catppuccin_light.insert("diff_minus".to_string(), "#f8d3d5".to_string());
    catppuccin_light.insert("attribute".to_string(), "#179299".to_string());
    catppuccin_light.insert("constructor".to_string(), "#df8e1d".to_string());
    catppuccin_light.insert("tag".to_string(), "#ea76cb".to_string());
    catppuccin_light.insert("escape".to_string(), "#d20f39".to_string());

    let mut catppuccin_dark = HashMap::new();
    catppuccin_dark.insert("background_color".to_string(), "#1e1e2e".to_string());
    catppuccin_dark.insert("text_color".to_string(), "#cdd6f4".to_string());
    catppuccin_dark.insert("link_color".to_string(), "#89b4fa".to_string());
    catppuccin_dark.insert("heading_color".to_string(), "#b4befe".to_string());
    catppuccin_dark.insert("code_background".to_string(), "#313244".to_string());
    catppuccin_dark.insert("code_text".to_string(), "#cdd6f4".to_string());
    catppuccin_dark.insert("border_color".to_string(), "#585b70".to_string());
    catppuccin_dark.insert("accent_color".to_string(), "#89b4fa".to_string()); // Same as link_color
    catppuccin_dark.insert("blockquote_color".to_string(), "#9399b2".to_string());
    catppuccin_dark.insert("secondary_background".to_string(), "#24273a".to_string());
    catppuccin_dark.insert("secondary_accent".to_string(), "#f38ba8".to_string());
    // Syntax highlighting colors
    catppuccin_dark.insert("highlight_add".to_string(), "rgba(166, 227, 161, 0.3)".to_string());
    catppuccin_dark.insert("highlight_del".to_string(), "rgba(243, 139, 168, 0.3)".to_string());
    catppuccin_dark.insert("highlight".to_string(), "rgba(137, 180, 250, 0.3)".to_string());
    catppuccin_dark.insert("type".to_string(), "#89b4fa".to_string());
    catppuccin_dark.insert("constant".to_string(), "#fab387".to_string());
    catppuccin_dark.insert("string".to_string(), "#a6e3a1".to_string());
    catppuccin_dark.insert("comment".to_string(), "#585b70".to_string());
    catppuccin_dark.insert("keyword".to_string(), "#cba6f7".to_string());
    catppuccin_dark.insert("function".to_string(), "#f38ba8".to_string());
    catppuccin_dark.insert("variable".to_string(), "#b4befe".to_string());
    catppuccin_dark.insert("punctuation".to_string(), "#9399b2".to_string());
    catppuccin_dark.insert("markup_heading".to_string(), "#f38ba8".to_string());
    catppuccin_dark.insert("diff_plus".to_string(), "rgba(166, 227, 161, 0.3)".to_string());
    catppuccin_dark.insert("diff_minus".to_string(), "rgba(243, 139, 168, 0.3)".to_string());
    catppuccin_dark.insert("attribute".to_string(), "#94e2d5".to_string());
    catppuccin_dark.insert("constructor".to_string(), "#f9e2af".to_string());
    catppuccin_dark.insert("tag".to_string(), "#f5c2e7".to_string());
    catppuccin_dark.insert("escape".to_string(), "#f38ba8".to_string());

    presets.insert("catppuccin".to_string(), (catppuccin_light, catppuccin_dark));

    // Gruvbox Preset
    let mut gruvbox_light = HashMap::new();
    gruvbox_light.insert("background_color".to_string(), "#fbf1c7".to_string());
    gruvbox_light.insert("text_color".to_string(), "#3c3836".to_string());
    gruvbox_light.insert("link_color".to_string(), "#458588".to_string());
    gruvbox_light.insert("heading_color".to_string(), "#b57614".to_string());
    gruvbox_light.insert("code_background".to_string(), "#ebdbb2".to_string());
    gruvbox_light.insert("code_text".to_string(), "#3c3836".to_string());
    gruvbox_light.insert("border_color".to_string(), "#a89984".to_string());
    gruvbox_light.insert("accent_color".to_string(), "#458588".to_string()); // Same as link_color
    gruvbox_light.insert("blockquote_color".to_string(), "#7c6f64".to_string());
    gruvbox_light.insert("secondary_background".to_string(), "#f2e5bc".to_string());
    gruvbox_light.insert("secondary_accent".to_string(), "#d65d0e".to_string());
    // Syntax highlighting colors
    gruvbox_light.insert("highlight_add".to_string(), "rgba(104, 135, 56, 0.3)".to_string());
    gruvbox_light.insert("highlight_del".to_string(), "rgba(204, 36, 29, 0.3)".to_string());
    gruvbox_light.insert("highlight".to_string(), "rgba(69, 133, 136, 0.3)".to_string());
    gruvbox_light.insert("type".to_string(), "#458588".to_string());
    gruvbox_light.insert("constant".to_string(), "#d65d0e".to_string());
    gruvbox_light.insert("string".to_string(), "#79740e".to_string());
    gruvbox_light.insert("comment".to_string(), "#928374".to_string());
    gruvbox_light.insert("keyword".to_string(), "#b57614".to_string());
    gruvbox_light.insert("function".to_string(), "#9d0006".to_string());
    gruvbox_light.insert("variable".to_string(), "#427b58".to_string());
    gruvbox_light.insert("punctuation".to_string(), "#7c6f64".to_string());
    gruvbox_light.insert("markup_heading".to_string(), "#9d0006".to_string());
    gruvbox_light.insert("diff_plus".to_string(), "#e7f0d2".to_string());
    gruvbox_light.insert("diff_minus".to_string(), "#f7d9d7".to_string());
    gruvbox_light.insert("attribute".to_string(), "#689d6a".to_string());
    gruvbox_light.insert("constructor".to_string(), "#b57614".to_string());
    gruvbox_light.insert("tag".to_string(), "#af3a03".to_string());
    gruvbox_light.insert("escape".to_string(), "#9d0006".to_string());

    let mut gruvbox_dark = HashMap::new();
    gruvbox_dark.insert("background_color".to_string(), "#282828".to_string());
    gruvbox_dark.insert("text_color".to_string(), "#ebdbb2".to_string());
    gruvbox_dark.insert("link_color".to_string(), "#83a598".to_string());
    gruvbox_dark.insert("heading_color".to_string(), "#fabd2f".to_string());
    gruvbox_dark.insert("code_background".to_string(), "#3c3836".to_string());
    gruvbox_dark.insert("code_text".to_string(), "#ebdbb2".to_string());
    gruvbox_dark.insert("border_color".to_string(), "#665c54".to_string());
    gruvbox_dark.insert("accent_color".to_string(), "#83a598".to_string()); // Same as link_color
    gruvbox_dark.insert("blockquote_color".to_string(), "#928374".to_string());
    gruvbox_dark.insert("secondary_background".to_string(), "#32302f".to_string());
    gruvbox_dark.insert("secondary_accent".to_string(), "#fe8019".to_string());
    // Syntax highlighting colors
    gruvbox_dark.insert("highlight_add".to_string(), "rgba(166, 192, 102, 0.3)".to_string());
    gruvbox_dark.insert("highlight_del".to_string(), "rgba(251, 73, 52, 0.3)".to_string());
    gruvbox_dark.insert("highlight".to_string(), "rgba(131, 165, 152, 0.3)".to_string());
    gruvbox_dark.insert("type".to_string(), "#83a598".to_string());
    gruvbox_dark.insert("constant".to_string(), "#fe8019".to_string());
    gruvbox_dark.insert("string".to_string(), "#b8bb26".to_string());
    gruvbox_dark.insert("comment".to_string(), "#928374".to_string());
    gruvbox_dark.insert("keyword".to_string(), "#fabd2f".to_string());
    gruvbox_dark.insert("function".to_string(), "#fb4934".to_string());
    gruvbox_dark.insert("variable".to_string(), "#8ec07c".to_string());
    gruvbox_dark.insert("punctuation".to_string(), "#a89984".to_string());
    gruvbox_dark.insert("markup_heading".to_string(), "#fb4934".to_string());
    gruvbox_dark.insert("diff_plus".to_string(), "rgba(166, 192, 102, 0.3)".to_string());
    gruvbox_dark.insert("diff_minus".to_string(), "rgba(251, 73, 52, 0.3)".to_string());
    gruvbox_dark.insert("attribute".to_string(), "#b8bb26".to_string());
    gruvbox_dark.insert("constructor".to_string(), "#fabd2f".to_string());
    gruvbox_dark.insert("tag".to_string(), "#d3869b".to_string());
    gruvbox_dark.insert("escape".to_string(), "#fb4934".to_string());

    presets.insert("gruvbox".to_string(), (gruvbox_light, gruvbox_dark));

    presets
}