# Apex Parser - Development Notes

## Project Status

**Fully implemented:**
- Apex parser (complete)
- SOQL parser (complete)
- SOQL to SQL converter (SQLite & PostgreSQL)
- DDL generation
- Apex to TypeScript transpiler
- JavaScript runtime for transpiled code
- Browser demo with in-browser SQLite execution

## Architecture

```
src/
├── lexer.rs              # Logos-based tokenizer with 2-token lookahead
├── parser.rs             # Recursive descent parser for Apex + SOQL
├── ast.rs                # AST node definitions
├── lib.rs                # Public API
├── wasm.rs               # WebAssembly bindings
├── sql/
│   ├── schema.rs         # Salesforce schema model
│   ├── dialect.rs        # SQL dialect abstraction (SQLite/Postgres)
│   ├── converter.rs      # SOQL to SQL converter
│   ├── ddl.rs            # DDL generation
│   ├── date_literals.rs  # SOQL date literal expansion
│   └── standard_objects.rs # Sales Cloud schema (21 objects)
└── transpile/
    ├── codegen.rs        # Apex to TypeScript code generator
    ├── context.rs        # Runtime interface definitions
    └── error.rs          # Transpilation errors

runtime/
├── apex-runtime.ts       # ApexRuntime class for SOQL/DML
└── apex-stdlib.ts        # Apex standard library shims
```

## Key Implementation Details

### Lexer
- Uses `logos` crate for fast tokenization
- Two-token lookahead via `peek()` and `peek_second()`

### Parser Challenges Solved

1. **Keywords as identifiers**: Apex allows many keywords as identifiers
2. **Cast vs parenthesized expression**: Heuristics in `try_parse_cast()`
3. **Qualified type names vs method calls**: Separate parse functions
4. **`instanceof` in expressions**: Handled in relational and binary parsers
5. **SOQL/SOSL**: Full inline query support with bind variables

### SOQL Support

**Fully supported:**
- SELECT with fields, relationships, subqueries, aggregates
- TYPEOF clause
- WHERE with all operators (=, !=, <, >, LIKE, IN, NOT IN, INCLUDES, EXCLUDES)
- AND, OR, NOT with parentheses
- Bind variables (`:variable`)
- ORDER BY with ASC/DESC, NULLS FIRST/LAST (including dotted paths like `Account.Name`)
- GROUP BY, HAVING
- LIMIT, OFFSET
- WITH SECURITY_ENFORCED, USER_MODE, SYSTEM_MODE
- FOR UPDATE, VIEW, REFERENCE
- Date literals (TODAY, LAST_N_DAYS:n, etc.)

### SOQL to SQL Conversion

The converter handles:
- Parent relationships via JOINs (`Account.Name` from Contact)
- Child subqueries with JSON aggregation
- Bind variables → parameterized queries
- Date literals → dialect-specific SQL
- Aggregate functions (COUNT, SUM, AVG, MIN, MAX)

**SQL Dialect Differences:**

| Feature | PostgreSQL | SQLite |
|---------|------------|--------|
| Boolean | `BOOLEAN` | `INTEGER` (0/1) |
| DateTime | `TIMESTAMP` | `TEXT` |
| Parameters | `$1, $2` | `?1, ?2` |
| Date functions | `CURRENT_DATE` | `date('now')` |
| JSON aggregation | `json_agg()` | `json_group_array()` |

### Apex to TypeScript Transpiler

The transpiler converts Apex code to TypeScript/JavaScript that can run in:
- Browser (Service Workers, main thread)
- Node.js
- Deno
- Edge functions (Cloudflare Workers, etc.)

**Key transformations:**
- SOQL queries → `await $runtime.query("...")`
- DML statements → `await $runtime.insert/update/delete(...)`
- Apex types → TypeScript types (List → array, Map → Map, etc.)
- Methods with SOQL/DML → async methods

**Syntax Mapping:**

| Apex | TypeScript |
|------|------------|
| `public class Foo` | `export class Foo` |
| `private String x` | `private x: string` |
| `List<Account>` | `Account[]` |
| `Map<String, Integer>` | `Map<string, number>` |
| `Set<Id>` | `Set<string>` |
| `for (Account a : accounts)` | `for (const a of accounts)` |
| `[SELECT ...]` | `await $runtime.query(\`...\`)` |
| `insert accounts` | `await $runtime.insert(accounts)` |

### JavaScript Runtime

The runtime (`runtime/apex-runtime.ts`) provides:
- `ApexRuntime` class with query/insert/update/delete/upsert/undelete
- `DatabaseAdapter` interface for custom backends
- `SQLiteDatabaseAdapter` for browser/local SQLite

The stdlib (`runtime/apex-stdlib.ts`) provides:
- `System.debug()`, `System.assert()`
- `ApexString` methods (isBlank, contains, split, etc.)
- `ApexList<T>`, `ApexSet<T>`, `ApexMap<K,V>`
- `ApexDate`, `ApexDateTime`
- `ApexMath`, `ApexJSON`

## Browser Demo

The demo (`demo.html`) includes:
1. Parse Apex code and extract SOQL queries
2. Convert SOQL to SQL (SQLite or PostgreSQL)
3. Generate DDL for Sales Cloud schema
4. Transpile Apex to TypeScript
5. **Execute transpiled Apex against in-browser SQLite**

The execution flow:
```
Apex Code → Transpile to JS → TypeScript Compiler → Execute with $runtime
    ↓
$runtime.query(SOQL) → convertSoqlToSql() → SQLite.exec() → Results
```

## WASM Bindings

Available functions:
- `parseApex(source)` - Parse Apex code, extract SOQL
- `parseSoql(source)` - Parse single SOQL query
- `convertSoqlToSql(soql, schema, dialect)` - Convert SOQL to SQL
- `generateDdl(schema, dialect)` - Generate CREATE TABLE statements
- `transpileApex(source, options)` - Transpile Apex to TypeScript
- `getRuntimeInterface()` - Get TypeScript interface for runtime
- `WasmSchema` class - Build schema from JSON or use standard Sales Cloud

## Commands

```bash
# Run all tests
cargo test

# Build WASM
wasm-pack build --target web --features wasm

# Run demo
python3 -m http.server 8080
# Open http://localhost:8080/demo.html
```

## Future Work

1. Full SOSL support
2. More comprehensive stdlib coverage
3. Bind variable resolution at runtime
4. SObject type generation from schema
