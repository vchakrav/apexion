use crate::ast::*;
use crate::lexer::{Lexer, Span, Token, TokenKind};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at {span:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Invalid expression at {0:?}")]
    InvalidExpression(Span),
    #[error("Invalid statement at {0:?}")]
    InvalidStatement(Span),
    #[error("Invalid type at {0:?}")]
    InvalidType(Span),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();
        Self { lexer, current }
    }

    /// Parse a complete compilation unit
    pub fn parse(&mut self) -> ParseResult<CompilationUnit> {
        let mut declarations = Vec::new();
        while !self.is_at_end() {
            declarations.push(self.parse_type_declaration()?);
        }
        Ok(CompilationUnit { declarations })
    }

    // ==================== Helper Methods ====================

    fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = std::mem::replace(&mut self.current, self.lexer.next_token());
        token
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current.kind) == std::mem::discriminant(kind)
    }

    #[allow(dead_code)]
    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|k| self.check(k))
    }

    fn consume(&mut self, kind: &TokenKind, expected: &str) -> ParseResult<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: expected.to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            })
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current_span(&self) -> Span {
        self.current.span
    }

    // ==================== Type Declarations ====================

    fn parse_type_declaration(&mut self) -> ParseResult<TypeDeclaration> {
        let annotations = self.parse_annotations()?;

        // Check for trigger first (no modifiers)
        if self.check(&TokenKind::Trigger) {
            return self
                .parse_trigger_declaration()
                .map(TypeDeclaration::Trigger);
        }

        let modifiers = self.parse_class_modifiers()?;

        match &self.current.kind {
            TokenKind::Class => self
                .parse_class_declaration(annotations, modifiers)
                .map(TypeDeclaration::Class),
            TokenKind::Interface => self
                .parse_interface_declaration(annotations, modifiers.access)
                .map(TypeDeclaration::Interface),
            TokenKind::Enum => self
                .parse_enum_declaration(annotations, modifiers.access)
                .map(TypeDeclaration::Enum),
            _ => Err(ParseError::UnexpectedToken {
                expected: "class, interface, or enum".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            }),
        }
    }

    fn parse_annotations(&mut self) -> ParseResult<Vec<Annotation>> {
        let mut annotations = Vec::new();
        while let TokenKind::Annotation(name) = &self.current.kind {
            let name = name.clone();
            let start = self.current_span();
            self.advance();

            let parameters = if self.match_token(&TokenKind::LParen) {
                let params = self.parse_annotation_parameters()?;
                self.consume(&TokenKind::RParen, ")")?;
                params
            } else {
                Vec::new()
            };

            let span = start.merge(self.current_span());
            annotations.push(Annotation {
                name,
                parameters,
                span,
            });
        }
        Ok(annotations)
    }

    fn parse_annotation_parameters(&mut self) -> ParseResult<Vec<AnnotationParameter>> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            // Check if this is a named parameter (name=value) or just a value
            // Named parameters: identifier followed by =
            let name = if let TokenKind::Identifier(id) = &self.current.kind {
                // Peek to see if next token is =
                let next = self.lexer.peek().kind.clone();
                if matches!(next, TokenKind::Eq) {
                    let name = id.clone();
                    self.advance(); // consume identifier
                    self.advance(); // consume =
                    Some(name)
                } else {
                    None
                }
            } else {
                None
            };

            let value = self.parse_expression()?;
            params.push(AnnotationParameter { name, value });

            // Annotation parameters can be comma-separated OR space-separated
            // If we see a comma, consume it
            // If we see ), we're done
            // If we see an identifier (for space-separated params), continue
            self.match_token(&TokenKind::Comma); // optional comma

            if self.check(&TokenKind::RParen) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_class_modifiers(&mut self) -> ParseResult<ClassModifiers> {
        let mut modifiers = ClassModifiers::default();

        loop {
            match &self.current.kind {
                TokenKind::Public => {
                    modifiers.access = AccessModifier::Public;
                    self.advance();
                }
                TokenKind::Private => {
                    modifiers.access = AccessModifier::Private;
                    self.advance();
                }
                TokenKind::Protected => {
                    modifiers.access = AccessModifier::Protected;
                    self.advance();
                }
                TokenKind::Global => {
                    modifiers.access = AccessModifier::Global;
                    self.advance();
                }
                TokenKind::Abstract => {
                    modifiers.is_abstract = true;
                    self.advance();
                }
                TokenKind::Virtual => {
                    modifiers.is_virtual = true;
                    self.advance();
                }
                TokenKind::WithSharing => {
                    modifiers.sharing = Some(SharingModifier::WithSharing);
                    self.advance();
                }
                TokenKind::WithoutSharing => {
                    modifiers.sharing = Some(SharingModifier::WithoutSharing);
                    self.advance();
                }
                TokenKind::InheritedSharing => {
                    modifiers.sharing = Some(SharingModifier::InheritedSharing);
                    self.advance();
                }
                _ => break,
            }
        }
        Ok(modifiers)
    }

    fn parse_member_modifiers(&mut self) -> ParseResult<MemberModifiers> {
        let mut modifiers = MemberModifiers::default();

        loop {
            match &self.current.kind {
                TokenKind::Public => {
                    modifiers.access = AccessModifier::Public;
                    self.advance();
                }
                TokenKind::Private => {
                    modifiers.access = AccessModifier::Private;
                    self.advance();
                }
                TokenKind::Protected => {
                    modifiers.access = AccessModifier::Protected;
                    self.advance();
                }
                TokenKind::Global => {
                    modifiers.access = AccessModifier::Global;
                    self.advance();
                }
                TokenKind::Static => {
                    modifiers.is_static = true;
                    self.advance();
                }
                TokenKind::Final => {
                    modifiers.is_final = true;
                    self.advance();
                }
                TokenKind::Abstract => {
                    modifiers.is_abstract = true;
                    self.advance();
                }
                TokenKind::Virtual => {
                    modifiers.is_virtual = true;
                    self.advance();
                }
                TokenKind::Override => {
                    modifiers.is_override = true;
                    self.advance();
                }
                TokenKind::Transient => {
                    modifiers.is_transient = true;
                    self.advance();
                }
                TokenKind::TestMethod => {
                    modifiers.is_testmethod = true;
                    self.advance();
                }
                TokenKind::WebService => {
                    modifiers.is_webservice = true;
                    self.advance();
                }
                // Sharing modifiers (for inner classes)
                TokenKind::WithSharing => {
                    modifiers.sharing = Some(SharingModifier::WithSharing);
                    self.advance();
                }
                TokenKind::WithoutSharing => {
                    modifiers.sharing = Some(SharingModifier::WithoutSharing);
                    self.advance();
                }
                TokenKind::InheritedSharing => {
                    modifiers.sharing = Some(SharingModifier::InheritedSharing);
                    self.advance();
                }
                _ => break,
            }
        }
        Ok(modifiers)
    }

    fn parse_class_declaration(
        &mut self,
        annotations: Vec<Annotation>,
        modifiers: ClassModifiers,
    ) -> ParseResult<ClassDeclaration> {
        let start = self.current_span();
        self.consume(&TokenKind::Class, "class")?;

        let name = self.parse_identifier()?;
        let type_parameters = self.parse_type_parameters()?;

        let extends = if self.match_token(&TokenKind::Extends) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };

        let implements = if self.match_token(&TokenKind::Implements) {
            self.parse_type_list()?
        } else {
            Vec::new()
        };

        self.consume(&TokenKind::LBrace, "{")?;
        let members = self.parse_class_members()?;
        let end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        Ok(ClassDeclaration {
            annotations,
            modifiers,
            name,
            type_parameters,
            extends,
            implements,
            members,
            span: start.merge(end),
        })
    }

    fn parse_interface_declaration(
        &mut self,
        annotations: Vec<Annotation>,
        access: AccessModifier,
    ) -> ParseResult<InterfaceDeclaration> {
        let start = self.current_span();
        self.consume(&TokenKind::Interface, "interface")?;

        let name = self.parse_identifier()?;
        let type_parameters = self.parse_type_parameters()?;

        let extends = if self.match_token(&TokenKind::Extends) {
            self.parse_type_list()?
        } else {
            Vec::new()
        };

        self.consume(&TokenKind::LBrace, "{")?;
        let members = self.parse_interface_members()?;
        let end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        Ok(InterfaceDeclaration {
            annotations,
            access,
            name,
            type_parameters,
            extends,
            members,
            span: start.merge(end),
        })
    }

    fn parse_enum_declaration(
        &mut self,
        annotations: Vec<Annotation>,
        access: AccessModifier,
    ) -> ParseResult<EnumDeclaration> {
        let start = self.current_span();
        self.consume(&TokenKind::Enum, "enum")?;

        let name = self.parse_identifier()?;
        self.consume(&TokenKind::LBrace, "{")?;

        let mut values = Vec::new();
        if !self.check(&TokenKind::RBrace) {
            loop {
                values.push(self.parse_identifier()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
                // Allow trailing comma
                if self.check(&TokenKind::RBrace) {
                    break;
                }
            }
        }

        let end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        Ok(EnumDeclaration {
            annotations,
            access,
            name,
            values,
            span: start.merge(end),
        })
    }

    fn parse_trigger_declaration(&mut self) -> ParseResult<TriggerDeclaration> {
        let start = self.current_span();
        self.consume(&TokenKind::Trigger, "trigger")?;

        let name = self.parse_identifier()?;
        self.consume(&TokenKind::On, "on")?;
        let object = self.parse_identifier()?;
        self.consume(&TokenKind::LParen, "(")?;

        let events = self.parse_trigger_events()?;
        self.consume(&TokenKind::RParen, ")")?;

        let body = self.parse_block()?;

        Ok(TriggerDeclaration {
            name,
            object,
            events,
            body,
            span: start.merge(self.current_span()),
        })
    }

    fn parse_trigger_events(&mut self) -> ParseResult<Vec<TriggerEvent>> {
        let mut events = Vec::new();

        loop {
            let is_before = if self.match_token(&TokenKind::Before) {
                true
            } else if self.match_token(&TokenKind::After) {
                false
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "before or after".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                });
            };

            let event = match &self.current.kind {
                TokenKind::Insert => {
                    self.advance();
                    if is_before {
                        TriggerEvent::BeforeInsert
                    } else {
                        TriggerEvent::AfterInsert
                    }
                }
                TokenKind::Update => {
                    self.advance();
                    if is_before {
                        TriggerEvent::BeforeUpdate
                    } else {
                        TriggerEvent::AfterUpdate
                    }
                }
                TokenKind::Delete => {
                    self.advance();
                    if is_before {
                        TriggerEvent::BeforeDelete
                    } else {
                        TriggerEvent::AfterDelete
                    }
                }
                TokenKind::Undelete => {
                    self.advance();
                    if is_before {
                        return Err(ParseError::UnexpectedToken {
                            expected: "after undelete (before undelete is not valid)".to_string(),
                            found: "before undelete".to_string(),
                            span: self.current.span,
                        });
                    }
                    TriggerEvent::AfterUndelete
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "insert, update, delete, or undelete".to_string(),
                        found: format!("{:?}", self.current.kind),
                        span: self.current.span,
                    })
                }
            };

            events.push(event);

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(events)
    }

    fn parse_type_parameters(&mut self) -> ParseResult<Vec<TypeParameter>> {
        if !self.match_token(&TokenKind::Lt) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        loop {
            let start = self.current_span();
            let name = self.parse_identifier()?;
            params.push(TypeParameter {
                name,
                span: start.merge(self.current_span()),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(&TokenKind::Gt, ">")?;
        Ok(params)
    }

    fn parse_type_list(&mut self) -> ParseResult<Vec<TypeRef>> {
        let mut types = Vec::new();
        loop {
            types.push(self.parse_type_ref()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(types)
    }

    // ==================== Type References ====================

    fn parse_type_ref(&mut self) -> ParseResult<TypeRef> {
        self.parse_type_ref_impl(false)
    }

    /// Parse a type reference, consuming the full qualified name.
    /// Use this when we know we're in a constructor context (after `new`).
    fn parse_type_ref_full(&mut self) -> ParseResult<TypeRef> {
        self.parse_type_ref_impl(true)
    }

    fn parse_type_ref_impl(&mut self, consume_full_qualified: bool) -> ParseResult<TypeRef> {
        let start = self.current_span();
        let name = if consume_full_qualified {
            self.parse_type_name_full()?
        } else {
            self.parse_type_name()?
        };

        let type_arguments = if self.match_token(&TokenKind::Lt) {
            let args = self.parse_type_arguments()?;
            self.consume_gt()?;
            args
        } else {
            Vec::new()
        };

        let is_array = if self.match_token(&TokenKind::LBracket) {
            self.consume(&TokenKind::RBracket, "]")?;
            true
        } else {
            false
        };

        Ok(TypeRef {
            name,
            type_arguments,
            is_array,
            span: start.merge(self.current_span()),
        })
    }

    /// Consume a '>' token, handling the case where '>>' or '>>>' was tokenized as a single token
    fn consume_gt(&mut self) -> ParseResult<()> {
        match &self.current.kind {
            TokenKind::Gt => {
                self.advance();
                Ok(())
            }
            TokenKind::GtGt => {
                // Replace >> with >
                self.current.kind = TokenKind::Gt;
                // Adjust the span to only cover the first >
                self.current.span.start += 1;
                Ok(())
            }
            TokenKind::GtGtGt => {
                // Replace >>> with >>
                self.current.kind = TokenKind::GtGt;
                // Adjust the span to only cover the first >
                self.current.span.start += 1;
                Ok(())
            }
            TokenKind::GtEq => {
                // Replace >= with =
                self.current.kind = TokenKind::Eq;
                self.current.span.start += 1;
                Ok(())
            }
            TokenKind::GtGtEq => {
                // Replace >>= with >=
                self.current.kind = TokenKind::GtEq;
                self.current.span.start += 1;
                Ok(())
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: ">".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            }),
        }
    }

    fn parse_type_name(&mut self) -> ParseResult<String> {
        self.parse_type_name_impl(false)
    }

    /// Parse a type name, consuming the full qualified name including any identifier followed by `(`.
    /// Use this when we know we're in a constructor context (after `new`).
    fn parse_type_name_full(&mut self) -> ParseResult<String> {
        self.parse_type_name_impl(true)
    }

    fn parse_type_name_impl(&mut self, consume_full_qualified: bool) -> ParseResult<String> {
        // Handle primitive types and collection types
        match &self.current.kind {
            TokenKind::Void => {
                self.advance();
                Ok("void".to_string())
            }
            TokenKind::Boolean => {
                self.advance();
                Ok("Boolean".to_string())
            }
            TokenKind::Integer => {
                self.advance();
                Ok("Integer".to_string())
            }
            TokenKind::Long => {
                self.advance();
                Ok("Long".to_string())
            }
            TokenKind::Double => {
                self.advance();
                Ok("Double".to_string())
            }
            TokenKind::Decimal => {
                self.advance();
                Ok("Decimal".to_string())
            }
            TokenKind::StringType => {
                self.advance();
                Ok("String".to_string())
            }
            TokenKind::Blob => {
                self.advance();
                Ok("Blob".to_string())
            }
            TokenKind::Date => {
                self.advance();
                Ok("Date".to_string())
            }
            TokenKind::Datetime => {
                self.advance();
                Ok("Datetime".to_string())
            }
            TokenKind::Time => {
                self.advance();
                Ok("Time".to_string())
            }
            TokenKind::Id => {
                self.advance();
                Ok("Id".to_string())
            }
            TokenKind::Object => {
                self.advance();
                Ok("Object".to_string())
            }
            TokenKind::List => {
                self.advance();
                Ok("List".to_string())
            }
            TokenKind::Set => {
                self.advance();
                Ok("Set".to_string())
            }
            TokenKind::Map => {
                self.advance();
                Ok("Map".to_string())
            }
            TokenKind::Identifier(name) => {
                let mut full_name = name.clone();
                self.advance();
                // Handle qualified names like System.Type
                // But be careful not to consume dots that precede method calls
                // (e.g., obj.method() should NOT be consumed as qualified type "obj.method")
                // We use peeking to check what's after the dot and after the identifier
                while self.check(&TokenKind::Dot) {
                    // Peek at what's after the dot
                    let after_dot = self.lexer.peek().kind.clone();

                    if let TokenKind::Identifier(part) = after_dot {
                        // Check if this identifier is followed by ( - that indicates a method call
                        // But if consume_full_qualified is true (e.g., after `new`), we consume anyway
                        if !consume_full_qualified {
                            let after_identifier = self.lexer.peek_second().kind.clone();

                            if matches!(after_identifier, TokenKind::LParen) {
                                // This looks like a method call: obj.method()
                                // Don't consume it as a type name
                                break;
                            }
                        }

                        // It's a qualified type name, consume the dot and the identifier
                        self.advance(); // consume dot
                        full_name.push('.');
                        full_name.push_str(&part);
                        self.advance(); // consume identifier
                    } else {
                        // Not followed by an identifier - could be a method call like Database.insert()
                        // (where insert is a keyword, not an Identifier token)
                        // Don't consume the dot, let expression parsing handle it
                        break;
                    }
                }
                Ok(full_name)
            }
            _ => Err(ParseError::InvalidType(self.current_span())),
        }
    }

    fn parse_type_arguments(&mut self) -> ParseResult<Vec<TypeRef>> {
        let mut args = Vec::new();
        if self.check(&TokenKind::Gt) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_type_ref()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(args)
    }

    // ==================== Class Members ====================

    fn parse_class_members(&mut self) -> ParseResult<Vec<ClassMember>> {
        let mut members = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            members.push(self.parse_class_member()?);
        }

        Ok(members)
    }

    fn parse_class_member(&mut self) -> ParseResult<ClassMember> {
        // Check for static initializer block: static { ... }
        if self.check(&TokenKind::Static) {
            // Need to look ahead to see if it's static { (block) or static followed by type/modifier
            let _start = self.current_span();
            self.advance(); // consume static

            if self.check(&TokenKind::LBrace) {
                // This is a static initializer block
                let block = self.parse_block()?;
                return Ok(ClassMember::StaticBlock(block));
            }

            // Not a static block - it's a static member
            // We need to continue parsing with the static modifier already consumed
            let annotations = self.parse_annotations()?;
            let mut modifiers = self.parse_member_modifiers()?;
            modifiers.is_static = true; // We already consumed 'static'

            return self.parse_class_member_after_modifiers(annotations, modifiers);
        }

        let annotations = self.parse_annotations()?;
        let modifiers = self.parse_member_modifiers()?;

        self.parse_class_member_after_modifiers(annotations, modifiers)
    }

    fn parse_class_member_after_modifiers(
        &mut self,
        annotations: Vec<Annotation>,
        modifiers: MemberModifiers,
    ) -> ParseResult<ClassMember> {
        // Check for inner types
        match &self.current.kind {
            TokenKind::Class => {
                let class_modifiers = ClassModifiers {
                    access: modifiers.access,
                    is_abstract: modifiers.is_abstract,
                    is_virtual: modifiers.is_virtual,
                    sharing: modifiers.sharing,
                };
                return self
                    .parse_class_declaration(annotations, class_modifiers)
                    .map(ClassMember::InnerClass);
            }
            TokenKind::Interface => {
                return self
                    .parse_interface_declaration(annotations, modifiers.access)
                    .map(ClassMember::InnerInterface);
            }
            TokenKind::Enum => {
                return self
                    .parse_enum_declaration(annotations, modifiers.access)
                    .map(ClassMember::InnerEnum);
            }
            _ => {}
        }

        // Parse type (or constructor name)
        let type_ref = self.parse_type_ref()?;

        // Check if this is a constructor (type followed directly by '(')
        if self.check(&TokenKind::LParen) && type_ref.type_arguments.is_empty() {
            // This is a constructor - the "type" we parsed is actually the constructor name
            return self.parse_constructor_rest(
                annotations,
                modifiers,
                type_ref.name,
                type_ref.span,
            );
        }

        // Get name (for methods, fields, properties)
        let name = self.parse_identifier()?;

        // Determine if this is a method, field, or property
        match &self.current.kind {
            TokenKind::LParen => {
                // Method
                self.parse_method_rest(annotations, modifiers, type_ref, name)
                    .map(ClassMember::Method)
            }
            TokenKind::LBrace => {
                // Property with accessor block
                self.parse_property_rest(annotations, modifiers, type_ref, name)
                    .map(ClassMember::Property)
            }
            TokenKind::Eq | TokenKind::Semicolon | TokenKind::Comma => {
                // Field
                self.parse_field_rest(annotations, modifiers, type_ref, name)
                    .map(ClassMember::Field)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "(, {, =, or ;".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            }),
        }
    }

    fn parse_constructor_rest(
        &mut self,
        annotations: Vec<Annotation>,
        modifiers: MemberModifiers,
        name: String,
        start: Span,
    ) -> ParseResult<ClassMember> {
        self.consume(&TokenKind::LParen, "(")?;
        let parameters = self.parse_parameters()?;
        self.consume(&TokenKind::RParen, ")")?;

        // Parse constructor body with potential chaining
        self.consume(&TokenKind::LBrace, "{")?;

        // Check for constructor chaining: this(...) or super(...)
        // We need to carefully check - this( or super( is chaining,
        // but this. or super. is a normal statement
        let chained_constructor = if self.check(&TokenKind::This) || self.check(&TokenKind::Super) {
            let chain_start = self.current_span();
            let is_this = self.check(&TokenKind::This);
            self.advance(); // consume this/super

            // Check if followed by ( - that's constructor chaining
            if self.check(&TokenKind::LParen) {
                let kind = if is_this {
                    ConstructorChainKind::This
                } else {
                    ConstructorChainKind::Super
                };
                self.advance(); // consume (
                let arguments = self.parse_arguments()?;
                self.consume(&TokenKind::RParen, ")")?;
                self.consume(&TokenKind::Semicolon, ";")?;

                Some(ConstructorChain {
                    kind,
                    arguments,
                    span: chain_start.merge(self.current_span()),
                })
            } else {
                // Not constructor chaining - it's this.field or similar
                // Create the this/super expression and parse the rest as a statement
                let base_expr = if is_this {
                    Expression::This(chain_start)
                } else {
                    Expression::Super(chain_start)
                };

                // Continue parsing this as an expression (field access, method call, etc.)
                let expr = self.parse_postfix_from(base_expr)?;
                let full_expr = self.parse_expression_rest(expr)?;
                self.consume(&TokenKind::Semicolon, ";")?;

                // Create statement and continue parsing body
                let stmt = Statement::Expression(ExpressionStatement {
                    expression: full_expr,
                    span: chain_start.merge(self.current_span()),
                });

                let mut statements = vec![stmt];
                while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                    statements.push(self.parse_statement()?);
                }

                let block_end = self.current_span();
                self.consume(&TokenKind::RBrace, "}")?;

                let body = Block {
                    statements,
                    span: start.merge(block_end),
                };

                return Ok(ClassMember::Constructor(ConstructorDeclaration {
                    annotations,
                    modifiers,
                    name,
                    parameters,
                    chained_constructor: None,
                    body,
                    span: start.merge(self.current_span()),
                }));
            }
        } else {
            None
        };

        // Parse remaining statements
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        let block_end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        let body = Block {
            statements,
            span: start.merge(block_end),
        };

        Ok(ClassMember::Constructor(ConstructorDeclaration {
            annotations,
            modifiers,
            name,
            parameters,
            chained_constructor,
            body,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_method_rest(
        &mut self,
        annotations: Vec<Annotation>,
        modifiers: MemberModifiers,
        return_type: TypeRef,
        name: String,
    ) -> ParseResult<MethodDeclaration> {
        let start = return_type.span;
        self.consume(&TokenKind::LParen, "(")?;
        let parameters = self.parse_parameters()?;
        self.consume(&TokenKind::RParen, ")")?;

        let body = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_block()?)
        };

        Ok(MethodDeclaration {
            annotations,
            modifiers,
            return_type,
            name,
            type_parameters: Vec::new(),
            parameters,
            body,
            span: start.merge(self.current_span()),
        })
    }

    fn parse_property_rest(
        &mut self,
        annotations: Vec<Annotation>,
        modifiers: MemberModifiers,
        type_ref: TypeRef,
        name: String,
    ) -> ParseResult<PropertyDeclaration> {
        let start = type_ref.span;
        self.consume(&TokenKind::LBrace, "{")?;

        let mut getter = None;
        let mut setter = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let accessor_modifiers = self.parse_member_modifiers()?;
            let accessor_start = self.current_span();

            if self.match_token(&TokenKind::Get) {
                let body = if self.match_token(&TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_block()?)
                };
                getter = Some(PropertyAccessor {
                    modifiers: accessor_modifiers,
                    body,
                    span: accessor_start.merge(self.current_span()),
                });
            } else if self.match_token(&TokenKind::Set) {
                let body = if self.match_token(&TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_block()?)
                };
                setter = Some(PropertyAccessor {
                    modifiers: accessor_modifiers,
                    body,
                    span: accessor_start.merge(self.current_span()),
                });
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "get or set".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                });
            }
        }

        let end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        Ok(PropertyDeclaration {
            annotations,
            modifiers,
            type_ref,
            name,
            getter,
            setter,
            span: start.merge(end),
        })
    }

    fn parse_field_rest(
        &mut self,
        annotations: Vec<Annotation>,
        modifiers: MemberModifiers,
        type_ref: TypeRef,
        first_name: String,
    ) -> ParseResult<FieldDeclaration> {
        let start = type_ref.span;
        let mut declarators = Vec::new();

        // First declarator
        let first_init = if self.match_token(&TokenKind::Eq) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        declarators.push(VariableDeclarator {
            name: first_name,
            initializer: first_init,
            span: start.merge(self.current_span()),
        });

        // Additional declarators
        while self.match_token(&TokenKind::Comma) {
            let decl_start = self.current_span();
            let name = self.parse_identifier()?;
            let initializer = if self.match_token(&TokenKind::Eq) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarators.push(VariableDeclarator {
                name,
                initializer,
                span: decl_start.merge(self.current_span()),
            });
        }

        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(FieldDeclaration {
            annotations,
            modifiers,
            type_ref,
            declarators,
            span: start.merge(self.current_span()),
        })
    }

    fn parse_interface_members(&mut self) -> ParseResult<Vec<InterfaceMember>> {
        let mut members = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let annotations = self.parse_annotations()?;
            let return_type = self.parse_type_ref()?;
            let name = self.parse_identifier()?;

            self.consume(&TokenKind::LParen, "(")?;
            let parameters = self.parse_parameters()?;
            self.consume(&TokenKind::RParen, ")")?;
            self.consume(&TokenKind::Semicolon, ";")?;

            members.push(InterfaceMember::Method(MethodSignature {
                annotations,
                return_type: return_type.clone(),
                name,
                parameters,
                span: return_type.span.merge(self.current_span()),
            }));
        }

        Ok(members)
    }

    fn parse_parameters(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut params = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let annotations = self.parse_annotations()?;
            let is_final = self.match_token(&TokenKind::Final);
            let type_ref = self.parse_type_ref()?;
            let start = type_ref.span;
            let name = self.parse_identifier()?;

            params.push(Parameter {
                annotations,
                is_final,
                type_ref,
                name,
                span: start.merge(self.current_span()),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_identifier(&mut self) -> ParseResult<String> {
        // Many keywords in Apex can also be used as identifiers in certain contexts
        // (field names, variable names, method names, etc.)
        // Apex is permissive - most keywords can be used as identifiers except a few
        let name = match &self.current.kind {
            TokenKind::Identifier(name) => name.clone(),
            // Keywords that can be used as identifiers in Apex
            TokenKind::Id => "Id".to_string(),
            TokenKind::First => "first".to_string(),
            TokenKind::Last => "last".to_string(),
            TokenKind::Order => "order".to_string(),
            TokenKind::Group => "group".to_string(),
            TokenKind::Limit => "limit".to_string(),
            TokenKind::Offset => "offset".to_string(),
            TokenKind::Date => "Date".to_string(),
            TokenKind::Time => "Time".to_string(),
            TokenKind::Trigger => "Trigger".to_string(),
            TokenKind::Object => "Object".to_string(),
            TokenKind::Set => "Set".to_string(),
            TokenKind::Map => "Map".to_string(),
            TokenKind::List => "List".to_string(),
            // Common keywords used as method names
            TokenKind::Get => "get".to_string(),
            TokenKind::Insert => "insert".to_string(),
            TokenKind::Update => "update".to_string(),
            TokenKind::Delete => "delete".to_string(),
            TokenKind::Upsert => "upsert".to_string(),
            TokenKind::Undelete => "undelete".to_string(),
            TokenKind::Merge => "merge".to_string(),
            // .class property access (Type.class)
            TokenKind::Class => "class".to_string(),
            // Other keywords that can be identifiers
            TokenKind::Integer => "Integer".to_string(),
            TokenKind::Long => "Long".to_string(),
            TokenKind::Double => "Double".to_string(),
            TokenKind::Decimal => "Decimal".to_string(),
            TokenKind::StringType => "String".to_string(),
            TokenKind::Boolean => "Boolean".to_string(),
            TokenKind::Blob => "Blob".to_string(),
            TokenKind::Datetime => "Datetime".to_string(),
            TokenKind::After => "after".to_string(),
            TokenKind::Before => "before".to_string(),
            TokenKind::On => "on".to_string(),
            TokenKind::By => "by".to_string(),
            TokenKind::Having => "having".to_string(),
            TokenKind::Select => "select".to_string(),
            TokenKind::From => "from".to_string(),
            TokenKind::Where => "where".to_string(),
            TokenKind::And => "and".to_string(),
            TokenKind::Or => "or".to_string(),
            TokenKind::In => "in".to_string(),
            TokenKind::Like => "like".to_string(),
            TokenKind::New => "new".to_string(),
            TokenKind::Not => "not".to_string(),
            TokenKind::Null => "null".to_string(),
            TokenKind::Void => "void".to_string(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                });
            }
        };
        self.advance();
        Ok(name)
    }

    // ==================== Statements ====================

    fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.current_span();
        self.consume(&TokenKind::LBrace, "{")?;

        let mut statements = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        let end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        Ok(Block {
            statements,
            span: start.merge(end),
        })
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match &self.current.kind {
            TokenKind::LBrace => self.parse_block().map(Statement::Block),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Do => self.parse_do_while_statement(),
            TokenKind::Switch => self.parse_switch_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Throw => self.parse_throw_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Try => self.parse_try_statement(),
            TokenKind::Insert
            | TokenKind::Update
            | TokenKind::Upsert
            | TokenKind::Delete
            | TokenKind::Undelete
            | TokenKind::Merge => self.parse_dml_statement(),
            TokenKind::Semicolon => {
                let span = self.current_span();
                self.advance();
                Ok(Statement::Empty(span))
            }
            TokenKind::Final => self.parse_local_variable_declaration(),
            _ => {
                // Could be a local variable declaration or expression statement
                // Try to determine by looking ahead
                self.parse_variable_or_expression_statement()
            }
        }
    }

    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::If, "if")?;
        self.consume(&TokenKind::LParen, "(")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenKind::RParen, ")")?;

        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.match_token(&TokenKind::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If(IfStatement {
            condition,
            then_branch,
            else_branch,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::For, "for")?;
        self.consume(&TokenKind::LParen, "(")?;

        // Try to determine if this is a traditional for or for-each
        // For-each: for (Type var : collection)
        // Traditional: for (init; condition; update)

        // Look for a type followed by identifier followed by colon
        if self.is_type_start() {
            let type_ref = self.parse_type_ref()?;
            let variable = self.parse_identifier()?;

            if self.match_token(&TokenKind::Colon) {
                // For-each
                let iterable = self.parse_expression()?;
                self.consume(&TokenKind::RParen, ")")?;
                let body = Box::new(self.parse_statement()?);

                return Ok(Statement::ForEach(ForEachStatement {
                    type_ref,
                    variable,
                    iterable,
                    body,
                    span: start.merge(self.current_span()),
                }));
            }

            // Traditional for with variable declaration as init
            let initializer = if self.match_token(&TokenKind::Eq) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            let mut declarators = vec![VariableDeclarator {
                name: variable,
                initializer,
                span: type_ref.span.merge(self.current_span()),
            }];

            while self.match_token(&TokenKind::Comma) {
                let name = self.parse_identifier()?;
                let init = if self.match_token(&TokenKind::Eq) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                declarators.push(VariableDeclarator {
                    name,
                    initializer: init,
                    span: self.current_span(),
                });
            }

            self.consume(&TokenKind::Semicolon, ";")?;

            let init = Some(ForInit::Variables(LocalVariableDeclaration {
                is_final: false,
                type_ref,
                declarators,
                span: start.merge(self.current_span()),
            }));

            return self.parse_traditional_for_rest(start, init);
        }

        // Traditional for without init or with expression init
        let init = if self.match_token(&TokenKind::Semicolon) {
            None
        } else {
            let exprs = self.parse_expression_list()?;
            self.consume(&TokenKind::Semicolon, ";")?;
            Some(ForInit::Expressions(exprs))
        };

        self.parse_traditional_for_rest(start, init)
    }

    fn parse_traditional_for_rest(
        &mut self,
        start: Span,
        init: Option<ForInit>,
    ) -> ParseResult<Statement> {
        let condition = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&TokenKind::Semicolon, ";")?;

        let update = if self.check(&TokenKind::RParen) {
            Vec::new()
        } else {
            self.parse_expression_list()?
        };
        self.consume(&TokenKind::RParen, ")")?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For(ForStatement {
            init,
            condition,
            update,
            body,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::While, "while")?;
        self.consume(&TokenKind::LParen, "(")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenKind::RParen, ")")?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While(WhileStatement {
            condition,
            body,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_do_while_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Do, "do")?;
        let body = Box::new(self.parse_statement()?);
        self.consume(&TokenKind::While, "while")?;
        self.consume(&TokenKind::LParen, "(")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenKind::RParen, ")")?;
        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::DoWhile(DoWhileStatement {
            body,
            condition,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_switch_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Switch, "switch")?;
        self.consume(&TokenKind::On, "on")?;
        let expression = self.parse_expression()?;
        self.consume(&TokenKind::LBrace, "{")?;

        let mut when_clauses = Vec::new();
        while self.match_token(&TokenKind::When) {
            let when_start = self.current_span();
            let values = self.parse_when_value()?;
            let block = self.parse_block()?;
            when_clauses.push(WhenClause {
                values,
                block,
                span: when_start.merge(self.current_span()),
            });
        }

        let end = self.current_span();
        self.consume(&TokenKind::RBrace, "}")?;

        Ok(Statement::Switch(SwitchStatement {
            expression,
            when_clauses,
            span: start.merge(end),
        }))
    }

    fn parse_when_value(&mut self) -> ParseResult<WhenValue> {
        if self.match_token(&TokenKind::Else) {
            return Ok(WhenValue::Else);
        }

        // Check if it's a type binding (Type variable)
        // Type bindings look like: when Account a { }
        // We need to be careful here because enum values like SPRING are identifiers too
        // Only treat it as a type binding if we see: TypeName identifier {
        if self.is_type_start() && !self.is_literal() {
            // Try parsing as type
            let type_ref = self.parse_type_ref()?;

            // Check if next token is an identifier (variable name for type binding)
            if let TokenKind::Identifier(var) = &self.current.kind {
                let variable = var.clone();
                self.advance();
                return Ok(WhenValue::Type { type_ref, variable });
            }

            // Not a type binding - convert type_ref back to expression
            // This handles cases like: when SPRING { } where SPRING is an enum value
            if type_ref.type_arguments.is_empty() && !type_ref.is_array {
                // Simple identifier, treat as expression
                let expr = Expression::Identifier(type_ref.name.clone(), type_ref.span);
                let mut literals = vec![expr];
                while self.match_token(&TokenKind::Comma) {
                    literals.push(self.parse_expression()?);
                }
                return Ok(WhenValue::Literals(literals));
            } else {
                // Complex type but no variable - this is an error
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                });
            }
        }

        // Literal values
        let mut literals = Vec::new();
        loop {
            literals.push(self.parse_expression()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(WhenValue::Literals(literals))
    }

    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Return, "return")?;

        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::Return(ReturnStatement {
            value,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_throw_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Throw, "throw")?;
        let exception = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::Throw(ThrowStatement {
            exception,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_break_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Break, "break")?;
        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::Break(BreakStatement {
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_continue_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Continue, "continue")?;
        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::Continue(ContinueStatement {
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_try_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        self.consume(&TokenKind::Try, "try")?;
        let try_block = self.parse_block()?;

        let mut catch_clauses = Vec::new();
        while self.match_token(&TokenKind::Catch) {
            let catch_start = self.current_span();
            self.consume(&TokenKind::LParen, "(")?;
            let exception_type = self.parse_type_ref()?;
            let variable = self.parse_identifier()?;
            self.consume(&TokenKind::RParen, ")")?;
            let block = self.parse_block()?;

            catch_clauses.push(CatchClause {
                exception_type,
                variable,
                block,
                span: catch_start.merge(self.current_span()),
            });
        }

        let finally_block = if self.match_token(&TokenKind::Finally) {
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::Try(TryStatement {
            try_block,
            catch_clauses,
            finally_block,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_dml_statement(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        let operation = match &self.current.kind {
            TokenKind::Insert => DmlOperation::Insert,
            TokenKind::Update => DmlOperation::Update,
            TokenKind::Upsert => DmlOperation::Upsert,
            TokenKind::Delete => DmlOperation::Delete,
            TokenKind::Undelete => DmlOperation::Undelete,
            TokenKind::Merge => DmlOperation::Merge,
            _ => unreachable!(),
        };
        self.advance();

        // Check for "as system" or "as user" access level
        let access_level = if let TokenKind::Identifier(s) = &self.current.kind {
            if s.to_lowercase() == "as" {
                self.advance();
                if let TokenKind::Identifier(level) = &self.current.kind {
                    let level_str = level.to_lowercase();
                    if level_str == "system" {
                        self.advance();
                        Some(DmlAccessLevel::System)
                    } else if level_str == "user" {
                        self.advance();
                        Some(DmlAccessLevel::User)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let expression = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::Dml(DmlStatement {
            operation,
            expression,
            access_level,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_local_variable_declaration(&mut self) -> ParseResult<Statement> {
        let start = self.current_span();
        let is_final = self.match_token(&TokenKind::Final);
        let type_ref = self.parse_type_ref()?;

        let mut declarators = Vec::new();
        loop {
            let decl_start = self.current_span();
            let name = self.parse_identifier()?;
            let initializer = if self.match_token(&TokenKind::Eq) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarators.push(VariableDeclarator {
                name,
                initializer,
                span: decl_start.merge(self.current_span()),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::LocalVariable(LocalVariableDeclaration {
            is_final,
            type_ref,
            declarators,
            span: start.merge(self.current_span()),
        }))
    }

    fn parse_variable_or_expression_statement(&mut self) -> ParseResult<Statement> {
        // Determine if this looks like a variable declaration
        // Variable declaration: Type Identifier (= expr)? (, Identifier (= expr)?)* ;
        // Expression: everything else
        //
        // The tricky part is that both can start with an identifier.
        // We use heuristics: if we see a known type keyword OR an identifier followed
        // by another identifier (not '(' or '.'), it's likely a variable declaration.

        let start = self.current_span();

        // Check if this looks like it could be a type
        if self.is_definite_type_start() {
            // Known type keywords are definitely types
            let type_ref = self.parse_type_ref()?;
            return self.parse_local_var_after_type(start, type_ref);
        }

        // For identifiers, we need to be more careful
        if let TokenKind::Identifier(_) = &self.current.kind {
            // Parse the potential type
            let type_ref = self.parse_type_ref()?;

            // Now check what follows - if it's an identifier, this is a variable declaration
            if let TokenKind::Identifier(_) = &self.current.kind {
                return self.parse_local_var_after_type(start, type_ref);
            }

            // Otherwise, we parsed too much - the "type" was actually part of an expression
            // We need to convert the type_ref back to an expression and continue parsing
            let expr = self.type_ref_to_expression(type_ref)?;
            let full_expr = self.parse_expression_rest(expr)?;
            self.consume(&TokenKind::Semicolon, ";")?;

            return Ok(Statement::Expression(ExpressionStatement {
                expression: full_expr,
                span: start.merge(self.current_span()),
            }));
        }

        // Parse as expression statement
        let expression = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::Expression(ExpressionStatement {
            expression,
            span: start.merge(self.current_span()),
        }))
    }

    fn is_definite_type_start(&self) -> bool {
        // Returns true for tokens that are definitely types, not identifiers
        matches!(
            &self.current.kind,
            TokenKind::Void
                | TokenKind::Boolean
                | TokenKind::Integer
                | TokenKind::Long
                | TokenKind::Double
                | TokenKind::Decimal
                | TokenKind::StringType
                | TokenKind::Blob
                | TokenKind::Date
                | TokenKind::Datetime
                | TokenKind::Time
                | TokenKind::Id
                | TokenKind::Object
                | TokenKind::List
                | TokenKind::Set
                | TokenKind::Map
        )
    }

    fn parse_local_var_after_type(
        &mut self,
        start: Span,
        type_ref: TypeRef,
    ) -> ParseResult<Statement> {
        let mut declarators = Vec::new();
        loop {
            let decl_start = self.current_span();
            let name = self.parse_identifier()?;
            let initializer = if self.match_token(&TokenKind::Eq) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            declarators.push(VariableDeclarator {
                name,
                initializer,
                span: decl_start.merge(self.current_span()),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(&TokenKind::Semicolon, ";")?;

        Ok(Statement::LocalVariable(LocalVariableDeclaration {
            is_final: false,
            type_ref,
            declarators,
            span: start.merge(self.current_span()),
        }))
    }

    fn type_ref_to_expression(&self, type_ref: TypeRef) -> ParseResult<Expression> {
        // Convert a simple type reference back to an identifier expression
        // This is used when we incorrectly parsed an expression as a type
        if type_ref.type_arguments.is_empty() && !type_ref.is_array {
            // If the name contains dots (qualified name), decompose into field accesses
            let parts: Vec<&str> = type_ref.name.split('.').collect();
            if parts.len() == 1 {
                Ok(Expression::Identifier(type_ref.name, type_ref.span))
            } else {
                // Build a chain of field accesses: a.b.c becomes FieldAccess(FieldAccess(a, b), c)
                let mut expr = Expression::Identifier(parts[0].to_string(), type_ref.span);
                for part in &parts[1..] {
                    expr = Expression::FieldAccess(Box::new(FieldAccessExpr {
                        object: expr,
                        field: part.to_string(),
                        span: type_ref.span,
                    }));
                }
                Ok(expr)
            }
        } else {
            // Complex type refs shouldn't appear in expression context
            Err(ParseError::InvalidExpression(type_ref.span))
        }
    }

    fn parse_expression_rest(&mut self, left: Expression) -> ParseResult<Expression> {
        // Continue parsing an expression given a left-hand side
        // This handles postfix operators, method calls, field access, etc.
        let start = left.span();
        let mut expr = left;

        // First handle postfix operations (method calls, field access, array access)
        loop {
            match &self.current.kind {
                TokenKind::LParen => {
                    // Method call on the identifier
                    if let Expression::Identifier(name, _) = expr {
                        self.advance();
                        let arguments = self.parse_arguments()?;
                        self.consume(&TokenKind::RParen, ")")?;

                        expr = Expression::MethodCall(Box::new(MethodCallExpr {
                            object: None,
                            name,
                            type_arguments: Vec::new(),
                            arguments,
                            span: start.merge(self.current_span()),
                        }));
                    } else {
                        break;
                    }
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.parse_identifier()?;

                    if self.check(&TokenKind::LParen) {
                        self.advance();
                        let arguments = self.parse_arguments()?;
                        self.consume(&TokenKind::RParen, ")")?;

                        expr = Expression::MethodCall(Box::new(MethodCallExpr {
                            object: Some(expr),
                            name,
                            type_arguments: Vec::new(),
                            arguments,
                            span: start.merge(self.current_span()),
                        }));
                    } else {
                        expr = Expression::FieldAccess(Box::new(FieldAccessExpr {
                            object: expr,
                            field: name,
                            span: start.merge(self.current_span()),
                        }));
                    }
                }
                TokenKind::QuestionDot => {
                    // Safe navigation: obj?.field or obj?.method()
                    self.advance();
                    let name = self.parse_identifier()?;

                    if self.check(&TokenKind::LParen) {
                        self.advance();
                        let arguments = self.parse_arguments()?;
                        self.consume(&TokenKind::RParen, ")")?;

                        let safe_obj = Expression::SafeNavigation(Box::new(SafeNavigationExpr {
                            object: expr,
                            field: name.clone(),
                            span: start.merge(self.current_span()),
                        }));
                        expr = Expression::MethodCall(Box::new(MethodCallExpr {
                            object: Some(safe_obj),
                            name,
                            type_arguments: Vec::new(),
                            arguments,
                            span: start.merge(self.current_span()),
                        }));
                    } else {
                        expr = Expression::SafeNavigation(Box::new(SafeNavigationExpr {
                            object: expr,
                            field: name,
                            span: start.merge(self.current_span()),
                        }));
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.consume(&TokenKind::RBracket, "]")?;
                    expr = Expression::ArrayAccess(Box::new(ArrayAccessExpr {
                        array: expr,
                        index,
                        span: start.merge(self.current_span()),
                    }));
                }
                TokenKind::PlusPlus => {
                    self.advance();
                    expr =
                        Expression::PostIncrement(Box::new(expr), start.merge(self.current_span()));
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    expr =
                        Expression::PostDecrement(Box::new(expr), start.merge(self.current_span()));
                }
                _ => break,
            }
        }

        // Now handle binary operators and other expression continuations
        self.parse_binary_rest(expr, 0)
    }

    fn parse_binary_rest(&mut self, left: Expression, min_prec: u8) -> ParseResult<Expression> {
        let mut left = left;
        let start = left.span();

        loop {
            let (op, prec) = match &self.current.kind {
                // Assignment operators (lowest precedence, right associative)
                TokenKind::Eq => (Some(AssignmentOp::Assign), 1),
                TokenKind::PlusEq => (Some(AssignmentOp::AddAssign), 1),
                TokenKind::MinusEq => (Some(AssignmentOp::SubAssign), 1),
                TokenKind::StarEq => (Some(AssignmentOp::MulAssign), 1),
                TokenKind::SlashEq => (Some(AssignmentOp::DivAssign), 1),
                TokenKind::PercentEq => (Some(AssignmentOp::ModAssign), 1),
                _ => (None, 0),
            };

            if let Some(assign_op) = op {
                if prec >= min_prec {
                    self.advance();
                    let right = self.parse_expression()?;
                    left = Expression::Assignment(Box::new(AssignmentExpr {
                        target: left,
                        operator: assign_op,
                        value: right,
                        span: start.merge(self.current_span()),
                    }));
                    continue;
                }
            }

            // Binary operators
            let (bin_op, prec) = match &self.current.kind {
                TokenKind::OrOr => (Some(BinaryOp::Or), 2),
                TokenKind::AndAnd => (Some(BinaryOp::And), 3),
                TokenKind::Pipe => (Some(BinaryOp::BitwiseOr), 4),
                TokenKind::Caret => (Some(BinaryOp::BitwiseXor), 5),
                TokenKind::Amp => (Some(BinaryOp::BitwiseAnd), 6),
                TokenKind::EqEq => (Some(BinaryOp::Equal), 7),
                TokenKind::NotEq => (Some(BinaryOp::NotEqual), 7),
                TokenKind::EqEqEq => (Some(BinaryOp::ExactEqual), 7),
                TokenKind::NotEqEq => (Some(BinaryOp::ExactNotEqual), 7),
                TokenKind::Lt => (Some(BinaryOp::LessThan), 8),
                TokenKind::Gt => (Some(BinaryOp::GreaterThan), 8),
                TokenKind::LtEq => (Some(BinaryOp::LessOrEqual), 8),
                TokenKind::GtEq => (Some(BinaryOp::GreaterOrEqual), 8),
                TokenKind::LtLt => (Some(BinaryOp::LeftShift), 9),
                TokenKind::GtGt => (Some(BinaryOp::RightShift), 9),
                TokenKind::GtGtGt => (Some(BinaryOp::UnsignedRightShift), 9),
                TokenKind::Plus => (Some(BinaryOp::Add), 10),
                TokenKind::Minus => (Some(BinaryOp::Subtract), 10),
                TokenKind::Star => (Some(BinaryOp::Multiply), 11),
                TokenKind::Slash => (Some(BinaryOp::Divide), 11),
                TokenKind::Percent => (Some(BinaryOp::Modulo), 11),
                _ => (None, 0),
            };

            if let Some(binary_op) = bin_op {
                if prec > min_prec {
                    self.advance();
                    let right = self.parse_unary()?;
                    let right = self.parse_binary_rest(right, prec)?;
                    left = Expression::Binary(Box::new(BinaryExpr {
                        left,
                        operator: binary_op,
                        right,
                        span: start.merge(self.current_span()),
                    }));
                    continue;
                }
            }

            // Handle instanceof (same precedence as relational operators)
            if self.check(&TokenKind::Instanceof) {
                let prec = 8; // Same as relational operators
                if prec > min_prec {
                    self.advance();
                    let type_ref = self.parse_type_ref()?;
                    left = Expression::Instanceof(Box::new(InstanceofExpr {
                        expression: left,
                        type_ref,
                        span: start.merge(self.current_span()),
                    }));
                    continue;
                }
            }

            break;
        }

        Ok(left)
    }

    fn is_type_start(&self) -> bool {
        matches!(
            &self.current.kind,
            TokenKind::Void
                | TokenKind::Boolean
                | TokenKind::Integer
                | TokenKind::Long
                | TokenKind::Double
                | TokenKind::Decimal
                | TokenKind::StringType
                | TokenKind::Blob
                | TokenKind::Date
                | TokenKind::Datetime
                | TokenKind::Time
                | TokenKind::Id
                | TokenKind::Object
                | TokenKind::List
                | TokenKind::Set
                | TokenKind::Map
                | TokenKind::Identifier(_)
        )
    }

    fn is_literal(&self) -> bool {
        matches!(
            &self.current.kind,
            TokenKind::Null
                | TokenKind::True
                | TokenKind::False
                | TokenKind::IntegerLiteral(_)
                | TokenKind::LongLiteral(_)
                | TokenKind::DoubleLiteral(_)
                | TokenKind::StringLiteral(_)
        )
    }

    fn parse_expression_list(&mut self) -> ParseResult<Vec<Expression>> {
        let mut exprs = Vec::new();
        loop {
            exprs.push(self.parse_expression()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(exprs)
    }

    // ==================== Expressions ====================

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let expr = self.parse_ternary()?;

        let op = match &self.current.kind {
            TokenKind::Eq => Some(AssignmentOp::Assign),
            TokenKind::PlusEq => Some(AssignmentOp::AddAssign),
            TokenKind::MinusEq => Some(AssignmentOp::SubAssign),
            TokenKind::StarEq => Some(AssignmentOp::MulAssign),
            TokenKind::SlashEq => Some(AssignmentOp::DivAssign),
            TokenKind::PercentEq => Some(AssignmentOp::ModAssign),
            TokenKind::AmpEq => Some(AssignmentOp::AndAssign),
            TokenKind::PipeEq => Some(AssignmentOp::OrAssign),
            TokenKind::CaretEq => Some(AssignmentOp::XorAssign),
            TokenKind::LtLtEq => Some(AssignmentOp::LeftShiftAssign),
            TokenKind::GtGtEq => Some(AssignmentOp::RightShiftAssign),
            TokenKind::GtGtGtEq => Some(AssignmentOp::UnsignedRightShiftAssign),
            _ => None,
        };

        if let Some(operator) = op {
            self.advance();
            let value = self.parse_assignment()?;
            Ok(Expression::Assignment(Box::new(AssignmentExpr {
                target: expr,
                operator,
                value,
                span: start.merge(self.current_span()),
            })))
        } else {
            Ok(expr)
        }
    }

    fn parse_ternary(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let condition = self.parse_null_coalesce()?;

        if self.match_token(&TokenKind::Question) {
            let then_expr = self.parse_expression()?;
            self.consume(&TokenKind::Colon, ":")?;
            let else_expr = self.parse_ternary()?;

            Ok(Expression::Ternary(Box::new(TernaryExpr {
                condition,
                then_expr,
                else_expr,
                span: start.merge(self.current_span()),
            })))
        } else {
            Ok(condition)
        }
    }

    fn parse_null_coalesce(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_or()?;

        while self.match_token(&TokenKind::QuestionQuestion) {
            let right = self.parse_or()?;
            left = Expression::NullCoalesce(Box::new(NullCoalesceExpr {
                left,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_or(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_and()?;

        while self.match_token(&TokenKind::OrOr) {
            let right = self.parse_and()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::Or,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_or()?;

        while self.match_token(&TokenKind::AndAnd) {
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::And,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_xor()?;

        while self.match_token(&TokenKind::Pipe) {
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::BitwiseOr,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_bitwise_and()?;

        while self.match_token(&TokenKind::Caret) {
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::BitwiseXor,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_equality()?;

        while self.match_token(&TokenKind::Amp) {
            let right = self.parse_equality()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::BitwiseAnd,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_relational()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::EqEq => Some(BinaryOp::Equal),
                TokenKind::NotEq => Some(BinaryOp::NotEqual),
                TokenKind::EqEqEq => Some(BinaryOp::ExactEqual),
                TokenKind::NotEqEq => Some(BinaryOp::ExactNotEqual),
                _ => None,
            };

            if let Some(operator) = op {
                self.advance();
                let right = self.parse_relational()?;
                left = Expression::Binary(Box::new(BinaryExpr {
                    left,
                    operator,
                    right,
                    span: start.merge(self.current_span()),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_relational(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_shift()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Lt => Some(BinaryOp::LessThan),
                TokenKind::Gt => Some(BinaryOp::GreaterThan),
                TokenKind::LtEq => Some(BinaryOp::LessOrEqual),
                TokenKind::GtEq => Some(BinaryOp::GreaterOrEqual),
                _ => None,
            };

            if let Some(operator) = op {
                self.advance();
                let right = self.parse_shift()?;
                left = Expression::Binary(Box::new(BinaryExpr {
                    left,
                    operator,
                    right,
                    span: start.merge(self.current_span()),
                }));
            } else if self.match_token(&TokenKind::Instanceof) {
                let type_ref = self.parse_type_ref()?;
                left = Expression::Instanceof(Box::new(InstanceofExpr {
                    expression: left,
                    type_ref,
                    span: start.merge(self.current_span()),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_shift(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_additive()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::LtLt => Some(BinaryOp::LeftShift),
                TokenKind::GtGt => Some(BinaryOp::RightShift),
                TokenKind::GtGtGt => Some(BinaryOp::UnsignedRightShift),
                _ => None,
            };

            if let Some(operator) = op {
                self.advance();
                let right = self.parse_additive()?;
                left = Expression::Binary(Box::new(BinaryExpr {
                    left,
                    operator,
                    right,
                    span: start.merge(self.current_span()),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Plus => Some(BinaryOp::Add),
                TokenKind::Minus => Some(BinaryOp::Subtract),
                _ => None,
            };

            if let Some(operator) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expression::Binary(Box::new(BinaryExpr {
                    left,
                    operator,
                    right,
                    span: start.merge(self.current_span()),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_unary()?;

        loop {
            let op = match &self.current.kind {
                TokenKind::Star => Some(BinaryOp::Multiply),
                TokenKind::Slash => Some(BinaryOp::Divide),
                TokenKind::Percent => Some(BinaryOp::Modulo),
                _ => None,
            };

            if let Some(operator) = op {
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::Binary(Box::new(BinaryExpr {
                    left,
                    operator,
                    right,
                    span: start.merge(self.current_span()),
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();

        // Pre-increment/decrement
        if self.match_token(&TokenKind::PlusPlus) {
            let operand = self.parse_unary()?;
            return Ok(Expression::PreIncrement(
                Box::new(operand),
                start.merge(self.current_span()),
            ));
        }
        if self.match_token(&TokenKind::MinusMinus) {
            let operand = self.parse_unary()?;
            return Ok(Expression::PreDecrement(
                Box::new(operand),
                start.merge(self.current_span()),
            ));
        }

        // Unary operators
        let op = match &self.current.kind {
            TokenKind::Minus => Some(UnaryOp::Negate),
            TokenKind::Bang => Some(UnaryOp::Not),
            TokenKind::Tilde => Some(UnaryOp::BitwiseNot),
            _ => None,
        };

        if let Some(operator) = op {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expression::Unary(Box::new(UnaryExpr {
                operator,
                operand,
                span: start.merge(self.current_span()),
            })));
        }

        // Cast expression: (Type)expr
        if self.check(&TokenKind::LParen) {
            // Try to parse as cast
            if let Some(cast_expr) = self.try_parse_cast()? {
                // Cast result can have postfix operations: ((Type)expr).method()
                return self.parse_postfix_from(cast_expr);
            }
        }

        self.parse_postfix()
    }

    fn try_parse_cast(&mut self) -> ParseResult<Option<Expression>> {
        // Cast expression: (Type)expr
        // We need to distinguish between (Type)expr and (expr)
        // Strategy: Look at what's inside the parens - if it looks like a type followed by ),
        // then it's a cast.

        let start = self.current_span();

        // Save current position for potential backtracking
        // We can't really backtrack with logos, so we'll use heuristics

        // Check if we have ( followed by what looks like a type
        if !self.check(&TokenKind::LParen) {
            return Ok(None);
        }

        // Peek ahead to see if this is a cast
        // A cast looks like: (TypeName)expr or (TypeName<...>)expr
        // A parenthesized expression looks like: (expr op expr) or (identifier.method())

        // Heuristic: if we see (Identifier) followed by an expression-starting token,
        // it's likely a cast. But if we see (Identifier op ...) it's a parenthesized expr.

        // For simplicity, we'll look for patterns that are definitely casts:
        // (PrimitiveType)
        // (Identifier) followed by identifier, literal, or (

        self.advance(); // consume (

        // Check if the token after ( is a type
        let is_type_token = self.is_type_start();

        if !is_type_token {
            // Not a type, so this is a parenthesized expression
            // Parse the expression and return
            let expr = self.parse_expression()?;
            self.consume(&TokenKind::RParen, ")")?;
            return Ok(Some(Expression::Parenthesized(
                Box::new(expr),
                start.merge(self.current_span()),
            )));
        }

        // Parse the type
        let type_ref = self.parse_type_ref()?;

        // After type, we expect )
        if !self.check(&TokenKind::RParen) {
            // This isn't a cast - it's a parenthesized expression starting with an identifier
            // Convert the type back to an expression and continue parsing
            let expr = self.type_ref_to_expression(type_ref)?;
            let full_expr = self.parse_expression_continue(expr)?;
            self.consume(&TokenKind::RParen, ")")?;
            return Ok(Some(Expression::Parenthesized(
                Box::new(full_expr),
                start.merge(self.current_span()),
            )));
        }

        self.advance(); // consume )

        // Now check what follows - if it's an expression-starting token, this is a cast
        // Otherwise, it was a parenthesized type reference (which shouldn't happen)

        let is_expression_start = matches!(
            &self.current.kind,
            TokenKind::Identifier(_) | TokenKind::IntegerLiteral(_) | TokenKind::LongLiteral(_)
            | TokenKind::DoubleLiteral(_) | TokenKind::StringLiteral(_) | TokenKind::True
            | TokenKind::False | TokenKind::Null | TokenKind::This | TokenKind::Super
            | TokenKind::New | TokenKind::LParen | TokenKind::Bang | TokenKind::Minus
            | TokenKind::Plus | TokenKind::PlusPlus | TokenKind::MinusMinus | TokenKind::Tilde
            | TokenKind::LBracket | TokenKind::HexLiteral(_) | TokenKind::BinaryLiteral(_)
            | TokenKind::OctalLiteral(_)
            // Keywords that can start expressions (used as identifiers in some contexts)
            | TokenKind::Trigger | TokenKind::Map | TokenKind::List | TokenKind::Set
            | TokenKind::Object | TokenKind::Id | TokenKind::Date | TokenKind::Datetime
            | TokenKind::Time | TokenKind::Integer | TokenKind::Long | TokenKind::Double
            | TokenKind::Decimal | TokenKind::StringType | TokenKind::Boolean | TokenKind::Blob
        );

        if is_expression_start {
            // This is a cast expression
            let operand = self.parse_unary()?;
            Ok(Some(Expression::Cast(Box::new(CastExpr {
                type_ref,
                expression: operand,
                span: start.merge(self.current_span()),
            }))))
        } else {
            // This was just a parenthesized type reference - convert to identifier
            let expr = self.type_ref_to_expression(type_ref)?;
            Ok(Some(Expression::Parenthesized(
                Box::new(expr),
                start.merge(self.current_span()),
            )))
        }
    }

    /// Continue parsing an expression after we've already parsed the first part
    fn parse_expression_continue(&mut self, left: Expression) -> ParseResult<Expression> {
        // This handles cases where we parsed an identifier thinking it was a type,
        // but it's actually part of an expression like (a + b) or (a.method())

        // First, handle postfix operations (method calls, field access, etc.)
        let expr = self.parse_postfix_from(left)?;

        // Then handle binary operators
        let expr = self.parse_binary_rest(expr, 0)?;

        // Handle null coalesce (??)
        let start = expr.span();
        let mut expr = expr;
        while self.match_token(&TokenKind::QuestionQuestion) {
            let right = self.parse_unary()?;
            let right = self.parse_binary_rest(right, 0)?;
            expr = Expression::NullCoalesce(Box::new(NullCoalesceExpr {
                left: expr,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        // Handle ternary (? :)
        if self.match_token(&TokenKind::Question) {
            let then_expr = self.parse_expression()?;
            self.consume(&TokenKind::Colon, ":")?;
            let else_expr = self.parse_ternary()?;

            return Ok(Expression::Ternary(Box::new(TernaryExpr {
                condition: expr,
                then_expr,
                else_expr,
                span: start.merge(self.current_span()),
            })));
        }

        Ok(expr)
    }

    fn parse_postfix(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_primary()?;
        self.parse_postfix_from(expr)
    }

    /// Parse postfix operations (field access, method calls, array access, etc.)
    /// starting from an already-parsed expression
    fn parse_postfix_from(&mut self, initial: Expression) -> ParseResult<Expression> {
        let start = initial.span();
        let mut expr = initial;

        loop {
            match &self.current.kind {
                TokenKind::Dot => {
                    self.advance();
                    let name = self.parse_identifier()?;

                    // Check if it's a method call
                    if self.check(&TokenKind::LParen) {
                        self.advance();
                        let arguments = self.parse_arguments()?;
                        self.consume(&TokenKind::RParen, ")")?;

                        expr = Expression::MethodCall(Box::new(MethodCallExpr {
                            object: Some(expr),
                            name,
                            type_arguments: Vec::new(),
                            arguments,
                            span: start.merge(self.current_span()),
                        }));
                    } else {
                        expr = Expression::FieldAccess(Box::new(FieldAccessExpr {
                            object: expr,
                            field: name,
                            span: start.merge(self.current_span()),
                        }));
                    }
                }
                TokenKind::QuestionDot => {
                    self.advance();
                    let name = self.parse_identifier()?;

                    // Check if it's a safe method call obj?.method()
                    if self.check(&TokenKind::LParen) {
                        self.advance();
                        let arguments = self.parse_arguments()?;
                        self.consume(&TokenKind::RParen, ")")?;

                        // Create a safe navigation expression, then call method on it
                        // We represent obj?.method(args) as MethodCall with a SafeNavigation object
                        let safe_obj = Expression::SafeNavigation(Box::new(SafeNavigationExpr {
                            object: expr,
                            field: name.clone(),
                            span: start.merge(self.current_span()),
                        }));
                        expr = Expression::MethodCall(Box::new(MethodCallExpr {
                            object: Some(safe_obj),
                            name,
                            type_arguments: Vec::new(),
                            arguments,
                            span: start.merge(self.current_span()),
                        }));
                    } else {
                        expr = Expression::SafeNavigation(Box::new(SafeNavigationExpr {
                            object: expr,
                            field: name,
                            span: start.merge(self.current_span()),
                        }));
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.consume(&TokenKind::RBracket, "]")?;
                    expr = Expression::ArrayAccess(Box::new(ArrayAccessExpr {
                        array: expr,
                        index,
                        span: start.merge(self.current_span()),
                    }));
                }
                TokenKind::PlusPlus => {
                    self.advance();
                    expr =
                        Expression::PostIncrement(Box::new(expr), start.merge(self.current_span()));
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    expr =
                        Expression::PostDecrement(Box::new(expr), start.merge(self.current_span()));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();

        match &self.current.kind {
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Null(start))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Boolean(true, start))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Boolean(false, start))
            }
            TokenKind::IntegerLiteral(n)
            | TokenKind::HexLiteral(n)
            | TokenKind::BinaryLiteral(n)
            | TokenKind::OctalLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Integer(n, start))
            }
            TokenKind::LongLiteral(n) | TokenKind::HexLongLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Long(n, start))
            }
            TokenKind::DoubleLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Double(n, start))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::String(s, start))
            }
            TokenKind::This => {
                self.advance();
                Ok(Expression::This(start))
            }
            TokenKind::Super => {
                self.advance();
                Ok(Expression::Super(start))
            }
            TokenKind::New => {
                self.parse_new_expression()
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&TokenKind::RParen, ")")?;
                Ok(Expression::Parenthesized(
                    Box::new(expr),
                    start.merge(self.current_span()),
                ))
            }
            TokenKind::LBracket => {
                // SOQL query or array initializer
                self.parse_soql_or_array()
            }
            TokenKind::Identifier(_)
            | TokenKind::Id
            | TokenKind::First
            | TokenKind::Last
            | TokenKind::Order
            | TokenKind::Group
            | TokenKind::Limit
            | TokenKind::Offset
            | TokenKind::Date
            | TokenKind::Time
            | TokenKind::Trigger
            | TokenKind::Object
            | TokenKind::Set
            | TokenKind::Map
            | TokenKind::List
            // Type keywords that can be used as class names (for static method calls like String.isBlank())
            | TokenKind::StringType
            | TokenKind::Integer
            | TokenKind::Long
            | TokenKind::Double
            | TokenKind::Decimal
            | TokenKind::Boolean
            | TokenKind::Datetime
            | TokenKind::Blob => {
                let name = self.parse_identifier()?;

                // Check for method call
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    self.consume(&TokenKind::RParen, ")")?;

                    Ok(Expression::MethodCall(Box::new(MethodCallExpr {
                        object: None,
                        name,
                        type_arguments: Vec::new(),
                        arguments,
                        span: start.merge(self.current_span()),
                    })))
                } else {
                    Ok(Expression::Identifier(name, start))
                }
            }
            _ => Err(ParseError::InvalidExpression(start)),
        }
    }

    fn parse_new_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        self.consume(&TokenKind::New, "new")?;

        // Use parse_type_ref_full to consume full qualified names like Outer.Inner
        let type_ref = self.parse_type_ref_full()?;

        // Check for array creation or constructor call
        if self.match_token(&TokenKind::LBracket) {
            // Array creation
            if self.check(&TokenKind::RBracket) {
                // new Type[] or new Type[]{...}
                self.advance();
                let initializer = if self.match_token(&TokenKind::LBrace) {
                    let items = self.parse_array_initializer()?;
                    self.consume(&TokenKind::RBrace, "}")?;
                    Some(items)
                } else {
                    None
                };

                Ok(Expression::NewArray(Box::new(NewArrayExpr {
                    element_type: type_ref,
                    size: None,
                    initializer,
                    span: start.merge(self.current_span()),
                })))
            } else {
                // new Type[size]
                let size = self.parse_expression()?;
                self.consume(&TokenKind::RBracket, "]")?;

                Ok(Expression::NewArray(Box::new(NewArrayExpr {
                    element_type: type_ref,
                    size: Some(size),
                    initializer: None,
                    span: start.merge(self.current_span()),
                })))
            }
        } else if self.match_token(&TokenKind::LBrace) {
            // Map/Set/List literal initializer: new Map<K,V>{...}
            if type_ref.name == "Map" || type_ref.name.ends_with(".Map") {
                let initializer = self.parse_map_initializer()?;
                self.consume(&TokenKind::RBrace, "}")?;

                Ok(Expression::NewMap(Box::new(NewMapExpr {
                    type_ref,
                    initializer: Some(initializer),
                    span: start.merge(self.current_span()),
                })))
            } else {
                // List or Set literal
                let items = self.parse_array_initializer()?;
                self.consume(&TokenKind::RBrace, "}")?;

                if type_ref.name == "Set" || type_ref.name.ends_with(".Set") {
                    Ok(Expression::SetLiteral(
                        items,
                        start.merge(self.current_span()),
                    ))
                } else {
                    Ok(Expression::ListLiteral(
                        items,
                        start.merge(self.current_span()),
                    ))
                }
            }
        } else {
            // Constructor call
            self.consume(&TokenKind::LParen, "(")?;
            let arguments = self.parse_arguments()?;
            self.consume(&TokenKind::RParen, ")")?;

            Ok(Expression::New(Box::new(NewExpr {
                type_ref,
                arguments,
                span: start.merge(self.current_span()),
            })))
        }
    }

    fn parse_arguments(&mut self) -> ParseResult<Vec<Expression>> {
        let mut args = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_array_initializer(&mut self) -> ParseResult<Vec<Expression>> {
        let mut items = Vec::new();

        if self.check(&TokenKind::RBrace) {
            return Ok(items);
        }

        loop {
            items.push(self.parse_expression()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
            // Allow trailing comma
            if self.check(&TokenKind::RBrace) {
                break;
            }
        }

        Ok(items)
    }

    fn parse_map_initializer(&mut self) -> ParseResult<Vec<(Expression, Expression)>> {
        let mut items = Vec::new();

        if self.check(&TokenKind::RBrace) {
            return Ok(items);
        }

        loop {
            let key = self.parse_expression()?;
            self.consume(&TokenKind::Arrow, "=>")?;
            let value = self.parse_expression()?;
            items.push((key, value));

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
            // Allow trailing comma
            if self.check(&TokenKind::RBrace) {
                break;
            }
        }

        Ok(items)
    }

    fn parse_soql_or_array(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        self.consume(&TokenKind::LBracket, "[")?;

        // Check if this looks like SOQL (starts with SELECT)
        if self.check(&TokenKind::Select) {
            let query = self.parse_soql_query()?;
            self.consume(&TokenKind::RBracket, "]")?;
            Ok(Expression::Soql(Box::new(query)))
        }
        // Check if this looks like SOSL (starts with FIND)
        else if self.check(&TokenKind::Find) {
            let query = self.parse_sosl_query()?;
            self.consume(&TokenKind::RBracket, "]")?;
            Ok(Expression::Sosl(Box::new(query)))
        } else {
            // Array initializer
            let items = self.parse_array_initializer()?;
            self.consume(&TokenKind::RBracket, "]")?;
            Ok(Expression::ListLiteral(
                items,
                start.merge(self.current_span()),
            ))
        }
    }

    fn parse_soql_query(&mut self) -> ParseResult<SoqlQuery> {
        let start = self.current_span();
        self.consume(&TokenKind::Select, "SELECT")?;

        // Parse SELECT fields
        let select_clause = self.parse_select_fields()?;

        // FROM clause
        self.consume(&TokenKind::From, "FROM")?;
        let from_clause = self.parse_soql_identifier()?;

        // Optional WHERE clause
        let where_clause = if self.match_token(&TokenKind::Where) {
            Some(self.parse_soql_condition()?)
        } else {
            None
        };

        // Optional WITH clause (WITH SECURITY_ENFORCED, WITH USER_MODE, WITH SYSTEM_MODE)
        let with_clause = self.parse_soql_with_clause()?;

        // Optional GROUP BY clause
        let group_by_clause = if self.match_token(&TokenKind::Group) {
            self.consume(&TokenKind::By, "BY")?;
            self.parse_group_by_fields()?
        } else {
            Vec::new()
        };

        // Optional HAVING clause (only valid with GROUP BY)
        let having_clause = if self.match_token(&TokenKind::Having) {
            Some(self.parse_soql_condition()?)
        } else {
            None
        };

        // Optional ORDER BY clause
        let order_by_clause = if self.match_token(&TokenKind::Order) {
            self.consume(&TokenKind::By, "BY")?;
            self.parse_order_by_fields()?
        } else {
            Vec::new()
        };

        // Optional LIMIT clause
        let limit_clause = if self.match_token(&TokenKind::Limit) {
            Some(self.parse_soql_expression()?)
        } else {
            None
        };

        // Optional OFFSET clause
        let offset_clause = if self.match_token(&TokenKind::Offset) {
            Some(self.parse_soql_expression()?)
        } else {
            None
        };

        // Optional FOR clause (FOR UPDATE, FOR VIEW, FOR REFERENCE)
        let for_clause = if self.check(&TokenKind::For) {
            self.advance();
            if self.match_token(&TokenKind::Update) {
                Some(ForClause::Update)
            } else if self.check(&TokenKind::Identifier(String::new())) {
                // VIEW and REFERENCE are not keywords, check identifier
                let id = if let TokenKind::Identifier(s) = &self.current.kind {
                    s.to_lowercase()
                } else {
                    String::new()
                };
                if id == "view" {
                    self.advance();
                    Some(ForClause::View)
                } else if id == "reference" {
                    self.advance();
                    Some(ForClause::Reference)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(SoqlQuery {
            select_clause,
            from_clause,
            where_clause,
            with_clause,
            group_by_clause,
            having_clause,
            order_by_clause,
            limit_clause,
            offset_clause,
            for_clause,
            span: start.merge(self.current_span()),
        })
    }

    fn parse_group_by_fields(&mut self) -> ParseResult<Vec<String>> {
        let mut fields = Vec::new();
        loop {
            let field = self.parse_soql_field_path()?;
            fields.push(field);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Ok(fields)
    }

    fn parse_soql_with_clause(&mut self) -> ParseResult<Option<SoqlWithClause>> {
        // Check for WITH keyword (as identifier since it's not a reserved token)
        if let TokenKind::Identifier(s) = &self.current.kind {
            if s.to_lowercase() == "with" {
                self.advance();
                // Parse the WITH clause type
                if let TokenKind::Identifier(clause_type) = &self.current.kind {
                    let clause = match clause_type.to_lowercase().as_str() {
                        "security_enforced" => {
                            self.advance();
                            Some(SoqlWithClause::SecurityEnforced)
                        }
                        "user_mode" => {
                            self.advance();
                            Some(SoqlWithClause::UserMode)
                        }
                        "system_mode" => {
                            self.advance();
                            Some(SoqlWithClause::SystemMode)
                        }
                        _ => None,
                    };
                    return Ok(clause);
                }
            }
        }
        Ok(None)
    }

    fn parse_select_fields(&mut self) -> ParseResult<Vec<SelectField>> {
        let mut fields = Vec::new();

        loop {
            // Check for subquery: (SELECT ... FROM ...)
            if self.check(&TokenKind::LParen) {
                self.advance();
                let subquery = self.parse_soql_query()?;
                self.consume(&TokenKind::RParen, ")")?;
                fields.push(SelectField::SubQuery(Box::new(subquery)));
            }
            // Check for aggregate function: COUNT(), SUM(field), etc.
            else if self.is_aggregate_function() {
                let func = self.parse_aggregate_function()?;
                fields.push(func);
            }
            // Check for TYPEOF
            else if self.check(&TokenKind::Identifier(String::new())) {
                if let TokenKind::Identifier(s) = &self.current.kind {
                    if s.to_lowercase() == "typeof" {
                        let typeof_clause = self.parse_typeof_clause()?;
                        fields.push(SelectField::TypeOf(typeof_clause));
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                        continue;
                    }
                }
                // Regular field or relationship field (e.g., Account.Name, Contact__r.Email)
                let name = self.parse_soql_field_path()?;
                fields.push(SelectField::Field(name));
            } else {
                // Regular field
                let name = self.parse_soql_field_path()?;
                fields.push(SelectField::Field(name));
            }

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    fn is_aggregate_function(&self) -> bool {
        if let TokenKind::Identifier(s) = &self.current.kind {
            let lower = s.to_lowercase();
            matches!(
                lower.as_str(),
                "count" | "sum" | "avg" | "min" | "max" | "count_distinct"
            )
        } else {
            false
        }
    }

    fn parse_aggregate_function(&mut self) -> ParseResult<SelectField> {
        let name = if let TokenKind::Identifier(s) = &self.current.kind {
            s.clone()
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "aggregate function".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            });
        };
        self.advance();

        self.consume(&TokenKind::LParen, "(")?;

        // COUNT() can have no argument or an argument
        let field = if self.check(&TokenKind::RParen) {
            String::new() // COUNT() with no argument
        } else {
            self.parse_soql_field_path()?
        };

        self.consume(&TokenKind::RParen, ")")?;

        // Optional alias
        let alias = if let TokenKind::Identifier(s) = &self.current.kind {
            // Check if it's an alias (not a keyword)
            if !self.is_soql_keyword() {
                let a = s.clone();
                self.advance();
                Some(a)
            } else {
                None
            }
        } else {
            None
        };

        Ok(SelectField::AggregateFunction { name, field, alias })
    }

    fn is_soql_keyword(&self) -> bool {
        matches!(
            &self.current.kind,
            TokenKind::From
                | TokenKind::Where
                | TokenKind::Group
                | TokenKind::Having
                | TokenKind::Order
                | TokenKind::Limit
                | TokenKind::Offset
                | TokenKind::For
        )
    }

    fn parse_typeof_clause(&mut self) -> ParseResult<TypeOfClause> {
        // Skip "TYPEOF"
        self.advance();

        let field = self.parse_soql_identifier()?;

        let mut when_clauses = Vec::new();
        let mut else_fields = None;

        loop {
            if self.match_token(&TokenKind::When) {
                let type_name = self.parse_soql_identifier()?;
                self.consume(&TokenKind::Identifier("THEN".to_string()), "THEN")?;
                let fields = self.parse_typeof_fields()?;
                when_clauses.push(TypeOfWhen { type_name, fields });
            } else if self.match_token(&TokenKind::Else) {
                else_fields = Some(self.parse_typeof_fields()?);
                break;
            } else {
                break;
            }
        }

        // END keyword
        if let TokenKind::Identifier(s) = &self.current.kind {
            if s.to_lowercase() == "end" {
                self.advance();
            }
        }

        Ok(TypeOfClause {
            field,
            when_clauses,
            else_fields,
        })
    }

    fn parse_typeof_fields(&mut self) -> ParseResult<Vec<String>> {
        let mut fields = Vec::new();
        loop {
            let field = self.parse_soql_identifier()?;
            fields.push(field);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
            // Check if next token is a keyword that ends the field list
            if matches!(&self.current.kind, TokenKind::When | TokenKind::Else)
                || (matches!(&self.current.kind, TokenKind::Identifier(s) if s.to_lowercase() == "end"))
            {
                break;
            }
        }
        Ok(fields)
    }

    /// Parse a SOQL field path like "Name", "Account.Name", "Contact__r.Account.Name"
    fn parse_soql_field_path(&mut self) -> ParseResult<String> {
        let mut path = self.parse_soql_identifier()?;

        while self.match_token(&TokenKind::Dot) {
            let next = self.parse_soql_identifier()?;
            path.push('.');
            path.push_str(&next);
        }

        Ok(path)
    }

    /// Parse a SOQL condition expression (WHERE or HAVING clause)
    /// This handles bind variables like :varName
    fn parse_soql_condition(&mut self) -> ParseResult<Expression> {
        self.parse_soql_or_expression()
    }

    fn parse_soql_or_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_soql_and_expression()?;

        while self.match_token(&TokenKind::Or) {
            let right = self.parse_soql_and_expression()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::Or,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_soql_and_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        let mut left = self.parse_soql_not_expression()?;

        while self.match_token(&TokenKind::And) {
            let right = self.parse_soql_not_expression()?;
            left = Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::And,
                right,
                span: start.merge(self.current_span()),
            }));
        }

        Ok(left)
    }

    fn parse_soql_not_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();
        if self.match_token(&TokenKind::Not) {
            let expr = self.parse_soql_not_expression()?;
            return Ok(Expression::Unary(Box::new(UnaryExpr {
                operator: UnaryOp::Not,
                operand: expr,
                span: start.merge(self.current_span()),
            })));
        }
        self.parse_soql_comparison()
    }

    fn parse_soql_comparison(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();

        // Handle parenthesized expressions
        if self.check(&TokenKind::LParen) {
            self.advance();
            let expr = self.parse_soql_condition()?;
            self.consume(&TokenKind::RParen, ")")?;
            return Ok(expr);
        }

        let left = self.parse_soql_expression()?;

        // Comparison operators
        let operator = if self.match_token(&TokenKind::EqEq) || self.match_token(&TokenKind::Eq) {
            Some(BinaryOp::Equal)
        } else if self.match_token(&TokenKind::NotEq) || self.match_token(&TokenKind::LtGt) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(&TokenKind::Lt) {
            Some(BinaryOp::LessThan)
        } else if self.match_token(&TokenKind::Gt) {
            Some(BinaryOp::GreaterThan)
        } else if self.match_token(&TokenKind::LtEq) {
            Some(BinaryOp::LessOrEqual)
        } else if self.match_token(&TokenKind::GtEq) {
            Some(BinaryOp::GreaterOrEqual)
        } else if self.match_token(&TokenKind::Like) {
            Some(BinaryOp::Like)
        } else if self.match_token(&TokenKind::In) {
            // IN operator - parse list of values
            self.consume(&TokenKind::LParen, "(")?;
            let mut values = Vec::new();
            loop {
                values.push(self.parse_soql_expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.consume(&TokenKind::RParen, ")")?;
            // Create a special IN expression using Binary with a list
            return Ok(Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: BinaryOp::In,
                right: Expression::NewArray(Box::new(NewArrayExpr {
                    element_type: TypeRef {
                        name: "Object".to_string(),
                        type_arguments: Vec::new(),
                        is_array: false,
                        span: start,
                    },
                    size: None,
                    initializer: Some(values),
                    span: start.merge(self.current_span()),
                })),
                span: start.merge(self.current_span()),
            })));
        } else if self.match_token(&TokenKind::Not) {
            // NOT IN
            if self.match_token(&TokenKind::In) {
                self.consume(&TokenKind::LParen, "(")?;
                let mut values = Vec::new();
                loop {
                    values.push(self.parse_soql_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.consume(&TokenKind::RParen, ")")?;
                return Ok(Expression::Binary(Box::new(BinaryExpr {
                    left,
                    operator: BinaryOp::NotIn,
                    right: Expression::NewArray(Box::new(NewArrayExpr {
                        element_type: TypeRef {
                            name: "Object".to_string(),
                            type_arguments: Vec::new(),
                            is_array: false,
                            span: start,
                        },
                        size: None,
                        initializer: Some(values),
                        span: start.merge(self.current_span()),
                    })),
                    span: start.merge(self.current_span()),
                })));
            }
            return Err(ParseError::UnexpectedToken {
                expected: "IN".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            });
        } else if self.match_token(&TokenKind::Includes) {
            Some(BinaryOp::Includes)
        } else if self.match_token(&TokenKind::Excludes) {
            Some(BinaryOp::Excludes)
        } else {
            None
        };

        if let Some(op) = operator {
            let right = self.parse_soql_expression()?;
            Ok(Expression::Binary(Box::new(BinaryExpr {
                left,
                operator: op,
                right,
                span: start.merge(self.current_span()),
            })))
        } else {
            Ok(left)
        }
    }

    /// Parse a SOQL expression (can include bind variables)
    fn parse_soql_expression(&mut self) -> ParseResult<Expression> {
        let start = self.current_span();

        // Check for bind variable :varName
        if self.match_token(&TokenKind::Colon) {
            let var_name = self.parse_identifier()?;
            return Ok(Expression::BindVariable(
                var_name,
                start.merge(self.current_span()),
            ));
        }

        // Check for date literals like TODAY, LAST_N_DAYS:n
        if let TokenKind::Identifier(s) = &self.current.kind {
            let lower = s.to_lowercase();
            if is_soql_date_literal(&lower) {
                let literal = s.clone();
                self.advance();

                // Check for :n suffix (e.g., LAST_N_DAYS:30)
                if self.match_token(&TokenKind::Colon) {
                    if let TokenKind::IntegerLiteral(n) = &self.current.kind {
                        let n = *n;
                        self.advance();
                        return Ok(Expression::Identifier(
                            format!("{}:{}", literal, n),
                            start.merge(self.current_span()),
                        ));
                    }
                }

                return Ok(Expression::Identifier(
                    literal,
                    start.merge(self.current_span()),
                ));
            }
        }

        // Parse regular expression (literals, field paths, etc.)
        match &self.current.kind {
            TokenKind::IntegerLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Integer(n, start))
            }
            TokenKind::LongLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Long(n, start))
            }
            TokenKind::DoubleLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expression::Double(n, start))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::String(s, start))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Boolean(true, start))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Boolean(false, start))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Null(start))
            }
            _ => {
                // Field path
                let path = self.parse_soql_field_path()?;
                Ok(Expression::Identifier(
                    path,
                    start.merge(self.current_span()),
                ))
            }
        }
    }

    /// Parse an identifier in SOQL context, where many keywords can be used as field names
    fn parse_soql_identifier(&mut self) -> ParseResult<String> {
        let name = match &self.current.kind {
            TokenKind::Identifier(name) => name.clone(),
            // Keywords that can be field names in SOQL
            TokenKind::Id => "Id".to_string(),
            TokenKind::Date => "Date".to_string(),
            TokenKind::Time => "Time".to_string(),
            TokenKind::Object => "Object".to_string(),
            TokenKind::Order => "Order".to_string(),
            TokenKind::Group => "Group".to_string(),
            TokenKind::Limit => "Limit".to_string(),
            TokenKind::Offset => "Offset".to_string(),
            TokenKind::First => "First".to_string(),
            TokenKind::Last => "Last".to_string(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: format!("{:?}", self.current.kind),
                    span: self.current.span,
                });
            }
        };
        self.advance();
        Ok(name)
    }

    fn parse_order_by_fields(&mut self) -> ParseResult<Vec<OrderByField>> {
        let mut fields = Vec::new();

        loop {
            // Use parse_soql_field_path to support dotted paths like Account.Name
            let field = self.parse_soql_field_path()?;
            let ascending = if self.match_token(&TokenKind::Desc) {
                false
            } else {
                self.match_token(&TokenKind::Asc);
                true
            };

            let nulls_first = if self.match_token(&TokenKind::Nulls) {
                if self.match_token(&TokenKind::First) {
                    Some(true)
                } else if self.match_token(&TokenKind::Last) {
                    Some(false)
                } else {
                    None
                }
            } else {
                None
            };

            fields.push(OrderByField {
                field,
                ascending,
                nulls_first,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    // ==================== SOSL Parsing ====================

    fn parse_sosl_query(&mut self) -> ParseResult<SoslQuery> {
        let start = self.current_span();
        self.consume(&TokenKind::Find, "FIND")?;

        // Parse search term - it's typically a string literal or braces with search expression
        let search_term = if let TokenKind::StringLiteral(s) = &self.current.kind {
            let s = s.clone();
            self.advance();
            s
        } else if self.check(&TokenKind::LBrace) {
            self.advance();
            // Parse until closing brace
            let mut term = String::new();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                if let TokenKind::Identifier(s) = &self.current.kind {
                    if !term.is_empty() {
                        term.push(' ');
                    }
                    term.push_str(s);
                } else if let TokenKind::StringLiteral(s) = &self.current.kind {
                    if !term.is_empty() {
                        term.push(' ');
                    }
                    term.push_str(s);
                }
                self.advance();
            }
            self.consume(&TokenKind::RBrace, "}")?;
            term
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "search term".to_string(),
                found: format!("{:?}", self.current.kind),
                span: self.current.span,
            });
        };

        // Optional IN clause: IN ALL FIELDS, IN NAME FIELDS, etc.
        let search_group = if self.match_token(&TokenKind::In) {
            self.parse_sosl_search_group()?
        } else {
            None
        };

        // Optional RETURNING clause
        let returning = if self.match_token(&TokenKind::Returning) {
            self.parse_sosl_returning()?
        } else {
            Vec::new()
        };

        // Optional WITH clauses (simplified)
        let mut with_clauses = Vec::new();
        while self.check(&TokenKind::Identifier(String::new())) {
            if let TokenKind::Identifier(s) = &self.current.kind {
                if s.to_lowercase() == "with" {
                    self.advance();
                    if let Some(clause) = self.parse_sosl_with_clause()? {
                        with_clauses.push(clause);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Optional LIMIT clause
        let limit_clause = if self.match_token(&TokenKind::Limit) {
            Some(self.parse_soql_expression()?)
        } else {
            None
        };

        Ok(SoslQuery {
            search_term,
            search_group,
            returning,
            with_clauses,
            limit_clause,
            span: start.merge(self.current_span()),
        })
    }

    fn parse_sosl_search_group(&mut self) -> ParseResult<Option<SearchGroup>> {
        // ALL FIELDS, NAME FIELDS, EMAIL FIELDS, PHONE FIELDS, SIDEBAR FIELDS
        if let TokenKind::Identifier(s) = &self.current.kind {
            let lower = s.to_lowercase();
            if lower == "all"
                || lower == "name"
                || lower == "email"
                || lower == "phone"
                || lower == "sidebar"
            {
                let group_type = lower.clone();
                self.advance();

                // Expect FIELDS keyword
                if let TokenKind::Identifier(s2) = &self.current.kind {
                    if s2.to_lowercase() == "fields" {
                        self.advance();
                        return Ok(Some(match group_type.as_str() {
                            "all" => SearchGroup::AllFields,
                            "name" => SearchGroup::NameFields,
                            "email" => SearchGroup::EmailFields,
                            "phone" => SearchGroup::PhoneFields,
                            "sidebar" => SearchGroup::SidebarFields,
                            _ => return Ok(None),
                        }));
                    }
                }
            }
        }
        Ok(None)
    }

    fn parse_sosl_returning(&mut self) -> ParseResult<Vec<SoslReturning>> {
        let mut returning = Vec::new();

        loop {
            let object = self.parse_identifier()?;

            // Optional fields in parentheses
            let (fields, where_clause, order_by, limit_clause) = if self.check(&TokenKind::LParen) {
                self.advance();

                // Parse field list
                let mut fields = Vec::new();
                while !self.check(&TokenKind::RParen)
                    && !self.check(&TokenKind::Where)
                    && !self.check(&TokenKind::Order)
                    && !self.check(&TokenKind::Limit)
                {
                    let field = self.parse_soql_field_path()?;
                    fields.push(field);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }

                // Optional WHERE
                let where_clause = if self.match_token(&TokenKind::Where) {
                    Some(self.parse_soql_condition()?)
                } else {
                    None
                };

                // Optional ORDER BY
                let order_by = if self.match_token(&TokenKind::Order) {
                    self.consume(&TokenKind::By, "BY")?;
                    self.parse_order_by_fields()?
                } else {
                    Vec::new()
                };

                // Optional LIMIT
                let limit_clause = if self.match_token(&TokenKind::Limit) {
                    if let TokenKind::IntegerLiteral(n) = &self.current.kind {
                        let n = *n;
                        self.advance();
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                };

                self.consume(&TokenKind::RParen, ")")?;
                (fields, where_clause, order_by, limit_clause)
            } else {
                (Vec::new(), None, Vec::new(), None)
            };

            returning.push(SoslReturning {
                object,
                fields,
                where_clause,
                order_by,
                limit_clause,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(returning)
    }

    fn parse_sosl_with_clause(&mut self) -> ParseResult<Option<SoslWithClause>> {
        if let TokenKind::Identifier(s) = &self.current.kind {
            let lower = s.to_lowercase();
            match lower.as_str() {
                "snippet" => {
                    self.advance();
                    Ok(Some(SoslWithClause::Snippet))
                }
                "spellcorrection" => {
                    self.advance();
                    Ok(Some(SoslWithClause::SpellCorrection))
                }
                "network" => {
                    self.advance();
                    self.consume(&TokenKind::Eq, "=")?;
                    let network = self.parse_identifier()?;
                    Ok(Some(SoslWithClause::Network(network)))
                }
                "data" => {
                    // DATA CATEGORY
                    self.advance();
                    if let TokenKind::Identifier(s2) = &self.current.kind {
                        if s2.to_lowercase() == "category" {
                            self.advance();
                            let group = self.parse_identifier()?;
                            // Skip the comparison operator (AT, ABOVE, BELOW, etc.)
                            self.advance();
                            let category = self.parse_identifier()?;
                            return Ok(Some(SoslWithClause::DataCategory(group, category)));
                        }
                    }
                    Ok(None)
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}

/// Check if an identifier is a SOQL date literal
fn is_soql_date_literal(s: &str) -> bool {
    matches!(
        s,
        "yesterday"
            | "today"
            | "tomorrow"
            | "last_week"
            | "this_week"
            | "next_week"
            | "last_month"
            | "this_month"
            | "next_month"
            | "last_90_days"
            | "next_90_days"
            | "last_n_days"
            | "next_n_days"
            | "last_n_weeks"
            | "next_n_weeks"
            | "last_n_months"
            | "next_n_months"
            | "last_n_quarters"
            | "next_n_quarters"
            | "last_n_years"
            | "next_n_years"
            | "last_n_fiscal_quarters"
            | "next_n_fiscal_quarters"
            | "last_n_fiscal_years"
            | "next_n_fiscal_years"
            | "this_quarter"
            | "last_quarter"
            | "next_quarter"
            | "this_year"
            | "last_year"
            | "next_year"
            | "this_fiscal_quarter"
            | "last_fiscal_quarter"
            | "next_fiscal_quarter"
            | "this_fiscal_year"
            | "last_fiscal_year"
            | "next_fiscal_year"
    )
}

/// Parse an Apex source string into a CompilationUnit
pub fn parse(source: &str) -> ParseResult<CompilationUnit> {
    let mut parser = Parser::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_class() {
        let source = r#"
            public class MyClass {
                public void doSomething() {
                    return;
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let cu = result.unwrap();
        assert_eq!(cu.declarations.len(), 1);

        if let TypeDeclaration::Class(class) = &cu.declarations[0] {
            assert_eq!(class.name, "MyClass");
            assert_eq!(class.modifiers.access, AccessModifier::Public);
            assert_eq!(class.members.len(), 1);
        } else {
            panic!("Expected class declaration");
        }
    }

    #[test]
    fn test_parse_class_with_fields() {
        let source = r#"
            public class Account {
                public String name;
                private Integer count = 0;
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
            public enum Status {
                PENDING,
                ACTIVE,
                CLOSED
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let cu = result.unwrap();

        if let TypeDeclaration::Enum(e) = &cu.declarations[0] {
            assert_eq!(e.name, "Status");
            assert_eq!(e.values, vec!["PENDING", "ACTIVE", "CLOSED"]);
        } else {
            panic!("Expected enum declaration");
        }
    }

    #[test]
    fn test_parse_interface() {
        let source = r#"
            public interface Callable {
                void call();
                String getName();
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expressions() {
        let source = r#"
            public class Test {
                public void test() {
                    Integer x = 1 + 2 * 3;
                    Boolean b = x > 5 && x < 10;
                    String s = 'hello' + 'world';
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_control_flow() {
        let source = r#"
            public class Test {
                public void test() {
                    if (x > 0) {
                        return;
                    } else {
                        throw new Exception('error');
                    }

                    for (Integer i = 0; i < 10; i++) {
                        continue;
                    }

                    while (true) {
                        break;
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_for_each() {
        let source = r#"
            public class Test {
                public void test() {
                    for (Account a : accounts) {
                        System.debug(a.Name);
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_try_catch() {
        let source = r#"
            public class Test {
                public void test() {
                    try {
                        doSomething();
                    } catch (Exception e) {
                        System.debug(e);
                    } finally {
                        cleanup();
                    }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_parse_soql() {
        let source = r#"
            public class Test {
                public void test() {
                    List<Account> accounts = [SELECT Id, Name FROM Account WHERE Name != null LIMIT 10];
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok(), "Parse error: {:?}", result.err());
    }

    #[test]
    fn test_parse_annotations() {
        let source = r#"
            @isTest
            public class TestClass {
                @isTest
                static void testMethod1() {
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
        let cu = result.unwrap();

        if let TypeDeclaration::Class(class) = &cu.declarations[0] {
            assert_eq!(class.annotations.len(), 1);
            assert_eq!(class.annotations[0].name, "isTest");
        }
    }

    #[test]
    fn test_parse_generics() {
        let source = r#"
            public class Test {
                public void test() {
                    List<String> strings = new List<String>();
                    Map<String, Integer> counts = new Map<String, Integer>();
                    Set<Id> ids = new Set<Id>();
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_property() {
        let source = r#"
            public class Test {
                public String Name { get; set; }
                public Integer Count {
                    get { return count; }
                    private set { count = value; }
                }
            }
        "#;

        let result = parse(source);
        assert!(result.is_ok());
    }
}
