# Apexion

A Rust-based Apex parser with SOQL-to-SQL conversion and TypeScript transpilation. Runs in the browser via WebAssembly.

## Features

- **Apex Parser** - Full Apex language support (classes, interfaces, triggers, enums, SOQL, SOSL)
- **SOQL to SQL** - Convert SOQL queries to SQLite or PostgreSQL
- **Apex to TypeScript** - Transpile Apex code to TypeScript/JavaScript
- **Browser Execution** - Run transpiled Apex against in-browser SQLite via WebAssembly
- **Sales Cloud Schema** - Built-in schema for 21 standard Salesforce objects

## Demo

Try it live: [demo.html](demo.html)

1. Parse Apex code and extract SOQL queries
2. Convert SOQL to SQL (SQLite/PostgreSQL)
3. Generate DDL for Salesforce schema
4. Transpile Apex to TypeScript
5. Execute transpiled Apex against in-browser SQLite

## Quick Start

### Build

```bash
# Native (for tests)
cargo test

# WebAssembly
wasm-pack build --target web --features wasm
```

### Run Demo

```bash
python3 -m http.server 8080
# Open http://localhost:8080/demo.html
```

## Example

### Apex Input
```apex
public class AccountService {
    public List<Account> getActiveAccounts() {
        return [SELECT Id, Name, Industry 
                FROM Account 
                WHERE IsDeleted = false 
                ORDER BY Name];
    }
}
```

### Transpiled TypeScript
```typescript
export class AccountService {
    public async getActiveAccounts(): Promise<Account[]> {
        return await $runtime.query(
            "SELECT Id, Name, Industry FROM Account WHERE IsDeleted = false ORDER BY Name"
        );
    }
}
```

### Generated SQL (SQLite)
```sql
SELECT t0.id, t0.name, t0.industry
FROM "account" t0
WHERE t0.is_deleted = 0
ORDER BY t0.name
```

## Architecture

```
src/
├── parser.rs           # Apex + SOQL parser
├── ast.rs              # AST definitions
├── sql/
│   ├── converter.rs    # SOQL → SQL conversion
│   ├── dialect.rs      # SQLite/PostgreSQL differences
│   ├── ddl.rs          # CREATE TABLE generation
│   └── schema.rs       # Salesforce schema model
├── transpile/
│   └── codegen.rs      # Apex → TypeScript
└── wasm.rs             # WebAssembly bindings

runtime/
├── apex-runtime.ts     # JavaScript runtime ($runtime)
└── apex-stdlib.ts      # Apex standard library shims
```

## WASM API

```javascript
import init, { 
    parseApex, 
    convertSoqlToSql, 
    transpileApex,
    generateDdl,
    WasmSchema 
} from './pkg/apexion.js';

await init();

// Parse Apex
const result = parseApex(apexCode);

// Convert SOQL to SQL
const schema = new WasmSchema();
schema.loadSalesCloud();
const sql = convertSoqlToSql(soql, schema, 'sqlite');

// Transpile Apex to TypeScript
const ts = transpileApex(apexCode, { typescript: true });

// Generate DDL
const ddl = generateDdl(schema, 'sqlite');
```

## SOQL Support

- SELECT with fields, relationships, subqueries, aggregates
- WHERE with all operators (=, !=, <, >, LIKE, IN, NOT IN)
- ORDER BY (including parent fields like `Account.Name`)
- GROUP BY, HAVING
- LIMIT, OFFSET
- Bind variables (`:variable`)
- Date literals (TODAY, LAST_N_DAYS, THIS_MONTH, etc.)

## SQL Dialect Differences

| Feature | PostgreSQL | SQLite |
|---------|------------|--------|
| Boolean | `BOOLEAN` | `INTEGER` |
| Parameters | `$1, $2` | `?1, ?2` |
| Date functions | `CURRENT_DATE` | `date('now')` |
| JSON aggregation | `json_agg()` | `json_group_array()` |

## Documentation

- [CLAUDE.md](CLAUDE.md) - Architecture and implementation details
- [NOTES.md](NOTES.md) - Development notes
- [PRODUCTION_ROADMAP.md](PRODUCTION_ROADMAP.md) - Gaps and estimates for production use

## License

MIT
