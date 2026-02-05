use crate::lexer::Span;

/// A compilation unit - the top-level AST node representing a single Apex file
#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    pub declarations: Vec<TypeDeclaration>,
}

/// A type declaration (class, interface, enum, or trigger)
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclaration {
    Class(ClassDeclaration),
    Interface(InterfaceDeclaration),
    Enum(EnumDeclaration),
    Trigger(TriggerDeclaration),
}

/// Access modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessModifier {
    #[default]
    Private,
    Public,
    Protected,
    Global,
}

/// Sharing modifiers for classes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharingModifier {
    WithSharing,
    WithoutSharing,
    InheritedSharing,
}

/// Class modifiers
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClassModifiers {
    pub access: AccessModifier,
    pub is_abstract: bool,
    pub is_virtual: bool,
    pub sharing: Option<SharingModifier>,
}

/// Method/property modifiers (also used for inner classes)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MemberModifiers {
    pub access: AccessModifier,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_transient: bool,
    pub is_testmethod: bool,
    pub is_webservice: bool,
    pub sharing: Option<SharingModifier>,
}

/// An annotation (e.g., @isTest, @AuraEnabled)
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: String,
    pub parameters: Vec<AnnotationParameter>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationParameter {
    pub name: Option<String>,
    pub value: Expression,
}

/// Class declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration {
    pub annotations: Vec<Annotation>,
    pub modifiers: ClassModifiers,
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub extends: Option<TypeRef>,
    pub implements: Vec<TypeRef>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

/// Interface declaration
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDeclaration {
    pub annotations: Vec<Annotation>,
    pub access: AccessModifier,
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub extends: Vec<TypeRef>,
    pub members: Vec<InterfaceMember>,
    pub span: Span,
}

/// Enum declaration
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclaration {
    pub annotations: Vec<Annotation>,
    pub access: AccessModifier,
    pub name: String,
    pub values: Vec<String>,
    pub span: Span,
}

/// Trigger declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerDeclaration {
    pub name: String,
    pub object: String,
    pub events: Vec<TriggerEvent>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerEvent {
    BeforeInsert,
    BeforeUpdate,
    BeforeDelete,
    AfterInsert,
    AfterUpdate,
    AfterDelete,
    AfterUndelete,
}

/// Type parameter (generics)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub name: String,
    pub span: Span,
}

/// Type reference (e.g., String, List<Account>, Map<String, Integer>)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeRef {
    pub name: String,
    pub type_arguments: Vec<TypeRef>,
    pub is_array: bool,
    pub span: Span,
}

impl TypeRef {
    pub fn simple(name: &str, span: Span) -> Self {
        Self {
            name: name.to_string(),
            type_arguments: vec![],
            is_array: false,
            span,
        }
    }
}

/// Class member (field, method, property, constructor, inner class, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Field(FieldDeclaration),
    Method(MethodDeclaration),
    Constructor(ConstructorDeclaration),
    Property(PropertyDeclaration),
    StaticBlock(Block),
    InnerClass(ClassDeclaration),
    InnerInterface(InterfaceDeclaration),
    InnerEnum(EnumDeclaration),
}

/// Interface member (method signature)
#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    Method(MethodSignature),
}

/// Field declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclaration {
    pub annotations: Vec<Annotation>,
    pub modifiers: MemberModifiers,
    pub type_ref: TypeRef,
    pub declarators: Vec<VariableDeclarator>,
    pub span: Span,
}

/// Variable declarator (name and optional initializer)
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarator {
    pub name: String,
    pub initializer: Option<Expression>,
    pub span: Span,
}

/// Method declaration
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDeclaration {
    pub annotations: Vec<Annotation>,
    pub modifiers: MemberModifiers,
    pub return_type: TypeRef,
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<Parameter>,
    pub body: Option<Block>,
    pub span: Span,
}

/// Method signature (for interfaces)
#[derive(Debug, Clone, PartialEq)]
pub struct MethodSignature {
    pub annotations: Vec<Annotation>,
    pub return_type: TypeRef,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub span: Span,
}

/// Constructor declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorDeclaration {
    pub annotations: Vec<Annotation>,
    pub modifiers: MemberModifiers,
    pub name: String,
    pub parameters: Vec<Parameter>,
    /// Constructor chaining: this(...) or super(...)
    pub chained_constructor: Option<ConstructorChain>,
    pub body: Block,
    pub span: Span,
}

