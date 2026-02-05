use apexrust::{tokenize, TokenKind};

// ==================== Keyword Tests ====================

#[test]
fn test_access_modifiers() {
    let tokens = tokenize("public private protected global");
    assert_eq!(tokens[0].kind, TokenKind::Public);
    assert_eq!(tokens[1].kind, TokenKind::Private);
    assert_eq!(tokens[2].kind, TokenKind::Protected);
    assert_eq!(tokens[3].kind, TokenKind::Global);
}

#[test]
fn test_class_modifiers() {
    let tokens = tokenize("abstract virtual static final transient override");
    assert_eq!(tokens[0].kind, TokenKind::Abstract);
    assert_eq!(tokens[1].kind, TokenKind::Virtual);
    assert_eq!(tokens[2].kind, TokenKind::Static);
    assert_eq!(tokens[3].kind, TokenKind::Final);
    assert_eq!(tokens[4].kind, TokenKind::Transient);
    assert_eq!(tokens[5].kind, TokenKind::Override);
}

#[test]
fn test_sharing_modifiers() {
    let tokens = tokenize("with sharing");
    assert_eq!(tokens[0].kind, TokenKind::WithSharing);

    let tokens = tokenize("without sharing");
    assert_eq!(tokens[0].kind, TokenKind::WithoutSharing);

    let tokens = tokenize("inherited sharing");
    assert_eq!(tokens[0].kind, TokenKind::InheritedSharing);
}

#[test]
fn test_type_declaration_keywords() {
    let tokens = tokenize("class interface enum trigger");
    assert_eq!(tokens[0].kind, TokenKind::Class);
    assert_eq!(tokens[1].kind, TokenKind::Interface);
    assert_eq!(tokens[2].kind, TokenKind::Enum);
    assert_eq!(tokens[3].kind, TokenKind::Trigger);
}

#[test]
fn test_control_flow_keywords() {
    let tokens = tokenize("if else for while do switch when break continue return throw try catch finally");
    assert_eq!(tokens[0].kind, TokenKind::If);
    assert_eq!(tokens[1].kind, TokenKind::Else);
    assert_eq!(tokens[2].kind, TokenKind::For);
    assert_eq!(tokens[3].kind, TokenKind::While);
    assert_eq!(tokens[4].kind, TokenKind::Do);
    assert_eq!(tokens[5].kind, TokenKind::Switch);
    assert_eq!(tokens[6].kind, TokenKind::When);
    assert_eq!(tokens[7].kind, TokenKind::Break);
    assert_eq!(tokens[8].kind, TokenKind::Continue);
    assert_eq!(tokens[9].kind, TokenKind::Return);
    assert_eq!(tokens[10].kind, TokenKind::Throw);
    assert_eq!(tokens[11].kind, TokenKind::Try);
    assert_eq!(tokens[12].kind, TokenKind::Catch);
    assert_eq!(tokens[13].kind, TokenKind::Finally);
}

#[test]
fn test_oop_keywords() {
    let tokens = tokenize("extends implements this super new instanceof");
    assert_eq!(tokens[0].kind, TokenKind::Extends);
    assert_eq!(tokens[1].kind, TokenKind::Implements);
    assert_eq!(tokens[2].kind, TokenKind::This);
    assert_eq!(tokens[3].kind, TokenKind::Super);
    assert_eq!(tokens[4].kind, TokenKind::New);
    assert_eq!(tokens[5].kind, TokenKind::Instanceof);
}

#[test]
fn test_primitive_type_keywords() {
    let tokens = tokenize("void boolean integer long double decimal string blob date datetime time id object");
    assert_eq!(tokens[0].kind, TokenKind::Void);
    assert_eq!(tokens[1].kind, TokenKind::Boolean);
    assert_eq!(tokens[2].kind, TokenKind::Integer);
    assert_eq!(tokens[3].kind, TokenKind::Long);
    assert_eq!(tokens[4].kind, TokenKind::Double);
    assert_eq!(tokens[5].kind, TokenKind::Decimal);
    assert_eq!(tokens[6].kind, TokenKind::StringType);
    assert_eq!(tokens[7].kind, TokenKind::Blob);
    assert_eq!(tokens[8].kind, TokenKind::Date);
    assert_eq!(tokens[9].kind, TokenKind::Datetime);
    assert_eq!(tokens[10].kind, TokenKind::Time);
    assert_eq!(tokens[11].kind, TokenKind::Id);
    assert_eq!(tokens[12].kind, TokenKind::Object);
}

