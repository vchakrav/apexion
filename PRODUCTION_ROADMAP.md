# Production Roadmap for ApexRust

## Current State

The demo proves the concept works end-to-end:
- Parse Apex → Transpile to TypeScript → Compile to JS → Execute against SQLite

However, several gaps exist for production use.

---

## 1. Transpiler Completeness

### Currently Working
- Classes, methods, constructors, properties
- Basic control flow (if/else, for, while, switch, try/catch)
- SOQL queries → async $runtime.query()
- DML statements → async $runtime.insert/update/delete()
- Basic expressions and operators

### Missing / Incomplete

| Feature | Effort | Priority |
|---------|--------|----------|
| **Inheritance** (`extends`, `super` calls) | 1-2 days | High |
| **Interfaces** (`implements`, interface methods) | 1 day | High |
| **Static methods/fields** | 0.5 days | High |
| **Inner classes** | 1 day | Medium |
| **Enums** (proper TypeScript enums) | 0.5 days | Medium |
| **Triggers** (event context) | 1-2 days | Medium |
| **Exception types** (custom exceptions) | 0.5 days | Medium |
| **Annotations** (@isTest, @AuraEnabled, etc.) | 1 day | Low |
| **SOSL queries** | 2-3 days | Low |
| **Batch/Queueable/Schedulable** | 2-3 days | Low |

### Code Quality Issues

1. **SOQL string reconstruction** - Currently reconstructs SOQL from AST. Edge cases may produce invalid SOQL.
   - Fix: Store original source span and extract literal text, OR comprehensive SOQL printer.

2. **Type inference** - No type checking means some edge cases may produce incorrect TypeScript.
   - Mitigation: Generate `any` for ambiguous cases, let TypeScript compiler catch issues.

3. **Method overloading** - Apex supports overloading, TypeScript doesn't directly.
   - Fix: Generate union types or use runtime dispatch.

---

## 2. Runtime Completeness

### Current Runtime
- Basic SOQL query execution
- Basic DML (insert/update/delete/upsert)
- SQLite adapter

### Missing

| Feature | Effort | Priority |
|---------|--------|----------|
| **Proper SObject types** | 2-3 days | High |
| **Relationship traversal** (Account.Contacts) | 1-2 days | High |
| **Database.SaveResult handling** | 1 day | High |
| **Partial success DML** (allOrNone=false) | 1 day | Medium |
| **Limits class** (Limits.getQueries(), etc.) | 1 day | Medium |
| **UserInfo class** | 0.5 days | Medium |
| **Schema.* methods** | 2-3 days | Medium |
| **Test.* methods** | 2-3 days | Medium |
| **JSON serialization/deserialization** | 1 day | Medium |
| **Http/HttpRequest/HttpResponse** | 2-3 days | Low |

### Standard Library Gaps

Current `apex-stdlib.ts` has basics. Production needs:

```
Estimated additions:
- String: ~90% complete, add format(), split() edge cases
- List: ~80% complete, add sort comparators, deepClone
- Set: ~90% complete
- Map: ~90% complete
- Date/DateTime: ~70% complete, add timezone handling
- Decimal: NOT IMPLEMENTED (need arbitrary precision)
- Blob: NOT IMPLEMENTED
- Pattern/Matcher: NOT IMPLEMENTED (regex)
- Type: NOT IMPLEMENTED (runtime type info)
- Comparable/Comparator: NOT IMPLEMENTED
```

---

## 3. Schema & Data Model

### Current State
- Hardcoded Sales Cloud schema (21 objects)
- Manual field mapping

### Production Needs

1. **Schema Import from Salesforce**
   ```typescript
   // Fetch from org via jsforce/REST API
   const schema = await connection.describe('Account');
   // Convert to ApexRust schema format
   ```

2. **Dynamic Schema Loading**
   - Load schema at runtime, not compile time
   - Handle custom objects and fields
   - Handle managed package objects