/// Constructor chaining call: this(...) or super(...)
#[derive(Debug, Clone, PartialEq)]
pub struct ConstructorChain {
    pub kind: ConstructorChainKind,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructorChainKind {
    This,
    Super,
}

/// Property declaration
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDeclaration {
    pub annotations: Vec<Annotation>,
    pub modifiers: MemberModifiers,
    pub type_ref: TypeRef,
    pub name: String,
    pub getter: Option<PropertyAccessor>,
    pub setter: Option<PropertyAccessor>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyAccessor {
    pub modifiers: MemberModifiers,
    pub body: Option<Block>,
    pub span: Span,
}

/// Method/constructor parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub annotations: Vec<Annotation>,
    pub is_final: bool,
    pub type_ref: TypeRef,
    pub name: String,
    pub span: Span,
}

/// Block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// Statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Block(Block),
    LocalVariable(LocalVariableDeclaration),
    Expression(ExpressionStatement),
    If(IfStatement),
    For(ForStatement),
    ForEach(ForEachStatement),
    While(WhileStatement),
    DoWhile(DoWhileStatement),
    Switch(SwitchStatement),
    Return(ReturnStatement),
    Throw(ThrowStatement),
    Break(BreakStatement),
    Continue(ContinueStatement),
    Try(TryStatement),
    Dml(DmlStatement),
    Empty(Span),
}

/// Local variable declaration
#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableDeclaration {
    pub is_final: bool,
    pub type_ref: TypeRef,
    pub declarators: Vec<VariableDeclarator>,
    pub span: Span,
}

/// Expression statement
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub span: Span,
}

/// If statement
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
    pub span: Span,
}

/// Traditional for loop
#[derive(Debug, Clone, PartialEq)]
pub struct ForStatement {
    pub init: Option<ForInit>,
    pub condition: Option<Expression>,
    pub update: Vec<Expression>,
    pub body: Box<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Variables(LocalVariableDeclaration),
    Expressions(Vec<Expression>),
}

/// For-each loop
#[derive(Debug, Clone, PartialEq)]
pub struct ForEachStatement {
    pub type_ref: TypeRef,
    pub variable: String,
    pub iterable: Expression,
    pub body: Box<Statement>,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
    pub span: Span,
}

/// Do-while loop
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStatement {
    pub body: Box<Statement>,
    pub condition: Expression,
    pub span: Span,
}