#[test]
fn test_collection_keywords() {
    let tokens = tokenize("list set map");
    assert_eq!(tokens[0].kind, TokenKind::List);
    assert_eq!(tokens[1].kind, TokenKind::Set);
    assert_eq!(tokens[2].kind, TokenKind::Map);
}

#[test]
fn test_dml_keywords() {
    let tokens = tokenize("insert update upsert delete undelete merge");
    assert_eq!(tokens[0].kind, TokenKind::Insert);
    assert_eq!(tokens[1].kind, TokenKind::Update);
    assert_eq!(tokens[2].kind, TokenKind::Upsert);
    assert_eq!(tokens[3].kind, TokenKind::Delete);
    assert_eq!(tokens[4].kind, TokenKind::Undelete);
    assert_eq!(tokens[5].kind, TokenKind::Merge);
}

#[test]
fn test_soql_keywords() {
    let tokens = tokenize("select from where order by limit offset asc desc nulls first last group having");
    assert_eq!(tokens[0].kind, TokenKind::Select);
    assert_eq!(tokens[1].kind, TokenKind::From);
    assert_eq!(tokens[2].kind, TokenKind::Where);
    assert_eq!(tokens[3].kind, TokenKind::Order);
    assert_eq!(tokens[4].kind, TokenKind::By);
    assert_eq!(tokens[5].kind, TokenKind::Limit);
    assert_eq!(tokens[6].kind, TokenKind::Offset);
    assert_eq!(tokens[7].kind, TokenKind::Asc);
    assert_eq!(tokens[8].kind, TokenKind::Desc);
    assert_eq!(tokens[9].kind, TokenKind::Nulls);
    assert_eq!(tokens[10].kind, TokenKind::First);
    assert_eq!(tokens[11].kind, TokenKind::Last);
    assert_eq!(tokens[12].kind, TokenKind::Group);
    assert_eq!(tokens[13].kind, TokenKind::Having);
}

#[test]
fn test_logical_keywords() {
    let tokens = tokenize("and or not in like includes excludes");
    assert_eq!(tokens[0].kind, TokenKind::And);
    assert_eq!(tokens[1].kind, TokenKind::Or);
    assert_eq!(tokens[2].kind, TokenKind::Not);
    assert_eq!(tokens[3].kind, TokenKind::In);
    assert_eq!(tokens[4].kind, TokenKind::Like);
    assert_eq!(tokens[5].kind, TokenKind::Includes);
    assert_eq!(tokens[6].kind, TokenKind::Excludes);
}

#[test]
fn test_trigger_keywords() {
    let tokens = tokenize("before after on");
    assert_eq!(tokens[0].kind, TokenKind::Before);
    assert_eq!(tokens[1].kind, TokenKind::After);
    assert_eq!(tokens[2].kind, TokenKind::On);
}

#[test]
fn test_literal_keywords() {
    let tokens = tokenize("null true false");
    assert_eq!(tokens[0].kind, TokenKind::Null);
    assert_eq!(tokens[1].kind, TokenKind::True);
    assert_eq!(tokens[2].kind, TokenKind::False);
}

#[test]
fn test_property_keywords() {
    let tokens = tokenize("get set");
    assert_eq!(tokens[0].kind, TokenKind::Get);
    assert_eq!(tokens[1].kind, TokenKind::Set);
}

#[test]
fn test_special_keywords() {
    let tokens = tokenize("testmethod webservice");
    assert_eq!(tokens[0].kind, TokenKind::TestMethod);
    assert_eq!(tokens[1].kind, TokenKind::WebService);
}

// ==================== Case Insensitivity Tests ====================

#[test]
fn test_case_insensitive_keywords() {
    let tokens = tokenize("PUBLIC Public public PUBLIC");
    assert!(tokens.iter().take(4).all(|t| t.kind == TokenKind::Public));
}

#[test]
fn test_mixed_case_keywords() {
    let tokens = tokenize("Class CLASS cLaSs ClAsS");
    assert!(tokens.iter().take(4).all(|t| t.kind == TokenKind::Class));
}

#[test]
fn test_case_insensitive_types() {
    let tokens = tokenize("Integer INTEGER integer InTeGeR");
    assert!(tokens.iter().take(4).all(|t| t.kind == TokenKind::Integer));
}

// ==================== Operator Tests ====================

#[test]
fn test_arithmetic_operators() {
    let tokens = tokenize("+ - * /");
    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Slash);
}

