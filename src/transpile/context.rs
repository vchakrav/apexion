//! Runtime context for transpiled Apex code
//!
//! This defines the interface that the transpiled code expects.
//! The actual implementation is provided by the JavaScript runtime.

/// Runtime context interface
///
/// This is a marker struct - the actual runtime is JavaScript.
/// This documents what the transpiled code expects.
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    _private: (),
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// The TypeScript interface that the runtime must implement:
///
/// ```typescript
/// interface ApexRuntime {
///   // Database operations
///   query<T>(soql: string, binds?: Record<string, any>): Promise<T[]>;
///   insert(sobject: string, records: Record<string, any>[]): Promise<string[]>;
///   update(sobject: string, records: Record<string, any>[]): Promise<void>;
///   upsert(sobject: string, records: Record<string, any>[], externalIdField?: string): Promise<void>;
///   delete(sobject: string, ids: string[]): Promise<void>;
///
///   // System operations
///   debug(message: string): void;
///   now(): Date;
///   today(): Date;
///
///   // User context
///   getUserId(): string;
///   getUserName(): string;
/// }
/// ```
pub const RUNTIME_INTERFACE: &str = r#"
export interface ApexRuntime {
  // Database operations
  query<T = Record<string, any>>(soql: string, binds?: Record<string, any>): Promise<T[]>;
  insert(sobject: string, records: Record<string, any>[]): Promise<string[]>;
  update(sobject: string, records: Record<string, any>[]): Promise<void>;
  upsert(sobject: string, records: Record<string, any>[], externalIdField?: string): Promise<void>;
  delete(sobject: string, ids: string[]): Promise<void>;

  // System operations
  debug(message: string): void;
  now(): Date;
  today(): Date;

  // User context
  getUserId(): string;
  getUserName(): string;
}

// Global runtime instance injected at execution time
declare const $runtime: ApexRuntime;
"#;