/// Switch statement (Apex-style with when)
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStatement {
    pub expression: Expression,
    pub when_clauses: Vec<WhenClause>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhenClause {
    pub values: WhenValue,
    pub block: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WhenValue {
    Literals(Vec<Expression>),
    Type { type_ref: TypeRef, variable: String },
    Else,
}

/// Return statement
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub span: Span,
}

/// Throw statement
#[derive(Debug, Clone, PartialEq)]
pub struct ThrowStatement {
    pub exception: Expression,
    pub span: Span,
}

/// Break statement
#[derive(Debug, Clone, PartialEq)]
pub struct BreakStatement {
    pub span: Span,
}

/// Continue statement
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStatement {
    pub span: Span,
}

/// Try statement
#[derive(Debug, Clone, PartialEq)]
pub struct TryStatement {
    pub try_block: Block,
    pub catch_clauses: Vec<CatchClause>,
    pub finally_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub exception_type: TypeRef,
    pub variable: String,
    pub block: Block,
    pub span: Span,
}

/// DML statement
#[derive(Debug, Clone, PartialEq)]
pub struct DmlStatement {
    pub operation: DmlOperation,
    pub expression: Expression,
    pub access_level: Option<DmlAccessLevel>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmlOperation {
    Insert,
    Update,
    Upsert,
    Delete,
    Undelete,
    Merge,
}

/// DML access level (as system / as user)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmlAccessLevel {
    System,
    User,
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Null(Span),
    Boolean(bool, Span),
    Integer(i64, Span),
    Long(i64, Span),
    Double(f64, Span),
    String(String, Span),

    // Identifiers and access
    Identifier(String, Span),
    This(Span),
    Super(Span),

    // Member access
    FieldAccess(Box<FieldAccessExpr>),
    ArrayAccess(Box<ArrayAccessExpr>),
    SafeNavigation(Box<SafeNavigationExpr>),

    // Method calls
    MethodCall(Box<MethodCallExpr>),

    // Object creation
    New(Box<NewExpr>),
    NewArray(Box<NewArrayExpr>),
    NewMap(Box<NewMapExpr>),

    // Operators
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Ternary(Box<TernaryExpr>),
    NullCoalesce(Box<NullCoalesceExpr>),
    Instanceof(Box<InstanceofExpr>),
    Cast(Box<CastExpr>),

    // Assignment
    Assignment(Box<AssignmentExpr>),

    // Increment/Decrement
    PostIncrement(Box<Expression>, Span),
    PostDecrement(Box<Expression>, Span),
    PreIncrement(Box<Expression>, Span),
    PreDecrement(Box<Expression>, Span),

    // SOQL/SOSL
    Soql(Box<SoqlQuery>),
    Sosl(Box<SoslQuery>),

    // SOQL bind variable (:varName)
    BindVariable(String, Span),

    // Parenthesized
    Parenthesized(Box<Expression>, Span),

    // List/Set/Map literals
    ListLiteral(Vec<Expression>, Span),
    SetLiteral(Vec<Expression>, Span),
    MapLiteral(Vec<(Expression, Expression)>, Span),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Null(s) => *s,
            Expression::Boolean(_, s) => *s,
            Expression::Integer(_, s) => *s,
            Expression::Long(_, s) => *s,
            Expression::Double(_, s) => *s,
            Expression::String(_, s) => *s,
            Expression::Identifier(_, s) => *s,
            Expression::This(s) => *s,
            Expression::Super(s) => *s,
            Expression::FieldAccess(e) => e.span,
            Expression::ArrayAccess(e) => e.span,
            Expression::SafeNavigation(e) => e.span,
            Expression::MethodCall(e) => e.span,
            Expression::New(e) => e.span,
            Expression::NewArray(e) => e.span,
            Expression::NewMap(e) => e.span,
            Expression::Unary(e) => e.span,
            Expression::Binary(e) => e.span,
            Expression::Ternary(e) => e.span,
            Expression::NullCoalesce(e) => e.span,
            Expression::Instanceof(e) => e.span,
            Expression::Cast(e) => e.span,
            Expression::Assignment(e) => e.span,
            Expression::PostIncrement(_, s) => *s,
            Expression::PostDecrement(_, s) => *s,
            Expression::PreIncrement(_, s) => *s,
            Expression::PreDecrement(_, s) => *s,
            Expression::Soql(e) => e.span,
            Expression::Sosl(e) => e.span,
            Expression::BindVariable(_, s) => *s,
            Expression::Parenthesized(_, s) => *s,
            Expression::ListLiteral(_, s) => *s,
            Expression::SetLiteral(_, s) => *s,
            Expression::MapLiteral(_, s) => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldAccessExpr {
    pub object: Expression,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayAccessExpr {
    pub array: Expression,
    pub index: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafeNavigationExpr {
    pub object: Expression,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCallExpr {
    pub object: Option<Expression>,
    pub name: String,
    pub type_arguments: Vec<TypeRef>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    pub type_ref: TypeRef,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewArrayExpr {
    pub element_type: TypeRef,
    pub size: Option<Expression>,
    pub initializer: Option<Vec<Expression>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewMapExpr {
    pub type_ref: TypeRef,
    pub initializer: Option<Vec<(Expression, Expression)>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,    // -
    Not,       // !
    BitwiseNot, // ~
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Expression,
    pub operator: BinaryOp,
    pub right: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // Comparison
    Equal,
    NotEqual,
    ExactEqual,
    ExactNotEqual,
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,

    // Logical
    And,
    Or,

    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    UnsignedRightShift,

    // SOQL-specific
    Like,
    In,
    NotIn,
    Includes,
    Excludes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TernaryExpr {
    pub condition: Expression,
    pub then_expr: Expression,
    pub else_expr: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullCoalesceExpr {
    pub left: Expression,
    pub right: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstanceofExpr {
    pub expression: Expression,
    pub type_ref: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CastExpr {
    pub type_ref: TypeRef,
    pub expression: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub target: Expression,
    pub operator: AssignmentOp,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
}

/// SOQL Query
#[derive(Debug, Clone, PartialEq)]
pub struct SoqlQuery {
    pub select_clause: Vec<SelectField>,
    pub from_clause: String,
    pub where_clause: Option<Expression>,
    pub with_clause: Option<SoqlWithClause>,
    pub group_by_clause: Vec<String>,
    pub having_clause: Option<Expression>,
    pub order_by_clause: Vec<OrderByField>,
    pub limit_clause: Option<Expression>,
    pub offset_clause: Option<Expression>,
    pub for_clause: Option<ForClause>,
    pub span: Span,
}

/// SOQL WITH clause for security/sharing enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoqlWithClause {
    SecurityEnforced,
    UserMode,
    SystemMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectField {
    Field(String),
    SubQuery(Box<SoqlQuery>),
    TypeOf(TypeOfClause),
    AggregateFunction { name: String, field: String, alias: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeOfClause {
    pub field: String,
    pub when_clauses: Vec<TypeOfWhen>,
    pub else_fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeOfWhen {
    pub type_name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByField {
    pub field: String,
    pub ascending: bool,
    pub nulls_first: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForClause {
    View,
    Reference,
    Update,
}

/// SOSL Query
#[derive(Debug, Clone, PartialEq)]
pub struct SoslQuery {
    pub search_term: String,
    pub search_group: Option<SearchGroup>,
    pub returning: Vec<SoslReturning>,
    pub with_clauses: Vec<SoslWithClause>,
    pub limit_clause: Option<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchGroup {
    AllFields,
    NameFields,
    EmailFields,
    PhoneFields,
    SidebarFields,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoslReturning {
    pub object: String,
    pub fields: Vec<String>,
    pub where_clause: Option<Expression>,
    pub order_by: Vec<OrderByField>,
    pub limit_clause: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SoslWithClause {
    DataCategory(String, String), // Category group and category
    Network(String),
    Snippet,
    SpellCorrection,
}