#[test]
fn test_comparison_operators() {
    let tokens = tokenize("== != === !== < > <= >=");
    assert_eq!(tokens[0].kind, TokenKind::EqEq);
    assert_eq!(tokens[1].kind, TokenKind::NotEq);
    assert_eq!(tokens[2].kind, TokenKind::EqEqEq);
    assert_eq!(tokens[3].kind, TokenKind::NotEqEq);
    assert_eq!(tokens[4].kind, TokenKind::Lt);
    assert_eq!(tokens[5].kind, TokenKind::Gt);
    assert_eq!(tokens[6].kind, TokenKind::LtEq);
    assert_eq!(tokens[7].kind, TokenKind::GtEq);
}

#[test]
fn test_logical_operators() {
    let tokens = tokenize("&& || !");
    assert_eq!(tokens[0].kind, TokenKind::AndAnd);
    assert_eq!(tokens[1].kind, TokenKind::OrOr);
    assert_eq!(tokens[2].kind, TokenKind::Bang);
}

#[test]
fn test_bitwise_operators() {
    let tokens = tokenize("& | ^ ~ << >> >>>");
    assert_eq!(tokens[0].kind, TokenKind::Amp);
    assert_eq!(tokens[1].kind, TokenKind::Pipe);
    assert_eq!(tokens[2].kind, TokenKind::Caret);
    assert_eq!(tokens[3].kind, TokenKind::Tilde);
    assert_eq!(tokens[4].kind, TokenKind::LtLt);
    assert_eq!(tokens[5].kind, TokenKind::GtGt);
    assert_eq!(tokens[6].kind, TokenKind::GtGtGt);
}

#[test]
fn test_assignment_operators() {
    let tokens = tokenize("= += -= *= /= &= |= ^= <<= >>= >>>=");
    assert_eq!(tokens[0].kind, TokenKind::Eq);
    assert_eq!(tokens[1].kind, TokenKind::PlusEq);
    assert_eq!(tokens[2].kind, TokenKind::MinusEq);
    assert_eq!(tokens[3].kind, TokenKind::StarEq);
    assert_eq!(tokens[4].kind, TokenKind::SlashEq);
    assert_eq!(tokens[5].kind, TokenKind::AmpEq);
    assert_eq!(tokens[6].kind, TokenKind::PipeEq);
    assert_eq!(tokens[7].kind, TokenKind::CaretEq);
    assert_eq!(tokens[8].kind, TokenKind::LtLtEq);
    assert_eq!(tokens[9].kind, TokenKind::GtGtEq);
    assert_eq!(tokens[10].kind, TokenKind::GtGtGtEq);
}

#[test]
fn test_increment_decrement_operators() {
    let tokens = tokenize("++ --");
    assert_eq!(tokens[0].kind, TokenKind::PlusPlus);
    assert_eq!(tokens[1].kind, TokenKind::MinusMinus);
}

#[test]
fn test_special_operators() {
    let tokens = tokenize("? ?. ?? =>");
    assert_eq!(tokens[0].kind, TokenKind::Question);
    assert_eq!(tokens[1].kind, TokenKind::QuestionDot);
    assert_eq!(tokens[2].kind, TokenKind::QuestionQuestion);
    assert_eq!(tokens[3].kind, TokenKind::Arrow);
}

// ==================== Delimiter Tests ====================

#[test]
fn test_delimiters() {
    let tokens = tokenize("( ) { } [ ] ; , . : @");
    assert_eq!(tokens[0].kind, TokenKind::LParen);
    assert_eq!(tokens[1].kind, TokenKind::RParen);
    assert_eq!(tokens[2].kind, TokenKind::LBrace);
    assert_eq!(tokens[3].kind, TokenKind::RBrace);
    assert_eq!(tokens[4].kind, TokenKind::LBracket);
    assert_eq!(tokens[5].kind, TokenKind::RBracket);
    assert_eq!(tokens[6].kind, TokenKind::Semicolon);
    assert_eq!(tokens[7].kind, TokenKind::Comma);
    assert_eq!(tokens[8].kind, TokenKind::Dot);
    assert_eq!(tokens[9].kind, TokenKind::Colon);
    assert_eq!(tokens[10].kind, TokenKind::At);
}

// ==================== Literal Tests ====================

#[test]
fn test_integer_literals() {
    let tokens = tokenize("0 1 42 100 999999");
    assert!(matches!(tokens[0].kind, TokenKind::IntegerLiteral(0)));
    assert!(matches!(tokens[1].kind, TokenKind::IntegerLiteral(1)));
    assert!(matches!(tokens[2].kind, TokenKind::IntegerLiteral(42)));
    assert!(matches!(tokens[3].kind, TokenKind::IntegerLiteral(100)));
    assert!(matches!(tokens[4].kind, TokenKind::IntegerLiteral(999999)));
}