3. **Type Generation**
   ```typescript
   // Generate TypeScript interfaces from schema
   interface Account {
     Id: string;
     Name: string;
     Industry: string;
     // ... all fields with correct types
   }
   ```

4. **Relationship Metadata**
   - Parent lookups (AccountId → Account)
   - Child relationships (Account.Contacts)
   - Polymorphic lookups (OwnerId → User | Group)

**Effort: 3-5 days**

---

## 4. Database Backend Options

### Option A: SQLite (Current)
- **Pros**: Works offline, fast, no server needed
- **Cons**: Single user, no real persistence in browser, sync complexity
- **Use case**: Local development, testing, demos

### Option B: PostgreSQL/MySQL
- **Pros**: Real database, multi-user, persistent
- **Cons**: Requires server, network latency
- **Use case**: Server-side execution, staging environments

### Option C: Salesforce Org (via API)
- **Pros**: Real data, real permissions, production-ready
- **Cons**: API limits, network latency, auth complexity
- **Use case**: Production execution against real Salesforce data

### Option D: Hybrid
- SQLite for local cache
- Sync to Salesforce for persistence
- **Use case**: Offline-capable apps

**Recommendation**: Build adapter interface (done), implement each backend as needed.

---

## 5. Execution Environment

### Current: Browser eval()
- Works for demo
- Security concerns (eval is dangerous)
- No isolation

### Production Options

#### Option 1: Web Worker (Recommended for Browser)
```typescript
// Main thread
const worker = new Worker('apex-worker.js');
worker.postMessage({ code: transpiledCode, method: 'getAccounts' });
worker.onmessage = (e) => console.log(e.data.result);

// Worker
self.onmessage = async (e) => {
  const result = await executeApex(e.data.code, e.data.method);
  self.postMessage({ result });
};
```

**Pros**: Isolated, non-blocking, can be terminated
**Effort**: 2-3 days

#### Option 2: Node.js VM (Server)
```typescript
import { VM } from 'vm2';
const vm = new VM({ timeout: 5000, sandbox: { $runtime } });
const result = vm.run(transpiledCode);
```

**Pros**: Sandboxed, timeout support, server-side
**Effort**: 1-2 days

#### Option 3: Deno (Server)
```typescript
// Use Deno's built-in isolation
const worker = new Worker(new URL("./worker.ts", import.meta.url).href, {
  type: "module",
  deno: { permissions: { net: false, read: false, write: false } },
});
```

**Pros**: Excellent sandboxing, modern runtime
**Effort**: 1-2 days

#### Option 4: Cloudflare Workers / Edge
- Already sandboxed V8 isolates
- Perfect for serverless execution
- **Effort**: 1 day (mostly config)

---

## 6. Error Handling & Debugging

### Current State
- Basic error messages
- No source maps
- No stack trace mapping

### Production Needs

1. **Source Maps**
   - Map TypeScript errors back to Apex line numbers
   - Effort: 2-3 days

2. **Structured Errors**
   ```typescript
   class ApexExecutionError extends Error {
     apexLineNumber: number;
     apexFileName: string;
     apexMethodName: string;
     originalError: Error;
   }
   ```

3. **Governor Limits Simulation**
   ```typescript
   class LimitsTracker {
     queries = 0;
     dmlStatements = 0;
     cpuTime = 0;
     
     checkQueryLimit() {
       if (++this.queries > 100) {
         throw new LimitException('Too many SOQL queries: 101');
       }
     }
   }
   ```
   **Effort**: 2-3 days

4. **Logging/Tracing**
   - Capture System.debug() output
   - Execution timing
   - Query performance
   **Effort**: 1 day

---

## 7. Testing Support

### For Testing Transpiled Code

1. **Test Framework Integration**
   ```typescript
   // Jest/Vitest tests for transpiled Apex
   describe('AccountService', () => {
     it('should return active accounts', async () => {
       const service = new AccountService($runtime);
       const accounts = await service.getActiveAccounts();
       expect(accounts).toHaveLength(5);
     });
   });
   ```

