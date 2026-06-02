use crate::formatting::{formatter, tokenizer};

pub fn format(query: &str, params: &QueryParams, options: &FormatOptions) -> String {
    let named_placeholders = matches!(params, QueryParams::Named(_));

    let tokens = tokenizer::tokenize(query, named_placeholders, options);
    formatter::format(&tokens, params, options)
}

/// The SQL dialect to use. This affects parsing of special characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// Generic SQL syntax, most dialect-specific constructs are disabled
    Generic,
    /// Enables array syntax (`[`, `]`) and operators
    PostgreSql,
    /// Enables `[bracketed identifiers]` and `@variables`
    SQLServer,
}

/// Options for controlling how the library formats SQL
#[derive(Debug, Clone)]
pub struct FormatOptions<'a> {
    /// Controls the type and length of indentation to use
    ///
    /// Default: 2 spaces
    pub indent: Indent,
    /// When set, changes reserved keywords to ALL CAPS
    ///
    /// Default: false
    pub uppercase: Option<bool>,
    /// Controls the number of line breaks after a query
    ///
    /// Default: 1
    pub lines_between_queries: u8,
    /// Ignore case conversion for specified strings in the array.
    ///
    /// Default: None
    pub ignore_case_convert: Option<Vec<&'a str>>,
    /// Keep the query in a single line
    ///
    /// Default: false
    pub inline: bool,
    /// Maximum length of an inline block
    ///
    /// Default: 50
    pub max_inline_block: usize,
    /// Maximum length of inline arguments
    ///
    /// If unset keep every argument in a separate line
    ///
    /// Default: None
    pub max_inline_arguments: Option<usize>,
    /// Inline the argument at the top level if they would fit a line of this length
    ///
    /// Default: None
    pub max_inline_top_level: Option<usize>,
    /// Consider any JOIN statement as a top level keyword instead of a reserved keyword
    ///
    /// Default: false,
    pub joins_as_top_level: bool,
    /// Tell the SQL dialect to use
    ///
    /// Default: Generic
    pub dialect: Dialect,

    pub add_semicolons: bool,
}

impl<'a> Default for FormatOptions<'a> {
    fn default() -> Self {
        FormatOptions {
            indent: Indent::Spaces(2),
            uppercase: None,
            lines_between_queries: 1,
            ignore_case_convert: None,
            inline: false,
            max_inline_block: 50,
            max_inline_arguments: None,
            max_inline_top_level: None,
            joins_as_top_level: false,
            dialect: Dialect::Generic,
            add_semicolons: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Indent {
    Spaces(u8),
    Tabs,
}

#[derive(Debug, Clone, Default)]
pub enum QueryParams {
    Named(Vec<(String, String)>),
    Indexed(Vec<String>),
    #[default]
    None,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct SpanInfo {
    pub full_span: usize,
    pub blocks: usize,
    pub newline_before: bool,
    pub newline_after: bool,
    pub arguments: usize,
}
