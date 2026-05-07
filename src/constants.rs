pub const IGNORE_STRING: &str = "poppy-ignore";
pub const CONFIG_FILE: &str = ".poppy.toml";

pub const DEFAULT_CONFIG: &str = include_str!("./default.poppy.toml");

pub const SUPPORTED_EXTENSIONS: &[&str] = &["sql", "py", "rs", "ts", "js", "mjs", "vue"];

pub const SKIPPED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".idea",
    ".vscode",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "coverage",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".venv",
    "venv",
];
