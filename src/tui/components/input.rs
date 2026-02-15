use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct InputField {
    pub label: String,
    pub value: String,
    pub is_password: bool,
    pub is_active: bool,
    pub cursor_position: usize,
}

impl InputField {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: String::new(),
            is_password: false,
            is_active: false,
            cursor_position: 0,
        }
    }

    pub fn with_password(mut self) -> Self {
        self.is_password = true;
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    pub fn display_value(&self) -> String {
        if self.is_password {
            "•".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }

    pub fn to_paragraph(&self) -> Paragraph<'_> {
        let display = self.display_value();
        let style = if self.is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        Paragraph::new(display)
            .block(
                Block::default()
                    .title(self.label.clone())
                    .borders(Borders::ALL)
                    .border_style(style),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_field_insert() {
        let mut field = InputField::new("Test");
        field.insert_char('a');
        field.insert_char('b');
        field.insert_char('c');
        assert_eq!(field.value, "abc");
        assert_eq!(field.cursor_position, 3);
    }

    #[test]
    fn test_input_field_backspace() {
        let mut field = InputField::new("Test").with_value("abc");
        field.backspace();
        assert_eq!(field.value, "ab");
        assert_eq!(field.cursor_position, 2);
    }

    #[test]
    fn test_input_field_cursor_movement() {
        let mut field = InputField::new("Test").with_value("abcde");
        
        field.move_cursor_start();
        assert_eq!(field.cursor_position, 0);
        
        field.move_cursor_end();
        assert_eq!(field.cursor_position, 5);
        
        field.move_cursor_left();
        assert_eq!(field.cursor_position, 4);
        
        field.move_cursor_right();
        assert_eq!(field.cursor_position, 5);
    }

    #[test]
    fn test_password_masking() {
        let field = InputField::new("Password")
            .with_password()
            .with_value("secret");
        
        assert_eq!(field.display_value(), "••••••");
    }
}