#[test]
fn test_long_literals() {
    let tokens = tokenize("0L 1l 42L 100l 999999L");
    assert!(matches!(tokens[0].kind, TokenKind::LongLiteral(0)));
    assert!(matches!(tokens[1].kind, TokenKind::LongLiteral(1)));
    assert!(matches!(tokens[2].kind, TokenKind::LongLiteral(42)));
    assert!(matches!(tokens[3].kind, TokenKind::LongLiteral(100)));
    assert!(matches!(tokens[4].kind, TokenKind::LongLiteral(999999)));
}

#[test]
fn test_double_literals() {
    let tokens = tokenize("0.0 1.5 3.14159 100.001");
    assert!(matches!(tokens[0].kind, TokenKind::DoubleLiteral(n) if (n - 0.0).abs() < 0.0001));
    assert!(matches!(tokens[1].kind, TokenKind::DoubleLiteral(n) if (n - 1.5).abs() < 0.0001));
    assert!(matches!(tokens[2].kind, TokenKind::DoubleLiteral(n) if (n - 3.14159).abs() < 0.0001));
    assert!(matches!(tokens[3].kind, TokenKind::DoubleLiteral(n) if (n - 100.001).abs() < 0.0001));
}

#[test]
fn test_double_literals_with_exponent() {
    let tokens = tokenize("1.0e10 2.5E-5 3.14e+2");
    assert!(matches!(tokens[0].kind, TokenKind::DoubleLiteral(_)));
    assert!(matches!(tokens[1].kind, TokenKind::DoubleLiteral(_)));
    assert!(matches!(tokens[2].kind, TokenKind::DoubleLiteral(_)));
}

#[test]
fn test_string_literals() {
    let tokens = tokenize("'hello' 'world' 'test string'");
    assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s == "hello"));
    assert!(matches!(&tokens[1].kind, TokenKind::StringLiteral(s) if s == "world"));
    assert!(matches!(&tokens[2].kind, TokenKind::StringLiteral(s) if s == "test string"));
}

#[test]
fn test_string_literal_escapes() {
    let tokens = tokenize(r"'hello\nworld' 'tab\there' 'quote\'s'");
    assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s == "hello\nworld"));
    assert!(matches!(&tokens[1].kind, TokenKind::StringLiteral(s) if s == "tab\there"));
    assert!(matches!(&tokens[2].kind, TokenKind::StringLiteral(s) if s == "quote's"));
}

#[test]
fn test_empty_string() {
    let tokens = tokenize("''");
    assert!(matches!(&tokens[0].kind, TokenKind::StringLiteral(s) if s.is_empty()));
}

// ==================== Identifier Tests ====================

#[test]
fn test_simple_identifiers() {
    let tokens = tokenize("foo bar baz myVar");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "bar"));
    assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "baz"));
    assert!(matches!(&tokens[3].kind, TokenKind::Identifier(s) if s == "myVar"));
}

#[test]
fn test_identifiers_with_numbers() {
    let tokens = tokenize("var1 test2 abc123");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "var1"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "test2"));
    assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "abc123"));
}

#[test]
fn test_identifiers_with_underscores() {
    let tokens = tokenize("_private my_var __double CONSTANT_NAME");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "_private"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "my_var"));
    assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "__double"));
    assert!(matches!(&tokens[3].kind, TokenKind::Identifier(s) if s == "CONSTANT_NAME"));
}

#[test]
fn test_identifier_not_keyword() {
    // These should be identifiers, not keywords
    let tokens = tokenize("classes interfaces publics");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "classes"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "interfaces"));
    assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "publics"));
}

// ==================== Annotation Tests ====================

#[test]
fn test_simple_annotations() {
    let tokens = tokenize("@isTest @AuraEnabled @Future");
    assert!(matches!(&tokens[0].kind, TokenKind::Annotation(s) if s == "isTest"));
    assert!(matches!(&tokens[1].kind, TokenKind::Annotation(s) if s == "AuraEnabled"));
    assert!(matches!(&tokens[2].kind, TokenKind::Annotation(s) if s == "Future"));
}

#[test]
fn test_annotation_with_parentheses() {
    let tokens = tokenize("@isTest(SeeAllData=true)");
    assert!(matches!(&tokens[0].kind, TokenKind::Annotation(s) if s == "isTest"));
    assert_eq!(tokens[1].kind, TokenKind::LParen);
}

