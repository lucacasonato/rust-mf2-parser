use std::fmt::Debug;

use crate::util::LengthShort;
use crate::util::Location;
use crate::util::Span;
use crate::util::Spanned;
use crate::visitor::Visit;
use crate::visitor::Visitable;

macro_rules! ast_enum {
  {
    #[visit($visit_method:ident)]
    pub enum $name:ident<$lifetime:lifetime> {
      $( $item:ident $(<$item_lifetime:lifetime>)? ),* $(,)?
    }
  } => {
    #[derive(Clone)]
    pub enum $name<$lifetime> {
      $( $item ( $item$(<$item_lifetime>)? ), )*
    }

    impl ::std::fmt::Debug for $name<'_> {
      fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
          $( $name::$item(item) => ::std::fmt::Debug::fmt(item, f), )*
        }
      }
    }

    impl crate::util::Spanned for $name<'_> {
      fn span(&self) -> Span {
        match self {
          $( $name::$item(item) => item.span(), )*
        }
      }
    }

    impl crate::visitor::Visitable for $name<'_> {
      fn apply_visitor<V: crate::visitor::Visit + ?Sized>(&self, visitor: &mut V) {
        visitor.$visit_method(self);
      }

      fn apply_visitor_to_children<V: crate::visitor::Visit + ?Sized>(&self, visitor: &mut V) {
        match self {
          $( $name::$item(item) => item.apply_visitor(visitor), )*
        }
      }
    }
  };
}

#[derive(Clone)]
pub enum Message<'a> {
  Simple(Pattern<'a>),
  Complex(ComplexMessage<'a>),
}

impl Debug for Message<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Message::Simple(pattern) => Debug::fmt(pattern, f),
      Message::Complex(complex) => Debug::fmt(complex, f),
    }
  }
}

impl Spanned for Message<'_> {
  fn span(&self) -> Span {
    match self {
      Message::Simple(pattern) => pattern.span(),
      Message::Complex(complex) => complex.span(),
    }
  }
}

impl Visitable for Message<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    match self {
      Message::Simple(pattern) => pattern.apply_visitor(visitor),
      Message::Complex(complex) => complex.apply_visitor(visitor),
    }
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    match self {
      Message::Simple(pattern) => pattern.apply_visitor_to_children(visitor),
      Message::Complex(complex) => complex.apply_visitor_to_children(visitor),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Pattern<'a> {
  pub parts: Vec<PatternPart<'a>>,
}

impl Spanned for Pattern<'_> {
  fn span(&self) -> Span {
    match (self.parts.first(), self.parts.last()) {
      (Some(first), Some(last)) => {
        Span::new(first.span().start..last.span().end)
      }
      _ => Span::new(Location::dummy()..Location::dummy()),
    }
  }
}

impl Visitable for Pattern<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_pattern(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for part in &self.parts {
      part.apply_visitor(visitor);
    }
  }
}

ast_enum! {
  #[visit(visit_pattern_part)]
  pub enum PatternPart<'a> {
    Text<'a>,
    Escape,
    Expression<'a>,
    Markup<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct Text<'a> {
  pub start: Location,
  pub content: &'a str,
}

impl Spanned for Text<'_> {
  fn span(&self) -> Span {
    Span::new(self.start..self.start + self.content)
  }
}

impl Visitable for Text<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_text(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, _visitor: &mut V) {}
}

#[derive(Debug, Clone)]
pub struct Escape {
  pub start: Location,
  pub escaped_char: char,
}

impl Spanned for Escape {
  fn span(&self) -> Span {
    Span::new(self.start..self.start + '\\' + self.escaped_char)
  }
}

impl Visitable for Escape {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_escape(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, _visitor: &mut V) {}
}

ast_enum! {
  #[visit(visit_expression)]
  pub enum Expression<'a> {
    LiteralExpression<'a>,
    VariableExpression<'a>,
    AnnotationExpression<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct LiteralExpression<'a> {
  pub span: Span,
  pub literal: Literal<'a>,
  pub annotation: Option<Annotation<'a>>,
  pub attributes: Vec<Attribute<'a>>,
}

impl Spanned for LiteralExpression<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for LiteralExpression<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_literal_expression(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.literal.apply_visitor(visitor);
    if let Some(annotation) = &self.annotation {
      annotation.apply_visitor(visitor);
    }
    for attribute in &self.attributes {
      attribute.apply_visitor(visitor);
    }
  }
}

