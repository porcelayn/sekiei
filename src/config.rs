use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wildmatch::WildMatch;

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
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

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ThemeConfig {
    pub theme_type: ThemeType,
    pub preset: Option<String>,
    pub custom: Option<CustomTheme>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct GeneralConfig {
    pub base_url: String,
    pub title: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CustomTheme {
    pub light: HashMap<String, String>,
    pub dark: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Images {
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default)]
    pub compress_to_webp: bool,
}

impl Images {
    pub fn validate(&self) -> Result<(), String> {
        if self.quality != default_quality() && self.compress_to_webp {
            return Err("Fields 'quality' and 'compress_to_webp' cannot be set at the same time in [images]".to_string());
        }
        Ok(())
    }
}

fn default_quality() -> u8 {
    100
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Giscus {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub disabled_routes: Vec<String>,
    #[serde(default)]
    pub enabled_routes: Vec<String>,
    // Required fields when enable = true
    pub repo: Option<String>,
    pub repo_id: Option<String>,
    pub category: Option<String>,
    pub category_id: Option<String>,
}

impl Giscus {
    pub fn validate(&self) -> Result<(), String> {
        if !self.disabled_routes.is_empty() && !self.enabled_routes.is_empty() {
            return Err("Giscus configuration error: 'disabled_routes' and 'enabled_routes' cannot both be specified at the same time".to_string());
        }

        if self.enable {
            if self.repo.is_none() {
                return Err(
                    "Giscus configuration error: 'repo' is required when enable = true".to_string(),
                );
            }
            if self.repo_id.is_none() {
                return Err(
                    "Giscus configuration error: 'repo_id' is required when enable = true"
                        .to_string(),
                );
            }
            if self.category.is_none() {
                return Err(
                    "Giscus configuration error: 'category' is required when enable = true"
                        .to_string(),
                );
            }
            if self.category_id.is_none() {
                return Err(
                    "Giscus configuration error: 'category_id' is required when enable = true"
                        .to_string(),
                );
            }
        }

        Ok(())
    }

    pub fn is_enabled_for_route(&self, route: &str) -> bool {
        if !self.enable {
            return false;
        }

        if !self.enabled_routes.is_empty() {
            self.enabled_routes
                .iter()
                .any(|r| WildMatch::new(r).matches(route))
        } else if !self.disabled_routes.is_empty() {
            !self
                .disabled_routes
                .iter()
                .any(|r| WildMatch::new(r).matches(route))
        } else {
            true
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub theme: ThemeConfig,
    pub general: GeneralConfig,
    pub images: Images,
    #[serde(default)]
    pub giscus: Giscus,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        self.images.validate()?;
        self.giscus.validate()?;
        Ok(())
    }
}

impl Default for Giscus {
    fn default() -> Self {
        Giscus {
            enable: false,
            disabled_routes: Vec::new(),
            enabled_routes: Vec::new(),
            repo: None,
            repo_id: None,
            category: None,
            category_id: None,
        }
    }
}

pub fn get_preset_themes() -> HashMap<String, (HashMap<String, String>, HashMap<String, String>)> {
    // Catppuccin Light
    let catppuccin_light = vec![
        ("background_color", "#ffffff"),
        ("text_color", "#4c4f69"),
        ("link_color", "#1e66f5"),
        ("heading_color", "#8839ef"),
        ("code_background", "#e6e9ef"),
        ("code_text", "#4c4f69"),
        ("border_color", "#ccd0da"),
        ("accent_color", "#1e66f5"),
        ("blockquote_color", "#6c7086"),
        ("secondary_background", "#eff1f5"),
        ("secondary_accent", "#dd7878"),
        ("highlight_add", "rgba(87, 160, 112, 0.3)"),
        ("highlight_del", "rgba(210, 77, 87, 0.3)"),
        ("highlight", "rgba(30, 102, 245, 0.3)"),
        ("type", "#1e66f5"),
        ("constant", "#fe640b"),
        ("string", "#40a02b"),
        ("comment", "#8b949e"),
        ("keyword", "#8839ef"),
        ("function", "#d20f39"),
        ("variable", "#7287fd"),
        ("punctuation", "#6c7086"),
        ("markup_heading", "#d20f39"),
        ("diff_plus", "#d4f1d7"),
        ("diff_minus", "#f8d3d5"),
        ("attribute", "#179299"),
        ("constructor", "#df8e1d"),
        ("tag", "#ea76cb"),
        ("escape", "#d20f39"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Catppuccin Dark
    let catppuccin_dark = vec![
        ("background_color", "#1e1e2e"),
        ("text_color", "#cdd6f4"),
        ("link_color", "#89b4fa"),
        ("heading_color", "#b4befe"),
        ("code_background", "#313244"),
        ("code_text", "#cdd6f4"),
        ("border_color", "#585b70"),
        ("accent_color", "#89b4fa"),
        ("blockquote_color", "#9399b2"),
        ("secondary_background", "#24273a"),
        ("secondary_accent", "#f38ba8"),
        ("highlight_add", "rgba(166, 227, 161, 0.3)"),
        ("highlight_del", "rgba(243, 139, 168, 0.3)"),
        ("highlight", "rgba(137, 180, 250, 0.3)"),
        ("type", "#89b4fa"),
        ("constant", "#fab387"),
        ("string", "#a6e3a1"),
        ("comment", "#585b70"),
        ("keyword", "#cba6f7"),
        ("function", "#f38ba8"),
        ("variable", "#b4befe"),
        ("punctuation", "#9399b2"),
        ("markup_heading", "#f38ba8"),
        ("diff_plus", "rgba(166, 227, 161, 0.3)"),
        ("diff_minus", "rgba(243, 139, 168, 0.3)"),
        ("attribute", "#94e2d5"),
        ("constructor", "#f9e2af"),
        ("tag", "#f5c2e7"),
        ("escape", "#f38ba8"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Gruvbox Light
    let gruvbox_light = vec![
        ("background_color", "#fbf1c7"),
        ("text_color", "#3c3836"),
        ("link_color", "#458588"),
        ("heading_color", "#b57614"),
        ("code_background", "#ebdbb2"),
        ("code_text", "#3c3836"),
        ("border_color", "#a89984"),
        ("accent_color", "#458588"),
        ("blockquote_color", "#7c6f64"),
        ("secondary_background", "#f2e5bc"),
        ("secondary_accent", "#d65d0e"),
        ("highlight_add", "rgba(104, 135, 56, 0.3)"),
        ("highlight_del", "rgba(204, 36, 29, 0.3)"),
        ("highlight", "rgba(69, 133, 136, 0.3)"),
        ("type", "#458588"),
        ("constant", "#d65d0e"),
        ("string", "#79740e"),
        ("comment", "#928374"),
        ("keyword", "#b57614"),
        ("function", "#9d0006"),
        ("variable", "#427b58"),
        ("punctuation", "#7c6f64"),
        ("markup_heading", "#9d0006"),
        ("diff_plus", "#e7f0d2"),
        ("diff_minus", "#f7d9d7"),
        ("attribute", "#689d6a"),
        ("constructor", "#b57614"),
        ("tag", "#af3a03"),
        ("escape", "#9d0006"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Gruvbox Dark
    let gruvbox_dark = vec![
        ("background_color", "#282828"),
        ("text_color", "#ebdbb2"),
        ("link_color", "#83a598"),
        ("heading_color", "#fabd2f"),
        ("code_background", "#3c3836"),
        ("code_text", "#ebdbb2"),
        ("border_color", "#665c54"),
        ("accent_color", "#83a598"),
        ("blockquote_color", "#928374"),
        ("secondary_background", "#32302f"),
        ("secondary_accent", "#fe8019"),
        ("highlight_add", "rgba(166, 192, 102, 0.3)"),
        ("highlight_del", "rgba(251, 73, 52, 0.3)"),
        ("highlight", "rgba(131, 165, 152, 0.3)"),
        ("type", "#83a598"),
        ("constant", "#fe8019"),
        ("string", "#b8bb26"),
        ("comment", "#928374"),
        ("keyword", "#fabd2f"),
        ("function", "#fb4934"),
        ("variable", "#8ec07c"),
        ("punctuation", "#a89984"),
        ("markup_heading", "#fb4934"),
        ("diff_plus", "rgba(166, 192, 102, 0.3)"),
        ("diff_minus", "rgba(251, 73, 52, 0.3)"),
        ("attribute", "#b8bb26"),
        ("constructor", "#fabd2f"),
        ("tag", "#d3869b"),
        ("escape", "#fb4934"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Nord Light
    let nord_light = vec![
        ("background_color", "#eceff4"),
        ("text_color", "#2e3440"),
        ("link_color", "#5e81ac"),
        ("heading_color", "#4c566a"),
        ("code_background", "#e5e9f0"),
        ("code_text", "#2e3440"),
        ("border_color", "#d8dee9"),
        ("accent_color", "#5e81ac"),
        ("blockquote_color", "#4c566a"),
        ("secondary_background", "#f0f4f8"),
        ("secondary_accent", "#81a1c1"),
        ("highlight_add", "rgba(163, 190, 140, 0.3)"),
        ("highlight_del", "rgba(191, 97, 106, 0.3)"),
        ("highlight", "rgba(94, 129, 172, 0.3)"),
        ("type", "#5e81ac"),
        ("constant", "#d08770"),
        ("string", "#a3be8c"),
        ("comment", "#4c566a"),
        ("keyword", "#81a1c1"),
        ("function", "#b48ead"),
        ("variable", "#88c0d0"),
        ("punctuation", "#4c566a"),
        ("markup_heading", "#b48ead"),
        ("diff_plus", "#e7f0e4"),
        ("diff_minus", "#f4e3e5"),
        ("attribute", "#a3be8c"),
        ("constructor", "#d08770"),
        ("tag", "#81a1c1"),
        ("escape", "#b48ead"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Nord Dark
    let nord_dark = vec![
        ("background_color", "#2e3440"),
        ("text_color", "#d8dee9"),
        ("link_color", "#88c0d0"),
        ("heading_color", "#eceff4"),
        ("code_background", "#3b4252"),
        ("code_text", "#d8dee9"),
        ("border_color", "#434c5e"),
        ("accent_color", "#88c0d0"),
        ("blockquote_color", "#4c566a"),
        ("secondary_background", "#353b49"),
        ("secondary_accent", "#81a1c1"),
        ("highlight_add", "rgba(163, 190, 140, 0.3)"),
        ("highlight_del", "rgba(191, 97, 106, 0.3)"),
        ("highlight", "rgba(94, 129, 172, 0.3)"),
        ("type", "#88c0d0"),
        ("constant", "#d08770"),
        ("string", "#a3be8c"),
        ("comment", "#4c566a"),
        ("keyword", "#81a1c1"),
        ("function", "#b48ead"),
        ("variable", "#8fbcbb"),
        ("punctuation", "#d8dee9"),
        ("markup_heading", "#b48ead"),
        ("diff_plus", "rgba(163, 190, 140, 0.3)"),
        ("diff_minus", "rgba(191, 97, 106, 0.3)"),
        ("attribute", "#a3be8c"),
        ("constructor", "#d08770"),
        ("tag", "#81a1c1"),
        ("escape", "#b48ead"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // OneDark Light
    let onedark_light = vec![
        ("background_color", "#fafafa"),
        ("text_color", "#383a42"),
        ("link_color", "#4078f2"),
        ("heading_color", "#a626a4"),
        ("code_background", "#f0f0f0"),
        ("code_text", "#383a42"),
        ("border_color", "#d0d0d0"),
        ("accent_color", "#4078f2"),
        ("blockquote_color", "#696c77"),
        ("secondary_background", "#f5f5f5"),
        ("secondary_accent", "#e45649"),
        ("highlight_add", "rgba(166, 226, 46, 0.3)"),
        ("highlight_del", "rgba(255, 99, 71, 0.3)"),
        ("highlight", "rgba(64, 120, 242, 0.3)"),
        ("type", "#4078f2"),
        ("constant", "#986801"),
        ("string", "#50a14f"),
        ("comment", "#696c77"),
        ("keyword", "#a626a4"),
        ("function", "#c18401"),
        ("variable", "#0184bc"),
        ("punctuation", "#383a42"),
        ("markup_heading", "#c18401"),
        ("diff_plus", "#e9f5e9"),
        ("diff_minus", "#fcecea"),
        ("attribute", "#50a14f"),
        ("constructor", "#986801"),
        ("tag", "#4078f2"),
        ("escape", "#c18401"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // OneDark Dark
    let onedark_dark = vec![
        ("background_color", "#282c34"),
        ("text_color", "#abb2bf"),
        ("link_color", "#61afef"),
        ("heading_color", "#c678dd"),
        ("code_background", "#353b45"),
        ("code_text", "#abb2bf"),
        ("border_color", "#4b5263"),
        ("accent_color", "#61afef"),
        ("blockquote_color", "#5c6370"),
        ("secondary_background", "#21252b"),
        ("secondary_accent", "#e06c75"),
        ("highlight_add", "rgba(166, 226, 46, 0.3)"),
        ("highlight_del", "rgba(255, 99, 71, 0.3)"),
        ("highlight", "rgba(97, 175, 239, 0.3)"),
        ("type", "#61afef"),
        ("constant", "#d19a66"),
        ("string", "#98c379"),
        ("comment", "#5c6370"),
        ("keyword", "#c678dd"),
        ("function", "#e5c07b"),
        ("variable", "#56b6c2"),
        ("punctuation", "#abb2bf"),
        ("markup_heading", "#e5c07b"),
        ("diff_plus", "rgba(166, 226, 46, 0.3)"),
        ("diff_minus", "rgba(255, 99, 71, 0.3)"),
        ("attribute", "#98c379"),
        ("constructor", "#d19a66"),
        ("tag", "#61afef"),
        ("escape", "#e5c07b"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Rosé Pine Light
    let rosepine_light = vec![
        ("background_color", "#fffaf3"),
        ("text_color", "#575279"),
        ("link_color", "#286983"),
        ("heading_color", "#d7827e"),
        ("code_background", "#f2e9e1"),
        ("code_text", "#575279"),
        ("border_color", "#d4d4d4"),
        ("accent_color", "#286983"),
        ("blockquote_color", "#797593"),
        ("secondary_background", "#faf4ed"),
        ("secondary_accent", "#ea9d34"),
        ("highlight_add", "rgba(108, 153, 110, 0.3)"),
        ("highlight_del", "rgba(235, 111, 146, 0.3)"),
        ("highlight", "rgba(40, 105, 131, 0.3)"),
        ("type", "#286983"),
        ("constant", "#d7827e"),
        ("string", "#6c996e"),
        ("comment", "#797593"),
        ("keyword", "#907aa9"),
        ("function", "#ea9d34"),
        ("variable", "#56949f"),
        ("punctuation", "#575279"),
        ("markup_heading", "#ea9d34"),
        ("diff_plus", "#eef5ee"),
        ("diff_minus", "#fceeee"),
        ("attribute", "#6c996e"),
        ("constructor", "#d7827e"),
        ("tag", "#907aa9"),
        ("escape", "#ea9d34"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Rosé Pine Dark
    let rosepine_dark = vec![
        ("background_color", "#191724"),
        ("text_color", "#e0def4"),
        ("link_color", "#56949f"),
        ("heading_color", "#ebbcba"),
        ("code_background", "#1f1d2e"),
        ("code_text", "#e0def4"),
        ("border_color", "#403d52"),
        ("accent_color", "#56949f"),
        ("blockquote_color", "#6e6a86"),
        ("secondary_background", "#26233a"),
        ("secondary_accent", "#f6c177"),
        ("highlight_add", "rgba(108, 153, 110, 0.3)"),
        ("highlight_del", "rgba(235, 111, 146, 0.3)"),
        ("highlight", "rgba(86, 148, 159, 0.3)"),
        ("type", "#56949f"),
        ("constant", "#ebbcba"),
        ("string", "#9ccfd8"),
        ("comment", "#6e6a86"),
        ("keyword", "#c4a7e7"),
        ("function", "#f6c177"),
        ("variable", "#31748f"),
        ("punctuation", "#e0def4"),
        ("markup_heading", "#f6c177"),
        ("diff_plus", "rgba(108, 153, 110, 0.3)"),
        ("diff_minus", "rgba(235, 111, 146, 0.3)"),
        ("attribute", "#9ccfd8"),
        ("constructor", "#ebbcba"),
        ("tag", "#c4a7e7"),
        ("escape", "#f6c177"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Dracula Light
    let dracula_light = vec![
        ("background_color", "#f8f8f2"),
        ("text_color", "#282a36"),
        ("link_color", "#8be9fd"),
        ("heading_color", "#ff79c6"),
        ("code_background", "#f1fa8c"),
        ("code_text", "#282a36"),
        ("border_color", "#d8d8d8"),
        ("accent_color", "#8be9fd"),
        ("blockquote_color", "#6272a4"),
        ("secondary_background", "#f0f0f0"),
        ("secondary_accent", "#bd93f9"),
        ("highlight_add", "rgba(139, 233, 253, 0.3)"),
        ("highlight_del", "rgba(255, 121, 198, 0.3)"),
        ("highlight", "rgba(80, 250, 123, 0.3)"),
        ("type", "#8be9fd"),
        ("constant", "#f1fa8c"),
        ("string", "#50fa7b"),
        ("comment", "#6272a4"),
        ("keyword", "#ff79c6"),
        ("function", "#bd93f9"),
        ("variable", "#ffb86c"),
        ("punctuation", "#282a36"),
        ("markup_heading", "#bd93f9"),
        ("diff_plus", "#e9f5e9"),
        ("diff_minus", "#fcecea"),
        ("attribute", "#50fa7b"),
        ("constructor", "#f1fa8c"),
        ("tag", "#ff79c6"),
        ("escape", "#bd93f9"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Dracula Dark
    let dracula_dark = vec![
        ("background_color", "#282a36"),
        ("text_color", "#f8f8f2"),
        ("link_color", "#8be9fd"),
        ("heading_color", "#ff79c6"),
        ("code_background", "#44475a"),
        ("code_text", "#f8f8f2"),
        ("border_color", "#6272a4"),
        ("accent_color", "#8be9fd"),
        ("blockquote_color", "#6272a4"),
        ("secondary_background", "#343746"),
        ("secondary_accent", "#bd93f9"),
        ("highlight_add", "rgba(139, 233, 253, 0.3)"),
        ("highlight_del", "rgba(255, 121, 198, 0.3)"),
        ("highlight", "rgba(80, 250, 123, 0.3)"),
        ("type", "#8be9fd"),
        ("constant", "#f1fa8c"),
        ("string", "#50fa7b"),
        ("comment", "#6272a4"),
        ("keyword", "#ff79c6"),
        ("function", "#bd93f9"),
        ("variable", "#ffb86c"),
        ("punctuation", "#f8f8f2"),
        ("markup_heading", "#bd93f9"),
        ("diff_plus", "rgba(80, 250, 123, 0.3)"),
        ("diff_minus", "rgba(255, 121, 198, 0.3)"),
        ("attribute", "#50fa7b"),
        ("constructor", "#f1fa8c"),
        ("tag", "#ff79c6"),
        ("escape", "#bd93f9"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Tokyo Night Light
    let tokyonight_light = vec![
        ("background_color", "#f5f6f5"),
        ("text_color", "#343b58"),
        ("link_color", "#2a6ae9"),
        ("heading_color", "#965ada"),
        ("code_background", "#ebedeb"),
        ("code_text", "#343b58"),
        ("border_color", "#d7dae0"),
        ("accent_color", "#2a6ae9"),
        ("blockquote_color", "#676c87"),
        ("secondary_background", "#eef0f0"),
        ("secondary_accent", "#d84c6e"),
        ("highlight_add", "rgba(66, 211, 146, 0.3)"),
        ("highlight_del", "rgba(216, 76, 110, 0.3)"),
        ("highlight", "rgba(42, 106, 233, 0.3)"),
        ("type", "#2a6ae9"),
        ("constant", "#cb7756"),
        ("string", "#42d392"),
        ("comment", "#676c87"),
        ("keyword", "#965ada"),
        ("function", "#d89f5c"),
        ("variable", "#2b80b7"),
        ("punctuation", "#343b58"),
        ("markup_heading", "#d89f5c"),
        ("diff_plus", "#e9f5ef"),
        ("diff_minus", "#fcecee"),
        ("attribute", "#42d392"),
        ("constructor", "#cb7756"),
        ("tag", "#2a6ae9"),
        ("escape", "#d89f5c"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Tokyo Night Dark
    let tokyonight_dark = vec![
        ("background_color", "#1a1b26"),
        ("text_color", "#a9b1d6"),
        ("link_color", "#7aa2f7"),
        ("heading_color", "#bb9af7"),
        ("code_background", "#24283b"),
        ("code_text", "#a9b1d6"),
        ("border_color", "#414868"),
        ("accent_color", "#7aa2f7"),
        ("blockquote_color", "#565f89"),
        ("secondary_background", "#16161e"),
        ("secondary_accent", "#f7768e"),
        ("highlight_add", "rgba(66, 211, 146, 0.3)"),
        ("highlight_del", "rgba(255, 118, 142, 0.3)"),
        ("highlight", "rgba(122, 162, 247, 0.3)"),
        ("type", "#7aa2f7"),
        ("constant", "#ff9e64"),
        ("string", "#9ece6a"),
        ("comment", "#565f89"),
        ("keyword", "#bb9af7"),
        ("function", "#e0af68"),
        ("variable", "#7dcfff"),
        ("punctuation", "#a9b1d6"),
        ("markup_heading", "#e0af68"),
        ("diff_plus", "rgba(66, 211, 146, 0.3)"),
        ("diff_minus", "rgba(255, 118, 142, 0.3)"),
        ("attribute", "#9ece6a"),
        ("constructor", "#ff9e64"),
        ("tag", "#7aa2f7"),
        ("escape", "#e0af68"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Monokai Light
    let monokai_light = vec![
        ("background_color", "#fafafa"),
        ("text_color", "#272822"),
        ("link_color", "#66d9ef"),
        ("heading_color", "#f92672"),
        ("code_background", "#f0f0f0"),
        ("code_text", "#272822"),
        ("border_color", "#d0d0d0"),
        ("accent_color", "#66d9ef"),
        ("blockquote_color", "#75715e"),
        ("secondary_background", "#f5f5f5"),
        ("secondary_accent", "#fd971f"),
        ("highlight_add", "rgba(102, 217, 239, 0.3)"),
        ("highlight_del", "rgba(249, 38, 114, 0.3)"),
        ("highlight", "rgba(166, 226, 46, 0.3)"),
        ("type", "#66d9ef"),
        ("constant", "#ae81ff"),
        ("string", "#a6e22e"),
        ("comment", "#75715e"),
        ("keyword", "#f92672"),
        ("function", "#fd971f"),
        ("variable", "#e6db74"),
        ("punctuation", "#272822"),
        ("markup_heading", "#fd971f"),
        ("diff_plus", "#e9f5e9"),
        ("diff_minus", "#fcecea"),
        ("attribute", "#a6e22e"),
        ("constructor", "#ae81ff"),
        ("tag", "#f92672"),
        ("escape", "#fd971f"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Monokai Dark
    let monokai_dark = vec![
        ("background_color", "#272822"),
        ("text_color", "#f8f8f2"),
        ("link_color", "#66d9ef"),
        ("heading_color", "#f92672"),
        ("code_background", "#3e3d32"),
        ("code_text", "#f8f8f2"),
        ("border_color", "#75715e"),
        ("accent_color", "#66d9ef"),
        ("blockquote_color", "#75715e"),
        ("secondary_background", "#1e1f1c"),
        ("secondary_accent", "#fd971f"),
        ("highlight_add", "rgba(102, 217, 239, 0.3)"),
        ("highlight_del", "rgba(249, 38, 114, 0.3)"),
        ("highlight", "rgba(166, 226, 46, 0.3)"),
        ("type", "#66d9ef"),
        ("constant", "#ae81ff"),
        ("string", "#a6e22e"),
        ("comment", "#75715e"),
        ("keyword", "#f92672"),
        ("function", "#fd971f"),
        ("variable", "#e6db74"),
        ("punctuation", "#f8f8f2"),
        ("markup_heading", "#fd971f"),
        ("diff_plus", "rgba(166, 226, 46, 0.3)"),
        ("diff_minus", "rgba(249, 38, 114, 0.3)"),
        ("attribute", "#a6e22e"),
        ("constructor", "#ae81ff"),
        ("tag", "#f92672"),
        ("escape", "#fd971f"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Obsidian Light
    let obsidian_light = vec![
        ("background_color", "#f5f5f5"),
        ("text_color", "#2d2d2d"),
        ("link_color", "#5577cc"),
        ("heading_color", "#8839ef"),
        ("code_background", "#e8e8e8"),
        ("code_text", "#2d2d2d"),
        ("border_color", "#d3d3d3"),
        ("accent_color", "#5577cc"),
        ("blockquote_color", "#666666"),
        ("secondary_background", "#ececec"),
        ("secondary_accent", "#dd7878"),
        ("highlight_add", "rgba(87, 160, 112, 0.3)"),
        ("highlight_del", "rgba(210, 77, 87, 0.3)"),
        ("highlight", "rgba(85, 119, 204, 0.3)"),
        ("type", "#5577cc"),
        ("constant", "#fe640b"),
        ("string", "#40a02b"),
        ("comment", "#666666"),
        ("keyword", "#8839ef"),
        ("function", "#d20f39"),
        ("variable", "#7287fd"),
        ("punctuation", "#2d2d2d"),
        ("markup_heading", "#d20f39"),
        ("diff_plus", "#e7f0e4"),
        ("diff_minus", "#fcecea"),
        ("attribute", "#179299"),
        ("constructor", "#df8e1d"),
        ("tag", "#ea76cb"),
        ("escape", "#d20f39"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Obsidian Dark
    let obsidian_dark = vec![
        ("background_color", "#2d2d2d"),
        ("text_color", "#e6e6e6"),
        ("link_color", "#7aa2f7"),
        ("heading_color", "#cba6f7"),
        ("code_background", "#363636"),
        ("code_text", "#e6e6e6"),
        ("border_color", "#4a4a4a"),
        ("accent_color", "#7aa2f7"),
        ("blockquote_color", "#8a8a8a"),
        ("secondary_background", "#252525"),
        ("secondary_accent", "#f38ba8"),
        ("highlight_add", "rgba(166, 227, 161, 0.3)"),
        ("highlight_del", "rgba(243, 139, 168, 0.3)"),
        ("highlight", "rgba(122, 162, 247, 0.3)"),
        ("type", "#7aa2f7"),
        ("constant", "#ff9e64"),
        ("string", "#9ece6a"),
        ("comment", "#8a8a8a"),
        ("keyword", "#cba6f7"),
        ("function", "#f38ba8"),
        ("variable", "#b4befe"),
        ("punctuation", "#e6e6e6"),
        ("markup_heading", "#f38ba8"),
        ("diff_plus", "rgba(166, 227, 161, 0.3)"),
        ("diff_minus", "rgba(243, 139, 168, 0.3)"),
        ("attribute", "#94e2d5"),
        ("constructor", "#f9e2af"),
        ("tag", "#f5c2e7"),
        ("escape", "#f38ba8"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Everforest Light
    let everforest_light = vec![
        ("background_color", "#fdf6e3"),
        ("text_color", "#5c6a72"),
        ("link_color", "#3a94c5"),
        ("heading_color", "#df5b61"),
        ("code_background", "#f4f0d9"),
        ("code_text", "#5c6a72"),
        ("border_color", "#e0dcc7"),
        ("accent_color", "#3a94c5"),
        ("blockquote_color", "#82968b"),
        ("secondary_background", "#f8f4e1"),
        ("secondary_accent", "#f85552"),
        ("highlight_add", "rgba(93, 146, 119, 0.3)"),
        ("highlight_del", "rgba(223, 91, 97, 0.3)"),
        ("highlight", "rgba(58, 148, 197, 0.3)"),
        ("type", "#3a94c5"),
        ("constant", "#dfa000"),
        ("string", "#8da101"),
        ("comment", "#82968b"),
        ("keyword", "#df5b61"),
        ("function", "#f57d26"),
        ("variable", "#35a77c"),
        ("punctuation", "#5c6a72"),
        ("markup_heading", "#f57d26"),
        ("diff_plus", "#eef5ee"),
        ("diff_minus", "#fceeee"),
        ("attribute", "#8da101"),
        ("constructor", "#dfa000"),
        ("tag", "#df5b61"),
        ("escape", "#f57d26"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Everforest Dark
    let everforest_dark = vec![
        ("background_color", "#2d353b"),
        ("text_color", "#d3c6aa"),
        ("link_color", "#7fbbb3"),
        ("heading_color", "#e67e80"),
        ("code_background", "#343f44"),
        ("code_text", "#d3c6aa"),
        ("border_color", "#4a555b"),
        ("accent_color", "#7fbbb3"),
        ("blockquote_color", "#859289"),
        ("secondary_background", "#272e33"),
        ("secondary_accent", "#e69875"),
        ("highlight_add", "rgba(95, 164, 120, 0.3)"),
        ("highlight_del", "rgba(230, 126, 128, 0.3)"),
        ("highlight", "rgba(127, 187, 179, 0.3)"),
        ("type", "#7fbbb3"),
        ("constant", "#dbbc7f"),
        ("string", "#a7c080"),
        ("comment", "#859289"),
        ("keyword", "#e67e80"),
        ("function", "#d699b6"),
        ("variable", "#83c092"),
        ("punctuation", "#d3c6aa"),
        ("markup_heading", "#d699b6"),
        ("diff_plus", "rgba(95, 164, 120, 0.3)"),
        ("diff_minus", "rgba(230, 126, 128, 0.3)"),
        ("attribute", "#a7c080"),
        ("constructor", "#dbbc7f"),
        ("tag", "#e67e80"),
        ("escape", "#d699b6"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Solarized Light
    let solarized_light = vec![
        ("background_color", "#fdf6e3"),
        ("text_color", "#657b83"),
        ("link_color", "#268bd2"),
        ("heading_color", "#d33682"),
        ("code_background", "#eee8d5"),
        ("code_text", "#657b83"),
        ("border_color", "#e0e0dc"),
        ("accent_color", "#268bd2"),
        ("blockquote_color", "#93a1a1"),
        ("secondary_background", "#f5efe0"),
        ("secondary_accent", "#cb4b16"),
        ("highlight_add", "rgba(133, 153, 0, 0.3)"),
        ("highlight_del", "rgba(211, 54, 130, 0.3)"),
        ("highlight", "rgba(38, 139, 210, 0.3)"),
        ("type", "#268bd2"),
        ("constant", "#b58900"),
        ("string", "#859900"),
        ("comment", "#93a1a1"),
        ("keyword", "#d33682"),
        ("function", "#2aa198"),
        ("variable", "#6c71c4"),
        ("punctuation", "#657b83"),
        ("markup_heading", "#2aa198"),
        ("diff_plus", "#f0f5e9"),
        ("diff_minus", "#fceff4"),
        ("attribute", "#859900"),
        ("constructor", "#b58900"),
        ("tag", "#d33682"),
        ("escape", "#2aa198"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Solarized Dark
    let solarized_dark = vec![
        ("background_color", "#002b36"),
        ("text_color", "#839496"),
        ("link_color", "#268bd2"),
        ("heading_color", "#d33682"),
        ("code_background", "#073642"),
        ("code_text", "#839496"),
        ("border_color", "#35535a"),
        ("accent_color", "#268bd2"),
        ("blockquote_color", "#586e75"),
        ("secondary_background", "#01313f"),
        ("secondary_accent", "#cb4b16"),
        ("highlight_add", "rgba(133, 153, 0, 0.3)"),
        ("highlight_del", "rgba(211, 54, 130, 0.3)"),
        ("highlight", "rgba(38, 139, 210, 0.3)"),
        ("type", "#268bd2"),
        ("constant", "#b58900"),
        ("string", "#859900"),
        ("comment", "#586e75"),
        ("keyword", "#d33682"),
        ("function", "#2aa198"),
        ("variable", "#6c71c4"),
        ("punctuation", "#839496"),
        ("markup_heading", "#2aa198"),
        ("diff_plus", "rgba(133, 153, 0, 0.3)"),
        ("diff_minus", "rgba(211, 54, 130, 0.3)"),
        ("attribute", "#859900"),
        ("constructor", "#b58900"),
        ("tag", "#d33682"),
        ("escape", "#2aa198"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    let kanagawa_light = vec![
        ("background_color", "#f7f3e9"),
        ("text_color", "#545464"),
        ("link_color", "#1f7ab3"),
        ("heading_color", "#d0587e"),
        ("code_background", "#ece8dd"),
        ("code_text", "#545464"),
        ("border_color", "#d5d1c6"),
        ("accent_color", "#1f7ab3"),
        ("blockquote_color", "#79798e"),
        ("secondary_background", "#f0ece1"),
        ("secondary_accent", "#a63c5e"),
        ("highlight_add", "rgba(108, 153, 110, 0.3)"),
        ("highlight_del", "rgba(208, 88, 126, 0.3)"),
        ("highlight", "rgba(31, 122, 179, 0.3)"),
        ("type", "#1f7ab3"),
        ("constant", "#a63c5e"),
        ("string", "#6c996e"),
        ("comment", "#79798e"),
        ("keyword", "#d0587e"),
        ("function", "#b3663c"),
        ("variable", "#2e869c"),
        ("punctuation", "#545464"),
        ("markup_heading", "#b3663c"),
        ("diff_plus", "#eef5ee"),
        ("diff_minus", "#fceeee"),
        ("attribute", "#6c996e"),
        ("constructor", "#a63c5e"),
        ("tag", "#d0587e"),
        ("escape", "#b3663c"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Kanagawa Dark
    let kanagawa_dark = vec![
        ("background_color", "#1f1f28"),
        ("text_color", "#dcd7ba"),
        ("link_color", "#7fb4ca"),
        ("heading_color", "#e46876"),
        ("code_background", "#2a2a37"),
        ("code_text", "#dcd7ba"),
        ("border_color", "#363646"),
        ("accent_color", "#7fb4ca"),
        ("blockquote_color", "#727169"),
        ("secondary_background", "#181820"),
        ("secondary_accent", "#ffa066"),
        ("highlight_add", "rgba(115, 162, 112, 0.3)"),
        ("highlight_del", "rgba(228, 104, 118, 0.3)"),
        ("highlight", "rgba(127, 180, 202, 0.3)"),
        ("type", "#7fb4ca"),
        ("constant", "#ffa066"),
        ("string", "#98bb6c"),
        ("comment", "#727169"),
        ("keyword", "#e46876"),
        ("function", "#ffa066"),
        ("variable", "#7aa89f"),
        ("punctuation", "#dcd7ba"),
        ("markup_heading", "#ffa066"),
        ("diff_plus", "rgba(115, 162, 112, 0.3)"),
        ("diff_minus", "rgba(228, 104, 118, 0.3)"),
        ("attribute", "#98bb6c"),
        ("constructor", "#ffa066"),
        ("tag", "#e46876"),
        ("escape", "#ffa066"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Oxocarbon Light
    let oxocarbon_light = vec![
        ("background_color", "#f9f9f9"),
        ("text_color", "#393939"),
        ("link_color", "#ee5396"),
        ("heading_color", "#ff7eb6"),
        ("code_background", "#f0f0f0"),
        ("code_text", "#393939"),
        ("border_color", "#dedede"),
        ("accent_color", "#ee5396"),
        ("blockquote_color", "#6e6e6e"),
        ("secondary_background", "#f4f4f4"),
        ("secondary_accent", "#08bdba"),
        ("highlight_add", "rgba(33, 155, 98, 0.3)"),
        ("highlight_del", "rgba(238, 83, 150, 0.3)"),
        ("highlight", "rgba(255, 126, 182, 0.3)"),
        ("type", "#ee5396"),
        ("constant", "#ff7eb6"),
        ("string", "#42be65"),
        ("comment", "#6e6e6e"),
        ("keyword", "#be95ff"),
        ("function", "#08bdba"),
        ("variable", "#3ddbd9"),
        ("punctuation", "#393939"),
        ("markup_heading", "#08bdba"),
        ("diff_plus", "#e9f5ed"),
        ("diff_minus", "#fcecf2"),
        ("attribute", "#42be65"),
        ("constructor", "#ff7eb6"),
        ("tag", "#be95ff"),
        ("escape", "#08bdba"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Oxocarbon Dark
    let oxocarbon_dark = vec![
        ("background_color", "#161616"),
        ("text_color", "#f2f4f8"),
        ("link_color", "#ee5396"),
        ("heading_color", "#ff7eb6"),
        ("code_background", "#252525"),
        ("code_text", "#f2f4f8"),
        ("border_color", "#393939"),
        ("accent_color", "#ee5396"),
        ("blockquote_color", "#999999"),
        ("secondary_background", "#1e1e1e"),
        ("secondary_accent", "#08bdba"),
        ("highlight_add", "rgba(33, 155, 98, 0.3)"),
        ("highlight_del", "rgba(238, 83, 150, 0.3)"),
        ("highlight", "rgba(255, 126, 182, 0.3)"),
        ("type", "#ee5396"),
        ("constant", "#ff7eb6"),
        ("string", "#42be65"),
        ("comment", "#999999"),
        ("keyword", "#be95ff"),
        ("function", "#08bdba"),
        ("variable", "#3ddbd9"),
        ("punctuation", "#f2f4f8"),
        ("markup_heading", "#08bdba"),
        ("diff_plus", "rgba(33, 155, 98, 0.3)"),
        ("diff_minus", "rgba(238, 83, 150, 0.3)"),
        ("attribute", "#42be65"),
        ("constructor", "#ff7eb6"),
        ("tag", "#be95ff"),
        ("escape", "#08bdba"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Base16 Default Light
    let base16_light = vec![
        ("background_color", "#f5f5f5"),
        ("text_color", "#202020"),
        ("link_color", "#d70000"),
        ("heading_color", "#af00db"),
        ("code_background", "#e8e8e8"),
        ("code_text", "#202020"),
        ("border_color", "#d0d0d0"),
        ("accent_color", "#d70000"),
        ("blockquote_color", "#666666"),
        ("secondary_background", "#ececec"),
        ("secondary_accent", "#d75f00"),
        ("highlight_add", "rgba(00, 135, 00, 0.3)"),
        ("highlight_del", "rgba(215, 0, 0, 0.3)"),
        ("highlight", "rgba(215, 95, 0, 0.3)"),
        ("type", "#d70000"),
        ("constant", "#d75f00"),
        ("string", "#008700"),
        ("comment", "#666666"),
        ("keyword", "#af00db"),
        ("function", "#00afaf"),
        ("variable", "#0087af"),
        ("punctuation", "#202020"),
        ("markup_heading", "#00afaf"),
        ("diff_plus", "#e9f5e9"),
        ("diff_minus", "#fcecea"),
        ("attribute", "#008700"),
        ("constructor", "#d75f00"),
        ("tag", "#af00db"),
        ("escape", "#00afaf"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Base16 Default Dark
    let base16_dark = vec![
        ("background_color", "#151515"),
        ("text_color", "#d0d0d0"),
        ("link_color", "#d70000"),
        ("heading_color", "#af00db"),
        ("code_background", "#303030"),
        ("code_text", "#d0d0d0"),
        ("border_color", "#505050"),
        ("accent_color", "#d70000"),
        ("blockquote_color", "#707070"),
        ("secondary_background", "#1c1c1c"),
        ("secondary_accent", "#d75f00"),
        ("highlight_add", "rgba(00, 135, 00, 0.3)"),
        ("highlight_del", "rgba(215, 0, 0, 0.3)"),
        ("highlight", "rgba(215, 95, 0, 0.3)"),
        ("type", "#d70000"),
        ("constant", "#d75f00"),
        ("string", "#008700"),
        ("comment", "#707070"),
        ("keyword", "#af00db"),
        ("function", "#00afaf"),
        ("variable", "#0087af"),
        ("punctuation", "#d0d0d0"),
        ("markup_heading", "#00afaf"),
        ("diff_plus", "rgba(00, 135, 00, 0.3)"),
        ("diff_minus", "rgba(215, 0, 0, 0.3)"),
        ("attribute", "#008700"),
        ("constructor", "#d75f00"),
        ("tag", "#af00db"),
        ("escape", "#00afaf"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<HashMap<_, _>>();

    // Return all preset themes
    vec![
        (
            "catppuccin".to_string(),
            (catppuccin_light, catppuccin_dark),
        ),
        ("gruvbox".to_string(), (gruvbox_light, gruvbox_dark)),
        ("nord".to_string(), (nord_light, nord_dark)),
        ("onedark".to_string(), (onedark_light, onedark_dark)),
        ("rosepine".to_string(), (rosepine_light, rosepine_dark)),
        ("dracula".to_string(), (dracula_light, dracula_dark)),
        (
            "tokyonight".to_string(),
            (tokyonight_light, tokyonight_dark),
        ),
        ("monokai".to_string(), (monokai_light, monokai_dark)),
        ("obsidian".to_string(), (obsidian_light, obsidian_dark)),
        (
            "everforest".to_string(),
            (everforest_light, everforest_dark),
        ),
        ("solarized".to_string(), (solarized_light, solarized_dark)),
        ("kanagawa".to_string(), (kanagawa_light, kanagawa_dark)),
        ("oxocarbon".to_string(), (oxocarbon_light, oxocarbon_dark)),
        ("base16".to_string(), (base16_light, base16_dark)),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>()
}
