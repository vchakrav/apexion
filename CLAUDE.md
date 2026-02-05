# ApexRust - Apex Parser, SOQL to SQL Converter & TypeScript Transpiler

## Project Overview

ApexRust is a Rust-based parser for Salesforce Apex code with:
- **SOQL to SQL conversion** - Convert SOQL queries to SQLite/PostgreSQL
- **Apex to TypeScript transpilation** - Run Apex code in JavaScript environments
- **WebAssembly support** - Full browser execution via WASM

## Architecture

```
src/
├── lib.rs              # Main library exports
├── lexer.rs            # Tokenizer (uses logos)
├── parser.rs           # Recursive descent parser for Apex + SOQL
├── ast.rs              # AST types for Apex language
├── wasm.rs             # WebAssembly bindings (wasm-bindgen)
├── sql/
│   ├── mod.rs          # SQL module exports
│   ├── schema.rs       # SalesforceSchema, SObjectDescribe, FieldDescribe
│   ├── dialect.rs      # SqlDialect trait (PostgreSQL, SQLite)
│   ├── converter.rs    # SoqlToSqlConverter - main SOQL->SQL logic
│   ├── ddl.rs          # DDL generation (CREATE TABLE)
│   ├── date_literals.rs # SOQL date literals (TODAY, LAST_N_DAYS, etc.)
│   ├── error.rs        # ConversionError, ConversionWarning
│   └── standard_objects.rs # Sales Cloud schema (21 objects)
└── transpile/
    ├── mod.rs          # Transpiler module exports
    ├── codegen.rs      # Main Apex→TypeScript code generator
    ├── context.rs      # Runtime interface definitions
    └── error.rs        # Transpilation errors

runtime/
├── index.ts            # Main runtime exports
├── apex-runtime.ts     # ApexRuntime class (SOQL/DML execution)
└── apex-stdlib.ts      # Apex standard library shims (String, List, Map, etc.)
```

## Key Components

### Parser (`src/parser.rs`)
- Recursive descent parser for full Apex language
- SOQL queries parsed inline (Expression::Soql)
- Handles classes, interfaces, triggers, enums
- ~3000 lines

### SOQL to SQL Converter (`src/sql/converter.rs`)
- Converts parsed SOQL AST to SQL string
- Dialect-aware (PostgreSQL vs SQLite differences)
- Handles:
  - Basic SELECT/FROM/WHERE/ORDER BY/LIMIT/OFFSET
  - Parent relationships via JOINs (`Account.Name` from Contact)
  - Child subqueries with JSON aggregation
  - Bind variables (`:var` → `$1` or `?1`)
  - Aggregate functions (COUNT, SUM, AVG, MIN, MAX)
  - GROUP BY / HAVING
  - Date literals (TODAY, LAST_N_DAYS, THIS_MONTH, etc.)

### Schema Model (`src/sql/schema.rs`)
- `SalesforceSchema` - collection of SObjects
- `SObjectDescribe` - table definition with fields and relationships
- `FieldDescribe` - column with type, references, relationship info
- `ChildRelationship` - for subquery support

### Transpiler (`src/transpile/`)
- Converts Apex AST to TypeScript/JavaScript
- SOQL queries become async `$runtime.query()` calls
- DML statements become async `$runtime.insert/update/delete()` calls
- Supports TypeScript type annotations or plain JavaScript
- Handles classes, methods, properties, constructors
- Control flow: if/else, for, while, switch, try/catch

### Runtime (`runtime/`)
- `ApexRuntime` - Main runtime class injected as `$runtime`
- `DatabaseAdapter` - Interface for custom database backends
- `SQLiteDatabaseAdapter` - Ready-to-use SQLite adapter
- `apex-stdlib.ts` - Apex standard library shims:
  - `System.debug()`, `System.assert()`
  - `ApexString` - String methods (contains, split, replace, etc.)
  - `ApexList<T>` - List with add, get, remove, size
  - `ApexSet<T>`, `ApexMap<K,V>` - Collection classes
  - `ApexDate`, `ApexDateTime` - Date handling
  - `ApexMath`, `ApexJSON` - Utilities

### WASM Bindings (`src/wasm.rs`)
- `parseApex(source)` - Parse Apex code, extract SOQL
- `parseSoql(source)` - Parse single SOQL query
- `convertSoqlToSql(soql, schema, dialect)` - Convert SOQL to SQL
- `generateDdl(schema, dialect)` - Generate CREATE TABLE statements
- `transpileApex(source, options)` - Transpile Apex to TypeScript
- `getRuntimeInterface()` - Get TypeScript interface for runtime
- `WasmSchema` class - Build schema from JSON or use standard Sales Cloud

## Building

### Native (for tests)
```bash
cargo test
```

### WebAssembly
```bash
# Install wasm-pack if needed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM package
wasm-pack build --target web --features wasm

# Output in pkg/
```

## Running the Demo

```bash
# Start local server
python3 -m http.server 8080

# Open browser to http://localhost:8080/demo.html
```

