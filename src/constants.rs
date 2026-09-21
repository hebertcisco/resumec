pub const RESUME_SCHEMA_VERSION: u32 = 1;
pub const THEME_SCHEMA_VERSION: u32 = 1;
pub const APP_NAME: &str = "resumec";
pub const DEFAULT_PAGE_MARGIN_PT: f32 = 36.0;

pub const CLASSIC_THEME: &str = include_str!("../assets/themes/classic/theme.yml");
pub const MODERN_THEME: &str = include_str!("../assets/themes/modern/theme.yml");
pub const MINIMAL_THEME: &str = include_str!("../assets/themes/minimal/theme.yml");
pub const EXAMPLE_RESUME_YAML: &str = include_str!("../examples/resume.yaml");

pub const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("classic", CLASSIC_THEME),
    ("modern", MODERN_THEME),
    ("minimal", MINIMAL_THEME),
];