#[test]
fn test_common_apex_annotations() {
    let tokens = tokenize("@Deprecated @ReadOnly @RemoteAction @TestVisible @InvocableMethod @InvocableVariable");
    assert!(matches!(&tokens[0].kind, TokenKind::Annotation(s) if s == "Deprecated"));
    assert!(matches!(&tokens[1].kind, TokenKind::Annotation(s) if s == "ReadOnly"));
    assert!(matches!(&tokens[2].kind, TokenKind::Annotation(s) if s == "RemoteAction"));
    assert!(matches!(&tokens[3].kind, TokenKind::Annotation(s) if s == "TestVisible"));
    assert!(matches!(&tokens[4].kind, TokenKind::Annotation(s) if s == "InvocableMethod"));
    assert!(matches!(&tokens[5].kind, TokenKind::Annotation(s) if s == "InvocableVariable"));
}

// ==================== Comment Tests ====================

#[test]
fn test_single_line_comment() {
    let tokens = tokenize("foo // this is a comment\nbar");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "bar"));
}

#[test]
fn test_multiline_comment() {
    let tokens = tokenize("foo /* this is\na multiline\ncomment */ bar");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "bar"));
}

#[test]
fn test_nested_style_comment() {
    // Apex doesn't support nested comments, the first */ ends the comment
    let tokens = tokenize("foo /* outer /* inner */ bar");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "bar"));
}

#[test]
fn test_comment_at_end_of_file() {
    let tokens = tokenize("foo // comment at end");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

// ==================== Whitespace Tests ====================

#[test]
fn test_various_whitespace() {
    let tokens = tokenize("foo\tbar\n\nbaz\r\nqux");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert!(matches!(&tokens[1].kind, TokenKind::Identifier(s) if s == "bar"));
    assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "baz"));
    assert!(matches!(&tokens[3].kind, TokenKind::Identifier(s) if s == "qux"));
}

#[test]
fn test_no_whitespace_between_tokens() {
    let tokens = tokenize("foo+bar*baz");
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "foo"));
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert!(matches!(&tokens[2].kind, TokenKind::Identifier(s) if s == "bar"));
    assert_eq!(tokens[3].kind, TokenKind::Star);
    assert!(matches!(&tokens[4].kind, TokenKind::Identifier(s) if s == "baz"));
}

// ==================== Span Tests ====================

#[test]
fn test_token_spans() {
    let tokens = tokenize("public class Test");
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 6); // "public"
    assert_eq!(tokens[1].span.start, 7);
    assert_eq!(tokens[1].span.end, 12); // "class"
    assert_eq!(tokens[2].span.start, 13);
    assert_eq!(tokens[2].span.end, 17); // "Test"
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_input() {
    let tokens = tokenize("");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_only_whitespace() {
    let tokens = tokenize("   \t\n\r\n   ");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_only_comments() {
    let tokens = tokenize("// just a comment");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_consecutive_operators() {
    let tokens = tokenize("a+++b");
    // Should be: a ++ + b
    assert!(matches!(&tokens[0].kind, TokenKind::Identifier(s) if s == "a"));
    assert_eq!(tokens[1].kind, TokenKind::PlusPlus);
    assert_eq!(tokens[2].kind, TokenKind::Plus);
    assert!(matches!(&tokens[3].kind, TokenKind::Identifier(s) if s == "b"));
}

#[test]
fn test_generic_type_tokens() {
    let tokens = tokenize("List<String>");
    assert_eq!(tokens[0].kind, TokenKind::List);
    assert_eq!(tokens[1].kind, TokenKind::Lt);
    assert_eq!(tokens[2].kind, TokenKind::StringType);
    assert_eq!(tokens[3].kind, TokenKind::Gt);
}

#[test]
fn test_nested_generic_tokens() {
    let tokens = tokenize("Map<String, List<Integer>>");
    assert_eq!(tokens[0].kind, TokenKind::Map);
    assert_eq!(tokens[1].kind, TokenKind::Lt);
    assert_eq!(tokens[2].kind, TokenKind::StringType);
    assert_eq!(tokens[3].kind, TokenKind::Comma);
    assert_eq!(tokens[4].kind, TokenKind::List);
    assert_eq!(tokens[5].kind, TokenKind::Lt);
    assert_eq!(tokens[6].kind, TokenKind::Integer);
    assert_eq!(tokens[7].kind, TokenKind::GtGt); // This is tokenized as >> not > >
}