#[derive(Debug, Clone)]
pub struct VariableExpression<'a> {
  pub span: Span,
  pub variable: Variable<'a>,
  pub annotation: Option<Annotation<'a>>,
  pub attributes: Vec<Attribute<'a>>,
}

impl Spanned for VariableExpression<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for VariableExpression<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_variable_expression(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.variable.apply_visitor(visitor);
    if let Some(annotation) = &self.annotation {
      annotation.apply_visitor(visitor);
    }
    for attribute in &self.attributes {
      attribute.apply_visitor(visitor);
    }
  }
}

#[derive(Debug, Clone)]
pub struct Variable<'a> {
  pub span: Span,
  pub name: &'a str,
}

impl Spanned for Variable<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for Variable<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_variable(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, _visitor: &mut V) {}
}

#[derive(Debug, Clone)]
pub struct AnnotationExpression<'a> {
  pub span: Span,
  pub annotation: Annotation<'a>,
  pub attributes: Vec<Attribute<'a>>,
}

impl Spanned for AnnotationExpression<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for AnnotationExpression<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_annotation_expression(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.annotation.apply_visitor(visitor);
    for attribute in &self.attributes {
      attribute.apply_visitor(visitor);
    }
  }
}

ast_enum! {
  #[visit(visit_annotation)]
  pub enum Annotation<'a> {
    Function<'a>,
    PrivateUseAnnotation<'a>,
    ReservedAnnotation<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct Identifier<'a> {
  pub start: Location,
  pub namespace: Option<&'a str>,
  pub name: &'a str,
}

impl Spanned for Identifier<'_> {
  fn span(&self) -> Span {
    let mut end = self.start;
    if let Some(namespace) = self.namespace {
      end = end + namespace + ':';
    }
    end = end + self.name;

    Span::new(self.start..end)
  }
}

impl Visitable for Identifier<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_identifier(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, _visitor: &mut V) {}
}

#[derive(Debug, Clone)]
pub struct Function<'a> {
  pub start: Location,
  pub id: Identifier<'a>,
  pub options: Vec<FnOrMarkupOption<'a>>,
}

impl Spanned for Function<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self
      .options
      .last()
      .map_or(self.id.span().end, |last| last.span().end);
    Span::new(start..end)
  }
}

impl Visitable for Function<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_function(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.id.apply_visitor(visitor);
    for option in &self.options {
      option.apply_visitor(visitor);
    }
  }
}

#[derive(Debug, Clone)]
pub struct FnOrMarkupOption<'a> {
  pub key: Identifier<'a>,
  pub value: LiteralOrVariable<'a>,
}

impl Spanned for FnOrMarkupOption<'_> {
  fn span(&self) -> Span {
    let start = self.key.span().start;
    let end = self.value.span().end;
    Span::new(start..end)
  }
}

impl Visitable for FnOrMarkupOption<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_fn_or_markup_option(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.key.apply_visitor(visitor);
    self.value.apply_visitor(visitor);
  }
}

#[derive(Debug, Clone)]
pub struct Attribute<'a> {
  pub span: Span,
  pub key: Identifier<'a>,
  pub value: Option<LiteralOrVariable<'a>>,
}

impl Spanned for Attribute<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for Attribute<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_attribute(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.key.apply_visitor(visitor);
    if let Some(value) = &self.value {
      value.apply_visitor(visitor);
    }
  }
}

ast_enum! {
  #[visit(visit_literal_or_variable)]
  pub enum LiteralOrVariable<'a> {
    Literal<'a>,
    Variable<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct PrivateUseAnnotation<'a> {
  pub start: Location,
  pub sigil: char,
  pub body: Vec<ReservedBodyPart<'a>>,
}

impl Spanned for PrivateUseAnnotation<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self
      .body
      .last()
      .map_or(start + self.sigil, |last| last.span().end);
    Span::new(start..end)
  }
}

impl Visitable for PrivateUseAnnotation<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_private_use_annotation(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for part in &self.body {
      part.apply_visitor(visitor);
    }
  }
}

#[derive(Debug, Clone)]
pub struct ReservedAnnotation<'a> {
  pub start: Location,
  pub sigil: char,
  pub body: Vec<ReservedBodyPart<'a>>,
}

impl Spanned for ReservedAnnotation<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self
      .body
      .last()
      .map_or(start + self.sigil, |last| last.span().end);
    Span::new(start..end)
  }
}

impl Visitable for ReservedAnnotation<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_reserved_annotation(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for part in &self.body {
      part.apply_visitor(visitor);
    }
  }
}