The demo includes:
1. Parse Apex code and extract SOQL queries
2. Convert SOQL to SQL (SQLite or PostgreSQL dialect)
3. Generate DDL for Sales Cloud schema
4. Transpile Apex to TypeScript
5. **Execute transpiled Apex against in-browser SQLite** (full end-to-end!)
6. Run raw SQL/SOQL in browser using sql.js (SQLite WASM)

## Test Files

- `tests/sql_conversion_tests.rs` - Unit tests for SQL conversion
- `tests/sqlite_e2e_tests.rs` - End-to-end tests with actual SQLite
- `tests/standard_objects_soql_tests.rs` - 52 comprehensive SOQL tests
- `tests/new_features_tests.rs` - Parser feature tests

## Important Implementation Details

### SQL Dialect Differences

| Feature | PostgreSQL | SQLite |
|---------|------------|--------|
| Boolean | `BOOLEAN` | `INTEGER` (0/1) |
| DateTime | `TIMESTAMP` | `TEXT` |
| Parameters | `$1, $2` | `?1, ?2` |
| Date functions | `CURRENT_DATE` | `date('now')` |
| JSON aggregation | `json_agg()` | `json_group_array()` |
| Foreign keys | Generated | Omitted |

### Parent Relationship Resolution

SOQL `Account.Name` from Contact becomes:
```sql
SELECT contact.id, account.name AS "Account.Name"
FROM contact
LEFT JOIN account ON contact.account_id = account.id
```

### Child Subqueries

SOQL `(SELECT Id FROM Contacts)` becomes JSON aggregation:
```sql
(SELECT json_group_array(json_object('Id', c.id))
 FROM contact c WHERE c.account_id = account.id) AS "Contacts"
```

### Serialization Fix (WASM)

`serde_wasm_bindgen` returns JS `Map` objects by default. Use `serialize_maps_as_objects(true)` for plain objects:
```rust
fn to_js_value<T: Serialize>(value: &T) -> JsValue {
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_maps_as_objects(true);
    value.serialize(&serializer).unwrap_or(JsValue::NULL)
}
```

## Current Status

- [x] Apex parser (complete)
- [x] SOQL parser (complete)
- [x] SOQL to SQL converter (complete)
- [x] DDL generation (complete)
- [x] WASM bindings (complete)
- [x] Browser demo with sql.js (complete)
- [x] Apex to TypeScript transpiler (complete)
- [x] JavaScript runtime for transpiled code (complete)
- [x] Apex standard library shims (complete)
- [ ] SOSL support (partial)

## Transpiler Example

Apex input:
```apex
public class AccountService {
    public List<Account> getActiveAccounts() {
        return [SELECT Id, Name FROM Account WHERE IsDeleted = false];
    }
    
    public void createAccount(String name) {
        Account acc = new Account();
        acc.Name = name;
        insert acc;
    }
}
```

TypeScript output:
```typescript
import { ApexRuntime } from '@apexrust/runtime';
declare const $runtime: ApexRuntime;

export class AccountService {
    public async getActiveAccounts(): Promise<Account[]> {
        return await $runtime.query<Account>(
            "SELECT Id, Name FROM Account WHERE IsDeleted = false"
        );
    }
    
    public async createAccount(name: string): Promise<void> {
        const acc: Account = {} as Account;
        acc.Name = name;
        await $runtime.insert(acc);
    }
}
```

## End-to-End Execution Flow

The demo supports full end-to-end execution of Apex code:

```
Apex Source Code
       ↓
   [Parser] ──────────────────────────────────────┐
       ↓                                          │
   Apex AST                                       │
       ↓                                          │
   [Transpiler] ──→ TypeScript Code               │
       ↓                                          │
   [TS Compiler] ──→ JavaScript                   │
       ↓                                          │
   [Execute with $runtime]                        │
       ↓                                          │
   $runtime.query(SOQL)                           │
       ↓                                          │
   [SOQL→SQL Converter] ←─ uses schema from ──────┘
       ↓
   SQL Query
       ↓
   [SQLite (sql.js)]
       ↓
   Query Results → Returned to Apex code
```

## Production Roadmap

See `PRODUCTION_ROADMAP.md` for full details. Summary:

### Key Gaps for Production

| Area | Effort | Priority |
|------|--------|----------|
| **Transpiler completeness** (inheritance, interfaces, statics) | 1-2 weeks | High |
| **Runtime completeness** (full stdlib, SObject types, limits) | 1-2 weeks | High |
| **Schema import** from Salesforce org | 3-5 days | High |
| **Execution security** (Web Worker/VM sandboxing) | 3-5 days | High |
| **Error handling** (source maps, structured errors) | 2-3 days | Medium |
| **NPM packaging** | 2-3 days | Medium |

**Total estimate: 4-7 weeks for production MVP**

### Immediate Priorities
1. Fix inheritance/interfaces in transpiler
2. Web Worker execution (security)
3. Source maps for debugging
4. Schema import from Salesforce

### Use Cases This Enables
- Apex unit testing outside Salesforce (faster feedback)
- Apex to TypeScript migration
- Offline Salesforce mobile apps
- Interactive Apex playground/tutorials
- CI/CD validation before deployment

## Useful Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check WASM compilation
cargo check --features wasm

# Build WASM
wasm-pack build --target web --features wasm

# Stop demo server
pkill -f "python3 -m http.server 8080"
```
