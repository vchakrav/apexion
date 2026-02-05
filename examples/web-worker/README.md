# ApexRust Web Worker Example

This example demonstrates how to use ApexRust in a browser via WebAssembly with a Web Worker for isolated execution.

## Architecture

```
┌─────────────────────┐     postMessage      ┌─────────────────────────────┐
│    Main Thread      │ ──────────────────►  │        Web Worker           │
│                     │                      │                             │
│  - UI               │                      │  - ApexRust (WASM)          │
│  - Editor           │  ◄──────────────────  │  - sql.js (SQLite WASM)     │
│  - Results display  │     query results    │  - Apex execution           │
└─────────────────────┘                      └─────────────────────────────┘
```

## Building

### Prerequisites

1. Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. Install Node.js (for bundling, optional)

### Build WASM

From the repository root:

```bash
# Build the WASM package
wasm-pack build --target web --features wasm

# This creates the pkg/ directory with:
# - apexrust.js      (JavaScript glue code)
# - apexrust_bg.wasm (WebAssembly binary)
# - apexrust.d.ts    (TypeScript definitions)
```

### Run the Example

Option 1: Using a simple HTTP server (Python):
```bash
cd examples/web-worker
python3 -m http.server 8080
# Open http://localhost:8080
```

Option 2: Using Node.js http-server:
```bash
npx http-server examples/web-worker -p 8080
```

## Files

- `apex-worker.ts` - Web Worker that handles Apex parsing, SOQL conversion, and SQL execution
- `main.ts` - Example showing how to communicate with the worker from the main thread
- `index.html` - Interactive demo page (works in demo mode without WASM build)

## API

### Worker Messages

**Initialize:**
```typescript
worker.postMessage({ type: 'init' });
// Response: { type: 'ready' }
```

**Parse Apex:**
```typescript
worker.postMessage({ 
  type: 'parse', 
  apex: 'public class Foo { ... }' 
});
// Response: { type: 'result', data: { success: true, ast: '...', soqlQueries: [...] } }
```

**Convert SOQL to SQL:**
```typescript
worker.postMessage({
  type: 'convert-soql',
  soql: 'SELECT Id FROM Account',
  schema: { objects: [...] },
  dialect: 'sqlite' // or 'postgres'
});
// Response: { type: 'result', data: { success: true, sql: '...', parameters: [...] } }
```

**Execute SQL:**
```typescript
worker.postMessage({
  type: 'execute-sql',
  sql: 'SELECT * FROM account',
  params: []
});
// Response: { type: 'result', data: [{ columns: [...], values: [...] }] }
```

## Using with sql.js

For full SOQL execution, you'll need [sql.js](https://github.com/sql-js/sql.js):

```bash
npm install sql.js
```

The worker can then:
1. Generate DDL from your Salesforce schema
2. Create tables in an in-memory SQLite database
3. Insert mock/test data
4. Convert SOQL queries to SQL
5. Execute against the database

## Security Considerations

The Web Worker provides natural isolation:
- No access to DOM
- Separate memory space
- Can be terminated if code hangs
- Limited capability set

For additional security with untrusted Apex code, consider:
- Using a sandboxed iframe for the worker
- Implementing execution timeouts
- Validating input before parsing
