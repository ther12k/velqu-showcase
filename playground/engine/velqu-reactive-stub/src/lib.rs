//! Inert stand-in for `velqu-reactive` (see Cargo.toml header). Every
//! method is a no-op that preserves the real signatures velqu-view
//! compiles against.

use std::fmt;
use std::time::Duration;

// -- limits ---------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct JsLimits {
    pub max_heap_bytes: usize,
    pub max_stack_bytes: usize,
    pub max_execution_time: Duration,
    pub max_source_bytes: usize,
    pub max_event_payload_bytes: usize,
    pub max_mutations_per_turn: usize,
    pub max_output_string_bytes: usize,
    pub max_pending_jobs: usize,
}

impl Default for JsLimits {
    fn default() -> Self {
        Self {
            max_heap_bytes: 16 * 1024 * 1024,
            max_stack_bytes: 1024 * 1024,
            max_execution_time: Duration::from_millis(16),
            max_source_bytes: 256 * 1024,
            max_event_payload_bytes: 64 * 1024,
            max_mutations_per_turn: 1024,
            max_output_string_bytes: 64 * 1024,
            max_pending_jobs: 256,
        }
    }
}

#[derive(Debug)]
pub struct JsFailure;

impl fmt::Display for JsFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reactive engine unavailable in this build")
    }
}

// -- state ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ReactiveValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ReactiveValue>),
    Object(Vec<(String, ReactiveValue)>),
}

impl ReactiveValue {
    pub fn truthy(&self) -> bool {
        match self {
            ReactiveValue::Null => false,
            ReactiveValue::Bool(v) => *v,
            ReactiveValue::Number(v) => *v != 0.0 && !v.is_nan(),
            ReactiveValue::String(v) => !v.is_empty(),
            ReactiveValue::Array(_) | ReactiveValue::Object(_) => true,
        }
    }
}

// -- plan -----------------------------------------------------------------

pub trait ReactiveDom {
    type Node: Copy + Eq + fmt::Debug;
    fn root(&self) -> Self::Node;
    fn children(&self, node: Self::Node) -> Vec<Self::Node>;
    fn tag(&self, node: Self::Node) -> &str;
    fn attributes(&self, node: Self::Node) -> Vec<(String, String)>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceSpan {
    pub ordinal: usize,
    pub attribute: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Text,
    Show,
    Class,
    Style,
    Value,
    Disabled,
    Checked,
    Model,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopePlan<N> {
    pub node: N,
    pub parent: Option<usize>,
    pub initializer_source: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding<N> {
    pub node: N,
    pub scope: usize,
    pub kind: BindingKind,
    pub expression_source: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHandler {
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventBinding<N> {
    pub node: N,
    pub scope: usize,
    pub handler: EventHandler,
    pub handler_source: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactiveDiagnostic {
    pub span: SourceSpan,
    pub message: String,
}

impl fmt::Display for ReactiveDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node {} {}: {}", self.span.ordinal, self.span.attribute, self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReactiveDocument<N> {
    pub scopes: Vec<ScopePlan<N>>,
    pub bindings: Vec<Binding<N>>,
    pub events: Vec<EventBinding<N>>,
    pub diagnostics: Vec<ReactiveDiagnostic>,
}

impl<N> ReactiveDocument<N> {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.events.is_empty()
    }
}

/// Inert: an empty plan, exactly what the real compiler produces for
/// markup without reactive directives.
pub fn compile<N: Copy + Eq>(dom: &impl ReactiveDom<Node = N>) -> ReactiveDocument<N> {
    let _ = dom.root();
    ReactiveDocument {
        scopes: Vec::new(),
        bindings: Vec::new(),
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

// -- turn machine ---------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum PayloadValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadTooLarge {
    pub size: usize,
    pub max: usize,
}

pub struct EventPayload {
    #[allow(dead_code)]
    entries: Vec<(String, PayloadValue)>,
}

impl EventPayload {
    pub fn new(entries: Vec<(String, PayloadValue)>, max_bytes: usize) -> Result<Self, PayloadTooLarge> {
        let size: usize = entries
            .iter()
            .map(|(key, value)| match value {
                PayloadValue::Str(text) => key.len() + text.len(),
                _ => key.len(),
            })
            .sum();
        if size > max_bytes {
            return Err(PayloadTooLarge { size, max: max_bytes });
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mutation {
    pub binding: usize,
    pub kind: MutationKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationKind {
    SetText(String),
    SetVisible(bool),
    SetClass(String),
    SetStyle(String),
    SetControlValue(String),
    SetControlDisabled(bool),
    SetControlChecked(bool),
}

pub struct PendingTurn {
    pub mutations: Vec<Mutation>,
}

pub enum TurnOutcome {
    Prepared(PendingTurn),
    RolledBack(Vec<String>),
    NoChange,
}

pub struct ReactiveMachine {
    generation: u64,
    diagnostics: Vec<String>,
    state: ReactiveValue,
}

impl ReactiveMachine {
    pub fn new<N>(
        generation: u64,
        _limits: JsLimits,
        _plan: &ReactiveDocument<N>,
    ) -> Result<(Self, Vec<Mutation>), JsFailure> {
        Ok((
            Self { generation, diagnostics: Vec::new(), state: ReactiveValue::Null },
            Vec::new(),
        ))
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn state(&self) -> &ReactiveValue {
        &self.state
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn initializer_failures(&self) -> u32 {
        0
    }

    pub fn poisoned_units(&self) -> usize {
        0
    }

    pub fn record_host_diagnostic(&mut self, message: &str) {
        self.diagnostics.push(message.to_owned());
    }

    pub fn prepare(
        &mut self,
        _payload: Option<&EventPayload>,
        _handlers: &[usize],
        _model_write: Option<(&str, &str)>,
    ) -> TurnOutcome {
        TurnOutcome::NoChange
    }

    pub fn commit(&mut self, _pending: PendingTurn) {}
}
