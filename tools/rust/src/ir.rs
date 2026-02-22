#[derive(Debug, Clone)]
pub enum IrNode {
    Heading { level: usize, text: String },
    Paragraph { text: String },
    ListItem { depth: usize, text: String, ordered: bool },
    Table { rows: Vec<Vec<String>> },
    Kv { key: String, value: String },
    Blank,
    BlockRef { index: usize },
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub index: usize,
    pub lang: String,
    pub content: String,
}

pub struct Stage1Result {
    pub lines: Vec<String>,
    pub blocks: Vec<CodeBlock>,
}
