use crate::ast::*;
use crate::lexer::{Span, Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at position {span:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(source: &str) -> Result<Program, ParseError> {
        let tokens = crate::lexer::Lexer::tokenize(source);
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    /// Parse with error recovery - returns successfully parsed items and collected errors
    pub fn parse_resilient(source: &str) -> (Program, Vec<ParseError>) {
        let tokens = crate::lexer::Lexer::tokenize(source);
        let mut parser = Parser::new(tokens);
        parser.parse_program_resilient()
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> TokenKind {
        self.current().map(|t| t.kind).unwrap_or(TokenKind::Eof)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn skip_newlines_and_comments(&mut self) {
        while matches!(
            self.peek_kind(),
            TokenKind::Newline | TokenKind::Comment
        ) {
            self.advance();
        }
    }

    /// Skip tokens until we find the start of a new top-level item or EOF
    fn skip_to_next_item(&mut self) {
        let mut brace_depth = 0;

        loop {
            match self.peek_kind() {
                TokenKind::Eof => break,
                TokenKind::LBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                TokenKind::RBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    self.advance();
                    // If we closed all braces, we're at the end of an item
                    if brace_depth == 0 {
                        break;
                    }
                }
                // Top-level keywords at brace depth 0 indicate new item
                TokenKind::Nutrient | TokenKind::Ingredient | TokenKind::Formula
                | TokenKind::Template | TokenKind::Import if brace_depth == 0 => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parse program with error recovery - collects errors and continues
    fn parse_program_resilient(&mut self) -> (Program, Vec<ParseError>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        loop {
            self.skip_newlines_and_comments();

            match self.peek_kind() {
                TokenKind::Eof => break,
                TokenKind::Import => {
                    match self.parse_import() {
                        Ok(item) => items.push(Item::Import(item)),
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_item();
                        }
                    }
                }
                TokenKind::Nutrient => {
                    match self.parse_nutrient() {
                        Ok(item) => items.push(Item::Nutrient(item)),
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_item();
                        }
                    }
                }
                TokenKind::Ingredient => {
                    match self.parse_ingredient(false) {
                        Ok(item) => items.push(Item::Ingredient(item)),
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_item();
                        }
                    }
                }
                TokenKind::Formula => {
                    match self.parse_formula(false) {
                        Ok(item) => items.push(Item::Formula(item)),
                        Err(e) => {
                            errors.push(e);
                            self.skip_to_next_item();
                        }
                    }
                }
                TokenKind::Template => {
                    self.advance(); // consume 'template'
                    self.skip_newlines_and_comments();
                    match self.peek_kind() {
                        TokenKind::Ingredient => {
                            match self.parse_ingredient(true) {
                                Ok(item) => items.push(Item::Ingredient(item)),
                                Err(e) => {
                                    errors.push(e);
                                    self.skip_to_next_item();
                                }
                            }
                        }
                        TokenKind::Formula => {
                            match self.parse_formula(true) {
                                Ok(item) => items.push(Item::Formula(item)),
                                Err(e) => {
                                    errors.push(e);
                                    self.skip_to_next_item();
                                }
                            }
                        }
                        _ => {
                            if let Some(token) = self.current().cloned() {
                                errors.push(ParseError::UnexpectedToken {
                                    expected: "ingredient or formula after 'template'".to_string(),
                                    found: format!("{:?}", token.kind),
                                    span: token.span,
                                });
                            }
                            self.skip_to_next_item();
                        }
                    }
                }
                _ => {
                    // Unknown token at top level - record error and skip
                    if let Some(token) = self.current().cloned() {
                        errors.push(ParseError::UnexpectedToken {
                            expected: "import, nutrient, ingredient, formula, or template".to_string(),
                            found: format!("{:?}", token.kind),
                            span: token.span,
                        });
                    }
                    self.advance(); // Skip the bad token
                }
            }
        }

        (Program { items }, errors)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        self.skip_newlines_and_comments();
        let token = self.current().cloned();
        match token {
            Some(t) if t.kind == kind => {
                self.advance();
                Ok(t)
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", kind),
                found: format!("{:?}", t.kind),
                span: t.span,
            }),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();

        loop {
            self.skip_newlines_and_comments();

            match self.peek_kind() {
                TokenKind::Eof => break,
                TokenKind::Import => items.push(Item::Import(self.parse_import()?)),
                TokenKind::Nutrient => items.push(Item::Nutrient(self.parse_nutrient()?)),
                TokenKind::Ingredient => items.push(Item::Ingredient(self.parse_ingredient(false)?)),
                TokenKind::Formula => items.push(Item::Formula(self.parse_formula(false)?)),
                TokenKind::Template => {
                    self.advance(); // consume 'template'
                    self.skip_newlines_and_comments();
                    match self.peek_kind() {
                        TokenKind::Ingredient => items.push(Item::Ingredient(self.parse_ingredient(true)?)),
                        TokenKind::Formula => items.push(Item::Formula(self.parse_formula(true)?)),
                        _ => {
                            let token = self.current().cloned().unwrap();
                            return Err(ParseError::UnexpectedToken {
                                expected: "ingredient or formula after 'template'".to_string(),
                                found: format!("{:?}", token.kind),
                                span: token.span,
                            });
                        }
                    }
                }
                _ => {
                    let token = self.current().cloned().unwrap();
                    return Err(ParseError::UnexpectedToken {
                        expected: "import, nutrient, ingredient, formula, or template".to_string(),
                        found: format!("{:?}", token.kind),
                        span: token.span,
                    });
                }
            }
        }

        Ok(Program { items })
    }

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let start = self.expect(TokenKind::Import)?.span;

        // Parse path (sequence of idents with / and .)
        let mut path = String::new();
        loop {
            self.skip_newlines_and_comments();
            match self.peek_kind() {
                TokenKind::Dot => {
                    self.advance();
                    path.push('.');
                }
                TokenKind::Slash => {
                    self.advance();
                    path.push('/');
                }
                TokenKind::Ident => {
                    let token = self.advance().unwrap();
                    path.push_str(&token.text);
                }
                _ => break,
            }
        }

        // Check for .fm extension
        self.skip_newlines_and_comments();
        if self.peek_kind() == TokenKind::Dot {
            self.advance();
            let ext = self.expect(TokenKind::Ident)?;
            path.push('.');
            path.push_str(&ext.text);
        }

        // Check for alias: as name
        let mut alias = None;
        self.skip_newlines_and_comments();
        if self.peek_kind() == TokenKind::Ident {
            if let Some(token) = self.current() {
                if token.text == "as" {
                    self.advance();
                    let name = self.expect(TokenKind::Ident)?;
                    alias = Some(name.text);
                }
            }
        }

        // Check for selections: { ... }
        let mut selections = None;
        self.skip_newlines_and_comments();
        if self.peek_kind() == TokenKind::LBrace {
            self.advance();
            self.skip_newlines_and_comments();

            if self.peek_kind() == TokenKind::Star {
                self.advance();
                selections = Some(ImportSelections::All);
            } else {
                let mut names = Vec::new();
                loop {
                    self.skip_newlines_and_comments();
                    if self.peek_kind() == TokenKind::RBrace {
                        break;
                    }
                    let name = self.expect(TokenKind::Ident)?;
                    names.push(name.text);
                    self.skip_newlines_and_comments();
                    if self.peek_kind() == TokenKind::Comma {
                        self.advance();
                    }
                }
                selections = Some(ImportSelections::Named(names));
            }
            self.expect(TokenKind::RBrace)?;
        }

        let end = self.tokens.get(self.pos.saturating_sub(1))
            .map(|t| t.span.end)
            .unwrap_or(start.end);

        Ok(Import {
            span: Span::new(start.start, end),
            path,
            alias,
            selections,
        })
    }

    fn parse_nutrient(&mut self) -> Result<Nutrient, ParseError> {
        let start = self.expect(TokenKind::Nutrient)?.span;
        let name = self.expect(TokenKind::Ident)?.text;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        loop {
            self.skip_newlines_and_comments();
            if self.peek_kind() == TokenKind::RBrace {
                break;
            }
            properties.push(self.parse_property()?);
        }

        let end = self.expect(TokenKind::RBrace)?.span;

        Ok(Nutrient {
            span: Span::new(start.start, end.end),
            name,
            properties,
        })
    }

    fn parse_ingredient(&mut self, is_template: bool) -> Result<Ingredient, ParseError> {
        let start = self.expect(TokenKind::Ingredient)?.span;
        let name = self.expect(TokenKind::Ident)?.text;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        let mut nutrients = Vec::new();

        loop {
            self.skip_newlines_and_comments();
            match self.peek_kind() {
                TokenKind::RBrace => break,
                TokenKind::Ident => {
                    // Check if it's "nutrients" block or a property
                    let text = self.current().map(|t| t.text.as_str());
                    if text == Some("nutrients") || text == Some("nuts") {
                        self.advance();
                        self.expect(TokenKind::LBrace)?;
                        loop {
                            self.skip_newlines_and_comments();
                            if self.peek_kind() == TokenKind::RBrace {
                                break;
                            }
                            nutrients.push(self.parse_nutrient_value()?);
                        }
                        self.expect(TokenKind::RBrace)?;
                    } else {
                        properties.push(self.parse_property()?);
                    }
                }
                _ => {
                    let token = self.current().cloned().unwrap();
                    return Err(ParseError::UnexpectedToken {
                        expected: "property or nutrients block".to_string(),
                        found: format!("{:?}", token.kind),
                        span: token.span,
                    });
                }
            }
        }

        let end = self.expect(TokenKind::RBrace)?.span;

        Ok(Ingredient {
            span: Span::new(start.start, end.end),
            name,
            is_template,
            properties,
            nutrients,
        })
    }

    fn parse_formula(&mut self, is_template: bool) -> Result<Formula, ParseError> {
        let start = self.expect(TokenKind::Formula)?.span;
        let name = self.expect(TokenKind::Ident)?.text;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        let mut nutrients = Vec::new();
        let mut ingredients = Vec::new();

        loop {
            self.skip_newlines_and_comments();
            match self.peek_kind() {
                TokenKind::RBrace => break,
                TokenKind::Ident => {
                    let ident = self.current().unwrap().text.clone();
                    match ident.as_str() {
                        "nutrients" | "nuts" => {
                            self.advance();
                            self.expect(TokenKind::LBrace)?;
                            loop {
                                self.skip_newlines_and_comments();
                                if self.peek_kind() == TokenKind::RBrace {
                                    break;
                                }
                                nutrients.push(self.parse_nutrient_constraint()?);
                            }
                            self.expect(TokenKind::RBrace)?;
                        }
                        "ingredients" | "ings" => {
                            self.advance();
                            self.expect(TokenKind::LBrace)?;
                            loop {
                                self.skip_newlines_and_comments();
                                if self.peek_kind() == TokenKind::RBrace {
                                    break;
                                }
                                ingredients.push(self.parse_ingredient_constraint()?);
                            }
                            self.expect(TokenKind::RBrace)?;
                        }
                        _ => {
                            properties.push(self.parse_property()?);
                        }
                    }
                }
                _ => {
                    let token = self.current().cloned().unwrap();
                    return Err(ParseError::UnexpectedToken {
                        expected: "property, nutrients, or ingredients block".to_string(),
                        found: format!("{:?}", token.kind),
                        span: token.span,
                    });
                }
            }
        }

        let end = self.expect(TokenKind::RBrace)?.span;

        Ok(Formula {
            span: Span::new(start.start, end.end),
            name,
            is_template,
            properties,
            nutrients,
            ingredients,
        })
    }

    fn parse_property(&mut self) -> Result<Property, ParseError> {
        let name_token = self.expect(TokenKind::Ident)?;
        self.skip_newlines_and_comments();
        let (value, end) = match self.peek_kind() {
            TokenKind::String => {
                let token = self.advance().unwrap();
                // Remove quotes from string
                let s = token.text.trim_matches('"').to_string();
                (PropertyValue::String(s), token.span)
            }
            TokenKind::Number | TokenKind::Ident => {
                // Parse as expression to support references and arithmetic
                let expr = self.parse_expr()?;
                let span = self.expr_span(&expr);

                // Convert simple expressions to simpler PropertyValue variants
                let value = match &expr {
                    Expr::Number(n) => PropertyValue::Number(*n),
                    Expr::Reference(r) if r.parts.len() == 1 => {
                        if let ReferencePart::Ident(name) = &r.parts[0] {
                            PropertyValue::Ident(name.clone())
                        } else {
                            PropertyValue::Expr(expr)
                        }
                    }
                    _ => PropertyValue::Expr(expr),
                };
                (value, span)
            }
            _ => {
                let token = self.current().cloned().unwrap();
                return Err(ParseError::UnexpectedToken {
                    expected: "string, number, or expression".to_string(),
                    found: format!("{:?}", token.kind),
                    span: token.span,
                });
            }
        };

        Ok(Property {
            span: Span::new(name_token.span.start, end.end),
            name: name_token.text,
            value,
        })
    }

    /// Get the span of an expression
    fn expr_span(&self, expr: &Expr) -> Span {
        match expr {
            Expr::Number(_) => Span::new(0, 0), // Fallback, numbers don't store span
            Expr::Reference(r) => r.span,
            Expr::BinaryOp { left, right, .. } => {
                let left_span = self.expr_span(left);
                let right_span = self.expr_span(right);
                Span::new(left_span.start, right_span.end)
            }
            Expr::Paren(inner) => self.expr_span(inner),
        }
    }

    fn parse_nutrient_value(&mut self) -> Result<NutrientValue, ParseError> {
        let start = self.current().map(|t| t.span).unwrap_or(Span::new(0, 0));
        let nutrient = self.parse_reference()?;
        self.skip_newlines_and_comments();

        // Value is optional - if not present, this is a composition reference
        let (value, end) = if self.peek_kind() == TokenKind::Number {
            let value_token = self.advance().unwrap();
            let value: f64 = value_token.text.parse().map_err(|_| {
                ParseError::InvalidNumber(value_token.text.clone())
            })?;
            (Some(value), value_token.span.end)
        } else {
            // No value - this is a composition reference like `corn.nutrients`
            let end = self.tokens.get(self.pos.saturating_sub(1))
                .map(|t| t.span.end)
                .unwrap_or(start.end);
            (None, end)
        };

        Ok(NutrientValue {
            span: Span::new(start.start, end),
            nutrient,
            value,
        })
    }

    fn parse_nutrient_constraint(&mut self) -> Result<NutrientConstraint, ParseError> {
        let start = self.current().map(|t| t.span).unwrap_or(Span::new(0, 0));
        let expr = self.parse_expr()?;
        let bounds = self.parse_bounds(false)?;

        // Parse optional alias: `as identifier`
        self.skip_newlines_and_comments();
        let alias = if self.peek_kind() == TokenKind::As {
            self.advance();
            let ident = self.expect(TokenKind::Ident)?;
            Some(ident.text.clone())
        } else {
            None
        };

        let end = self.tokens.get(self.pos.saturating_sub(1))
            .map(|t| t.span.end)
            .unwrap_or(start.end);

        Ok(NutrientConstraint {
            span: Span::new(start.start, end),
            expr,
            bounds,
            alias,
        })
    }

    fn parse_ingredient_constraint(&mut self) -> Result<IngredientConstraint, ParseError> {
        let start = self.current().map(|t| t.span).unwrap_or(Span::new(0, 0));
        let expr = self.parse_expr()?;
        let bounds = self.parse_bounds(true)?;

        // Parse optional alias: `as identifier`
        self.skip_newlines_and_comments();
        let alias = if self.peek_kind() == TokenKind::As {
            self.advance();
            let ident = self.expect(TokenKind::Ident)?;
            Some(ident.text.clone())
        } else {
            None
        };

        let end = self.tokens.get(self.pos.saturating_sub(1))
            .map(|t| t.span.end)
            .unwrap_or(start.end);

        Ok(IngredientConstraint {
            span: Span::new(start.start, end),
            expr,
            bounds,
            alias,
        })
    }

    fn parse_bounds(&mut self, allow_percent: bool) -> Result<Bounds, ParseError> {
        let mut min = None;
        let mut max = None;

        loop {
            self.skip_newlines_and_comments();
            match self.peek_kind() {
                TokenKind::Min => {
                    self.advance();
                    min = Some(self.parse_bound_value(allow_percent)?);
                }
                TokenKind::Max => {
                    self.advance();
                    max = Some(self.parse_bound_value(allow_percent)?);
                }
                _ => break,
            }
        }

        Ok(Bounds { min, max })
    }

    fn parse_bound_value(&mut self, allow_percent: bool) -> Result<BoundValue, ParseError> {
        self.skip_newlines_and_comments();
        let value_token = self.expect(TokenKind::Number)?;
        let value: f64 = value_token.text.parse().map_err(|_| {
            ParseError::InvalidNumber(value_token.text.clone())
        })?;

        let is_percent = if allow_percent && self.peek_kind() == TokenKind::Percent {
            self.advance();
            true
        } else {
            false
        };

        Ok(BoundValue { value, is_percent })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            self.skip_newlines_and_comments();
            let op = match self.peek_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        loop {
            self.skip_newlines_and_comments();
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_primary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_newlines_and_comments();

        match self.peek_kind() {
            TokenKind::Number => {
                let token = self.advance().unwrap();
                let value: f64 = token.text.parse().map_err(|_| {
                    ParseError::InvalidNumber(token.text.clone())
                })?;
                Ok(Expr::Number(value))
            }
            TokenKind::Ident => {
                let reference = self.parse_reference()?;
                Ok(Expr::Reference(reference))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Paren(Box::new(expr)))
            }
            _ => {
                let token = self.current().cloned().unwrap();
                Err(ParseError::UnexpectedToken {
                    expected: "number, identifier, or (".to_string(),
                    found: format!("{:?}", token.kind),
                    span: token.span,
                })
            }
        }
    }

    fn parse_reference(&mut self) -> Result<Reference, ParseError> {
        let start = self.current().map(|t| t.span).unwrap_or(Span::new(0, 0));
        let mut parts = Vec::new();

        // First part must be an identifier
        let first = self.expect(TokenKind::Ident)?;
        parts.push(ReferencePart::Ident(first.text));

        // Parse subsequent parts
        loop {
            if self.peek_kind() != TokenKind::Dot {
                break;
            }
            self.advance();

            self.skip_newlines_and_comments();
            match self.peek_kind() {
                TokenKind::Ident => {
                    let token = self.advance().unwrap();
                    parts.push(ReferencePart::Ident(token.text.clone()));
                }
                // Handle min/max as reference parts (e.g., base.nutrients.protein.min)
                TokenKind::Min => {
                    self.advance();
                    parts.push(ReferencePart::Min);
                }
                TokenKind::Max => {
                    self.advance();
                    parts.push(ReferencePart::Max);
                }
                TokenKind::LBracket => {
                    self.advance();
                    let mut names = Vec::new();
                    loop {
                        self.skip_newlines_and_comments();
                        if self.peek_kind() == TokenKind::RBracket {
                            break;
                        }
                        let name = self.expect(TokenKind::Ident)?;
                        names.push(name.text);
                        self.skip_newlines_and_comments();
                        if self.peek_kind() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBracket)?;
                    parts.push(ReferencePart::Selection(names));
                }
                _ => break,
            }
        }

        let end = self.tokens.get(self.pos.saturating_sub(1))
            .map(|t| t.span.end)
            .unwrap_or(start.end);

        Ok(Reference {
            span: Span::new(start.start, end),
            parts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nutrient() {
        let source = r#"nutrient protein {
            name "Crude Protein"
            unit "%"
        }"#;
        let program = Parser::parse(source).unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Nutrient(n) => {
                assert_eq!(n.name, "protein");
                assert_eq!(n.properties.len(), 2);
            }
            _ => panic!("Expected nutrient"),
        }
    }

    #[test]
    fn test_parse_ingredient() {
        let source = r#"ingredient corn {
            name "Yellow Corn"
            cost 150
            nutrients {
                protein 8.5
                energy 3350
            }
        }"#;
        let program = Parser::parse(source).unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Ingredient(i) => {
                assert_eq!(i.name, "corn");
                assert_eq!(i.nutrients.len(), 2);
            }
            _ => panic!("Expected ingredient"),
        }
    }

    #[test]
    fn test_parse_formula() {
        let source = r#"formula starter {
            batch_size 1000

            nutrients {
                protein min 20 max 24
                energy min 2800
            }

            ingredients {
                corn max 50%
                soybean_meal min 20%
            }
        }"#;
        let program = Parser::parse(source).unwrap();
        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Formula(f) => {
                assert_eq!(f.name, "starter");
                assert_eq!(f.nutrients.len(), 2);
                assert_eq!(f.ingredients.len(), 2);
            }
            _ => panic!("Expected formula"),
        }
    }

    #[test]
    fn test_parse_ratio_constraint() {
        let source = r#"formula test {
            batch_size 1000
            nutrients {
                calcium / phosphorus min 1.5 max 2.0
            }
            ingredients {}
        }"#;
        let program = Parser::parse(source).unwrap();
        match &program.items[0] {
            Item::Formula(f) => {
                let constraint = &f.nutrients[0];
                match &constraint.expr {
                    Expr::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOp::Div);
                    }
                    _ => panic!("Expected binary op"),
                }
            }
            _ => panic!("Expected formula"),
        }
    }

    #[test]
    fn test_parse_import() {
        let source = r#"import ./nutrients.fm { * }"#;
        let program = Parser::parse(source).unwrap();
        match &program.items[0] {
            Item::Import(i) => {
                assert_eq!(i.path, "./nutrients.fm");
                assert_eq!(i.selections, Some(ImportSelections::All));
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_parse_reference_with_selection() {
        let source = r#"formula test {
            batch_size 1000
            nutrients {
                base.nutrients.[protein, energy]
            }
            ingredients {}
        }"#;
        let program = Parser::parse(source).unwrap();
        match &program.items[0] {
            Item::Formula(f) => {
                let constraint = &f.nutrients[0];
                match &constraint.expr {
                    Expr::Reference(r) => {
                        assert_eq!(r.parts.len(), 3);
                    }
                    _ => panic!("Expected reference"),
                }
            }
            _ => panic!("Expected formula"),
        }
    }

    #[test]
    fn test_shorthand_aliases() {
        // Test short aliases: nuts, ings, batch, desc
        let source = r#"
            ingredient corn {
                cost 100
                nuts {
                    protein 8.0
                }
            }

            formula test {
                batch 1000
                desc "Test formula"

                nuts {
                    protein min 20
                }

                ings {
                    corn
                }
            }
        "#;
        let program = Parser::parse(source).unwrap();
        assert_eq!(program.items.len(), 2);
        match &program.items[1] {
            Item::Formula(f) => {
                assert_eq!(f.name, "test");
                assert_eq!(f.nutrients.len(), 1);
                assert_eq!(f.ingredients.len(), 1);
            }
            _ => panic!("Expected formula"),
        }
    }

    #[test]
    fn test_resilient_parse_with_errors() {
        // Test that valid items are parsed even when other items have errors
        let source = r#"
            nutrient protein {
                name "Protein"
                unit "%"
            }

            nutrient broken
                name "This is broken - missing brace"
                unit "%"
            }

            ingredient corn {
                name "Corn"
                cost 100
                nuts {
                    protein 8.0
                }
            }

            formula test {
                batch 1000
                nuts {
                    protein min 20
                }
                ings {
                    corn
                }
            }
        "#;

        // Regular parse should fail
        assert!(Parser::parse(source).is_err());

        // Resilient parse should return valid items and errors
        let (program, errors) = Parser::parse_resilient(source);

        // Should have parsed the valid items
        assert_eq!(program.items.len(), 3, "Should have 3 valid items (protein, corn, test)");

        // Should have collected errors
        assert!(!errors.is_empty(), "Should have at least one error");

        // Check the valid items
        let mut found_protein = false;
        let mut found_corn = false;
        let mut found_test = false;

        for item in &program.items {
            match item {
                Item::Nutrient(n) if n.name == "protein" => found_protein = true,
                Item::Ingredient(i) if i.name == "corn" => found_corn = true,
                Item::Formula(f) if f.name == "test" => found_test = true,
                _ => {}
            }
        }

        assert!(found_protein, "Should have parsed protein nutrient");
        assert!(found_corn, "Should have parsed corn ingredient");
        assert!(found_test, "Should have parsed test formula");
    }

    #[test]
    fn test_parse_constraint_alias() {
        let source = r#"formula test {
            batch_size 1000
            nutrients {
                calcium / phosphorus min 1.5 max 2.0 as ca_p_ratio
                protein min 20 as min_protein
            }
            ingredients {
                corn + soybean_meal min 50% as grains
                corn max 70%
            }
        }"#;
        let program = Parser::parse(source).unwrap();
        match &program.items[0] {
            Item::Formula(f) => {
                // Check nutrient constraints
                assert_eq!(f.nutrients.len(), 2);
                assert_eq!(f.nutrients[0].alias, Some("ca_p_ratio".to_string()));
                assert_eq!(f.nutrients[1].alias, Some("min_protein".to_string()));

                // Check ingredient constraints
                assert_eq!(f.ingredients.len(), 2);
                assert_eq!(f.ingredients[0].alias, Some("grains".to_string()));
                assert_eq!(f.ingredients[1].alias, None); // No alias
            }
            _ => panic!("Expected formula"),
        }
    }

    #[test]
    fn test_parse_constraint_alias_no_bounds() {
        // Test alias without bounds (just naming an expression)
        let source = r#"formula test {
            batch_size 1000
            nutrients {}
            ingredients {
                corn + soybean_meal + wheat as grains
            }
        }"#;
        let program = Parser::parse(source).unwrap();
        match &program.items[0] {
            Item::Formula(f) => {
                assert_eq!(f.ingredients.len(), 1);
                assert_eq!(f.ingredients[0].alias, Some("grains".to_string()));
                assert!(f.ingredients[0].bounds.min.is_none());
                assert!(f.ingredients[0].bounds.max.is_none());
            }
            _ => panic!("Expected formula"),
        }
    }
}
