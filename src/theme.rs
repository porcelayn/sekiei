use crate::{config::{Config, ThemeType, get_preset_themes}, file_ops::safely_write_file};
use css_minify::optimizations::{Level as CssLevel, Minifier as CssMinifier};
use std::error::Error;
use std::path::Path;
use colored::Colorize;

pub fn generate_theme_css(config: &Config, theme_css_path: &Path) -> Result<(), Box<dyn Error>> {
    let required_vars = vec![
        "background_color", "text_color", "link_color", "heading_color",
        "code_background", "code_text", "border_color", "accent_color",
        "blockquote_color", "secondary_background", "secondary_accent",
        "highlight_add", "highlight_del", "highlight", "type", "constant",
        "string", "comment", "keyword", "function", "variable", "punctuation",
        "markup_heading", "diff_plus", "diff_minus", "attribute", "constructor",
        "tag", "escape",
    ];

    let (light_vars, dark_vars) = match config.theme.theme_type {
        ThemeType::Preset => {
            let preset_name = config.theme.preset.as_ref().ok_or("Preset name not specified in Config.toml")?;
            let presets = get_preset_themes();
            presets.get(preset_name)
                .ok_or_else(|| format!("Unknown preset theme: {}", preset_name))?
                .clone()
        }
        ThemeType::Custom => {
            let custom = config.theme.custom.as_ref().ok_or("Custom theme not specified in Config.toml")?;
            (custom.light.clone(), custom.dark.clone())
        }
    };

    for var in &required_vars {
        if !light_vars.contains_key(*var) {
            return Err(format!("Missing light theme variable: {}", var).into());
        }
        if !dark_vars.contains_key(*var) {
            return Err(format!("Missing dark theme variable: {}", var).into());
        }
    }

    let mut light_css = String::new();
    for (key, value) in &light_vars {
        let css_key = format!("--{}", key.replace("_", "-"));
        light_css.push_str(&format!("    {}: {};\n", css_key, value));
    }
    let mut dark_css = String::new();
    for (key, value) in &dark_vars {
        let css_key = format!("--{}", key.replace("_", "-"));
        dark_css.push_str(&format!("    {}: {};\n", css_key, value));
    }
    let theme_css = format!(
        r#"
:root {{
{light_css}
}}

@media (prefers-color-scheme: dark) {{
    :root:not([data-theme="light"]) {{
{dark_css}
    }}
}}

[data-theme="light"] {{
{light_css}
}}

[data-theme="dark"] {{
{dark_css}
}}
"#,
        light_css = light_css,
        dark_css = dark_css
    );

    let minified_theme_css = CssMinifier::default()
        .minify(&theme_css, CssLevel::Three)
        .map_err(|e| format!("Failed to minify theme.css: {}", e))?;
    safely_write_file(theme_css_path, &minified_theme_css)?;

    println!(
        "{} theme.css with {} theme",
        "Generated and minified".green(),
        config.theme.theme_type.as_str().yellow()
    );
    Ok(())
}