ast_enum! {
  #[visit(visit_reserved_body_part)]
  pub enum ReservedBodyPart<'a> {
    Text<'a>,
    Escape,
    Quoted<'a>,
  }
}

ast_enum! {
  #[visit(visit_literal)]
  pub enum Literal<'a> {
    Quoted<'a>,
    Text<'a>,
    Number<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct Quoted<'a> {
  pub span: Span,
  pub parts: Vec<QuotedPart<'a>>,
}

impl Spanned for Quoted<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for Quoted<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_quoted(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for part in &self.parts {
      part.apply_visitor(visitor);
    }
  }
}

ast_enum! {
  #[visit(visit_quoted_part)]
  pub enum QuotedPart<'a> {
    Text<'a>,
    Escape,
  }
}

#[derive(Debug, Clone, Copy)]
pub enum ExponentSign {
  Plus,
  Minus,
  None,
}

#[derive(Debug, Clone)]
pub struct Number<'a> {
  pub start: Location,
  pub raw: &'a str,
  pub is_negative: bool,
  pub integral_len: LengthShort,
  pub fractional_len: Option<LengthShort>,
  pub exponent_len: Option<(ExponentSign, LengthShort)>,
}

impl Spanned for Number<'_> {
  fn span(&self) -> Span {
    Span::new(self.start..self.start + self.raw)
  }
}

impl Visitable for Number<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_number(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, _visitor: &mut V) {}
}

impl<'a> Number<'a> {
  fn slice(&self, span: Span) -> &'a str {
    &self.raw[span.start.inner() as usize..span.end.inner() as usize]
  }

  fn integral_start(&self) -> Location {
    if self.is_negative {
      self.start + '-'
    } else {
      self.start
    }
  }

  fn integral_end(&self) -> Location {
    self.integral_start() + self.integral_len
  }

  pub fn integral_span(&self) -> Span {
    Span::new(self.integral_start()..self.integral_end())
  }

  pub fn integral_part(&self) -> &'a str {
    self.slice(self.integral_span())
  }

  pub fn fractional_span(&self) -> Option<Span> {
    self.fractional_len.map(|fractional_len| {
      let start = self.integral_end() + '.';
      let end = start + fractional_len;
      Span::new(start..end)
    })
  }

  pub fn fractional_part(&self) -> Option<&'a str> {
    self.fractional_span().map(|span| self.slice(span))
  }

  pub fn exponent_span(&self) -> Option<Span> {
    self.exponent_len.map(|(sign, exponent_len)| {
      let mut start = self.integral_end();
      if let Some(fractional_len) = &self.fractional_len {
        start = start + '.';
        start = start + *fractional_len;
      }

      start = start + 'e';

      if !matches!(sign, ExponentSign::None) {
        start = start + '-';
      };

      let end = start + exponent_len;

      Span::new(start..end)
    })
  }

  pub fn exponent_part(&self) -> Option<(ExponentSign, &'a str)> {
    self
      .exponent_span()
      .map(|span| (self.exponent_len.as_ref().unwrap().0, self.slice(span)))
  }
}

#[derive(Debug, Clone)]
pub struct Markup<'a> {
  pub span: Span,
  pub kind: MarkupKind,
  pub id: Identifier<'a>,
  pub options: Vec<FnOrMarkupOption<'a>>,
  pub attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, Clone)]
pub enum MarkupKind {
  Open,
  Standalone,
  Close,
}

impl Spanned for Markup<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for Markup<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_markup(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.id.apply_visitor(visitor);
    for option in &self.options {
      option.apply_visitor(visitor);
    }
    for attribute in &self.attributes {
      attribute.apply_visitor(visitor);
    }
  }
}

#[derive(Debug, Clone)]
pub struct ComplexMessage<'a> {
  pub declarations: Vec<Declaration<'a>>,
  pub body: ComplexMessageBody<'a>,
}

impl Spanned for ComplexMessage<'_> {
  fn span(&self) -> Span {
    let start = self
      .declarations
      .first()
      .map(|first| first.span().start.min(self.body.span().start))
      .unwrap_or_else(|| self.body.span().start);
    let end = self
      .declarations
      .last()
      .map(|last| last.span().end.max(self.body.span().end))
      .unwrap_or_else(|| self.body.span().end);
    Span::new(start..end)
  }
}

impl Visitable for ComplexMessage<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_complex_message(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for declaration in &self.declarations {
      declaration.apply_visitor(visitor);
    }
    self.body.apply_visitor(visitor);
  }
}

