use lsp_types::{Position, TextDocumentContentChangeEvent};
use tree_sitter::{InputEdit, Language, Parser, Point, Tree};

pub struct ParseState {
    parser: Parser,
    tree: Option<Tree>,
}

impl ParseState {
    pub fn new(language: Language, text: &str) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("Failed to set language for parser");

        let tree = parser.parse(text, None);
        Self { parser, tree }
    }

    pub fn apply_changes(
        &mut self,
        text_before: &str,
        text_after: &str,
        changes: &[TextDocumentContentChangeEvent],
    ) {
        for change in changes {
            if let Some(range) = change.range {
                let start_byte = Self::position_to_offset(text_before, range.start);
                let end_byte = Self::position_to_offset(text_before, range.end);

                let start_point = Point {
                    row: range.start.line as usize,
                    column: range.start.character as usize,
                };
                let old_end_point = Point {
                    row: range.end.line as usize,
                    column: range.end.character as usize,
                };

                let new_end_byte = start_byte + change.text.len();
                let new_end_point = Self::compute_end_point(start_point, &change.text);

                if let Some(tree) = &mut self.tree {
                    tree.edit(&InputEdit {
                        start_byte,
                        old_end_byte: end_byte,
                        new_end_byte,
                        start_position: start_point,
                        old_end_position: old_end_point,
                        new_end_position: new_end_point,
                    });
                }
            } else {
                self.tree = None;
            }
        }

        self.tree = self.parser.parse(text_after, self.tree.as_ref());
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn position_to_offset(text: &str, position: Position) -> usize {
        let mut offset = 0;
        for (i, line) in text.lines().enumerate() {
            if i == position.line as usize {
                offset += position.character as usize;
                break;
            }
            offset += line.len() + 1;
        }
        offset
    }

    fn compute_end_point(start_point: Point, new_text: &str) -> Point {
        let mut row = start_point.row;
        let mut column = start_point.column;
        for byte in new_text.bytes() {
            if byte == b'\n' {
                row += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        Point { row, column }
    }
}