2. **Mock Runtime**
   ```typescript
   const mockRuntime = {
     query: jest.fn().mockResolvedValue([{ Id: '001xxx', Name: 'Test' }]),
     insert: jest.fn().mockResolvedValue(['001xxx']),
   };
   ```

3. **Test Data Builders**
   ```typescript
   const account = TestDataBuilder.account()
     .withName('Test Corp')
     .withIndustry('Technology')
     .build();
   ```

**Effort**: 3-5 days for full test framework

---

## 8. Security Considerations

### Code Execution Security
- [ ] Sandbox execution (Web Worker / VM)
- [ ] Timeout enforcement
- [ ] Memory limits
- [ ] No file system access
- [ ] No network access (unless explicitly allowed)

### Data Security
- [ ] Row-level security (if not using Salesforce backend)
- [ ] Field-level security
- [ ] Sharing rules simulation
- [ ] User context (running user)

### Input Validation
- [ ] SQL injection prevention (parameterized queries ✓)
- [ ] SOQL injection prevention
- [ ] XSS prevention in outputs

**Effort**: 3-5 days for basic security hardening

---

## 9. Performance Optimization

### Current Performance
- Parsing: Fast (Rust/WASM)
- Transpilation: Fast
- Execution: Depends on runtime

### Optimization Opportunities

1. **Caching**
   - Cache parsed AST
   - Cache transpiled JavaScript
   - Cache compiled TypeScript

2. **Incremental Compilation**
   - Only re-transpile changed classes
   - Dependency tracking

3. **Query Optimization**
   - Query plan caching
   - Prepared statements
   - Connection pooling (for server backends)

4. **WASM Size**
   - Current: ~2MB
   - With wasm-opt: ~1MB
   - With compression: ~300KB
   - Tree-shaking unused features

**Effort**: 2-3 days for basic optimizations

---

## 10. Deployment & Distribution

### NPM Package
```bash
npm install @apexrust/core @apexrust/runtime
```

Structure:
```
@apexrust/core        # WASM parser + transpiler
@apexrust/runtime     # JavaScript runtime
@apexrust/sqlite      # SQLite adapter
@apexrust/postgres    # PostgreSQL adapter
@apexrust/salesforce  # Salesforce API adapter
```

### CDN Distribution
```html
<script src="https://unpkg.com/@apexrust/core"></script>
```

**Effort**: 2-3 days for packaging and publishing

---

## Summary: Minimum Viable Production

### Phase 1: Core Stability (1-2 weeks)
- [ ] Fix transpiler edge cases (inheritance, statics, interfaces)
- [ ] Complete SOQL string reconstruction
- [ ] Proper error handling with source mapping
- [ ] Web Worker execution environment

### Phase 2: Runtime Completeness (1-2 weeks)
- [ ] Complete SObject types
- [ ] Relationship traversal
- [ ] Full stdlib (String, List, Map, Date, etc.)
- [ ] Governor limits simulation

### Phase 3: Integration (1 week)
- [ ] Schema import from Salesforce
- [ ] TypeScript type generation
- [ ] PostgreSQL adapter
- [ ] NPM packaging

### Phase 4: Production Hardening (1-2 weeks)
- [ ] Security review and sandboxing
- [ ] Performance optimization
- [ ] Comprehensive test suite
- [ ] Documentation

**Total Estimate: 4-7 weeks for production-ready MVP**

---

## Use Cases This Enables

1. **Apex Unit Testing Outside Salesforce**
   - Run tests locally without deploying
   - Faster feedback loop
   - Mock external services

2. **Apex Code Migration**
   - Gradually migrate Apex to TypeScript
   - Run both side-by-side
   - Compare outputs

3. **Offline Salesforce Apps**
   - Mobile apps with offline support
   - Sync when connected

4. **Apex Learning/Playground**
   - Interactive Apex tutorials
   - Code playground without Salesforce org

5. **CI/CD Validation**
   - Static analysis
   - Query validation
   - Syntax checking before deployment