ast_enum! {
  #[visit(visit_declaration)]
  pub enum Declaration<'a> {
    InputDeclaration<'a>,
    LocalDeclaration<'a>,
    ReservedStatement<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct InputDeclaration<'a> {
  pub start: Location,
  pub expression: VariableExpression<'a>,
}

impl Spanned for InputDeclaration<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self.expression.span().end;
    Span::new(start..end)
  }
}

impl Visitable for InputDeclaration<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_input_declaration(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.expression.apply_visitor(visitor);
  }
}

#[derive(Debug, Clone)]
pub struct LocalDeclaration<'a> {
  pub start: Location,
  pub variable: Variable<'a>,
  pub expression: Expression<'a>,
}

impl Spanned for LocalDeclaration<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self.expression.span().end;
    Span::new(start..end)
  }
}

impl Visitable for LocalDeclaration<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_local_declaration(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.variable.apply_visitor(visitor);
    self.expression.apply_visitor(visitor);
  }
}

#[derive(Debug, Clone)]
pub struct ReservedStatement<'a> {
  pub start: Location,
  pub name: &'a str,
  pub body: Vec<ReservedBodyPart<'a>>,
  pub expressions: Vec<Expression<'a>>,
}

impl Spanned for ReservedStatement<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self
      .expressions
      .last()
      .map(|last| last.span().end)
      .unwrap_or_else(|| {
        self
          .body
          .last()
          .map(|last| last.span().end)
          .unwrap_or_else(|| start + '.' + self.name)
      });
    Span::new(start..end)
  }
}

impl Visitable for ReservedStatement<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_reserved_statement(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for part in &self.body {
      part.apply_visitor(visitor);
    }
    for expression in &self.expressions {
      expression.apply_visitor(visitor);
    }
  }
}

ast_enum! {
  #[visit(visit_complex_message_body)]
  pub enum ComplexMessageBody<'a> {
    QuotedPattern<'a>,
    Matcher<'a>,
  }
}

#[derive(Debug, Clone)]
pub struct QuotedPattern<'a> {
  pub span: Span,
  pub pattern: Pattern<'a>,
}

impl Spanned for QuotedPattern<'_> {
  fn span(&self) -> Span {
    self.span
  }
}

impl Visitable for QuotedPattern<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_quoted_pattern(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    self.pattern.apply_visitor(visitor);
  }
}

#[derive(Debug, Clone)]
pub struct Matcher<'a> {
  pub start: Location,
  pub selectors: Vec<Expression<'a>>,
  pub variants: Vec<Variant<'a>>,
}

impl Spanned for Matcher<'_> {
  fn span(&self) -> Span {
    let start = self.start;
    let end = self
      .variants
      .last()
      .map(|last| last.span().end)
      .unwrap_or_else(|| {
        self
          .selectors
          .last()
          .map(|last| last.span().end)
          .unwrap_or_else(|| start + ".match")
      });
    Span::new(start..end)
  }
}

impl Visitable for Matcher<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_matcher(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for selector in &self.selectors {
      selector.apply_visitor(visitor);
    }
    for variant in &self.variants {
      variant.apply_visitor(visitor);
    }
  }
}

#[derive(Debug, Clone)]
pub struct Variant<'a> {
  pub keys: Vec<Key<'a>>,
  pub pattern: QuotedPattern<'a>,
}

impl Spanned for Variant<'_> {
  fn span(&self) -> Span {
    let start = self
      .keys
      .first()
      .map(|first| first.span().start)
      .unwrap_or_else(|| self.pattern.span().start);
    let end = self.pattern.span().end;
    Span::new(start..end)
  }
}

impl Visitable for Variant<'_> {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_variant(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, visitor: &mut V) {
    for key in &self.keys {
      key.apply_visitor(visitor);
    }
    self.pattern.apply_visitor(visitor);
  }
}

ast_enum! {
  #[visit(visit_key)]
  pub enum Key<'a> {
    Literal<'a>,
    Star,
  }
}

#[derive(Debug, Clone)]
pub struct Star {
  pub start: Location,
}

impl Spanned for Star {
  fn span(&self) -> Span {
    Span::new(self.start..self.start + '*')
  }
}

impl Visitable for Star {
  fn apply_visitor<V: Visit + ?Sized>(&self, visitor: &mut V) {
    visitor.visit_star(self);
  }

  fn apply_visitor_to_children<V: Visit + ?Sized>(&self, _visitor: &mut V) {}
}
