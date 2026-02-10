/**
 * Apex Server — Bun HTTP server for Apex execution
 *
 * Uses apexrust WASM for parsing/transpilation and bun:sqlite for the database.
 * Start with: bun run apex-server.ts
 *
 * Endpoints:
 *   POST /deploy          — initialize DB + register Apex
 *   POST /apex/:class/:method — call a registered Apex handler
 *   GET  /apex/:class/:method — call with query params
 *   GET  /routes          — list registered routes
 */

import { Database } from "bun:sqlite";
import initWasm, {
  WasmSchema,
  convertSoqlToSql,
  transpileApex,
  parseApex,
} from "./pkg/apexrust.js";

// =============================================================================
// State
// =============================================================================

let db: Database | null = null;
let schema: any = null;
let handlers = new Map<
  string,
  {
    className: string;
    methodName: string;
    params: string[];
    fn: Function;
    cacheable: boolean;
  }
>();
let ready = false;

// =============================================================================
// Initialize WASM on startup
// =============================================================================

await initWasm();
console.log("[apex-server] WASM loaded");

// =============================================================================
// HTTP Server
// =============================================================================

const server = Bun.serve({
  port: 3000,
  async fetch(req) {
    const url = new URL(req.url);

    // CORS headers for browser access
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
    };

    if (req.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: corsHeaders });
    }

    try {
      // POST /deploy — initialize DB + register Apex
      if (url.pathname === "/deploy" && req.method === "POST") {
        const body = await req.json();
        const result = await deploy(body);
        return Response.json(result, { headers: corsHeaders });
      }

      // GET /routes — list registered routes
      if (url.pathname === "/routes") {
        return Response.json(
          { routes: Array.from(handlers.keys()) },
          { headers: corsHeaders },
        );
      }

      // /apex/:class/:method — call Apex handler
      if (url.pathname.startsWith("/apex/")) {
        const path = url.pathname.replace(/^\/apex\//, "");

        let params: Record<string, any> = {};
        if (
          req.method === "POST" ||
          req.method === "PUT" ||
          req.method === "PATCH"
        ) {
          const text = await req.text();
          if (text) {
            try {
              params = JSON.parse(text);
            } catch {}
          }
        } else {
          for (const [key, value] of url.searchParams) {
            try {
              params[key] = JSON.parse(value);
            } catch {
              params[key] = value;
            }
          }
        }

        const result = await callApex(path, params);
        return Response.json(result.body, {
          status: result.status,
          headers: corsHeaders,
        });
      }

      // Static file serving — serve from project directory
      const filePath =
        url.pathname === "/" ? "/dreamhouse-sw-demo.html" : url.pathname;
      const file = Bun.file(import.meta.dir + filePath);
      if (await file.exists()) {
        return new Response(file);
      }

      return Response.json(
        { error: "Not found" },
        { status: 404, headers: corsHeaders },
      );
    } catch (err: any) {
      return Response.json(
        { error: err.message },
        { status: 500, headers: corsHeaders },
      );
    }
  },
});

console.log(`[apex-server] Listening on http://localhost:${server.port}`);

// =============================================================================
// Deploy — initialize DB, schema, seed data, register Apex
// =============================================================================

async function deploy(body: {
  schema?: any[];
  ddl?: string[];
  seedSql?: string[];
  apex?: string | string[];
}) {
  // 1. Create or reset the database
  db = new Database(":memory:");

  // 2. Build schema
  schema = new WasmSchema();
  if (body.schema) {
    for (const obj of body.schema) {
      schema.addObject(obj);
    }
  }

  // 3. Execute DDL
  if (body.ddl) {
    for (const stmt of body.ddl) {
      db.run(stmt);
    }
  }

  // 4. Seed data
  if (body.seedSql) {
    for (const stmt of body.seedSql) {
      db.run(stmt);
    }
  }

  ready = true;
  console.log("[apex-server] Database initialized");

  // 5. Register Apex if provided
  const allRoutes: string[] = [];
  const apexSources = Array.isArray(body.apex)
    ? body.apex
    : body.apex
      ? [body.apex]
      : [];

  for (const source of apexSources) {
    const result = await registerApex(source);
    allRoutes.push(...result.routes);
  }

  return { success: true, routes: allRoutes };
}

// =============================================================================
// Register Apex — parse, transpile, extract routes, eval
// =============================================================================

async function registerApex(source: string) {
  if (!ready) throw new Error("Not initialized. Call /deploy first.");

  // 1. Parse
  const parseResult = parseApex(source);
  if (!parseResult || !parseResult.success) {
    throw new Error(`Parse error: ${parseResult?.error || "Unknown"}`);
  }

  // 2. Extract routes from source text
  const routes = extractRoutesFromSource(source);

  // 3. Transpile to JavaScript
  const transpileResult = transpileApex(source, {
    typescript: false,
    asyncDatabase: true,
    includeImports: false,
  });

  if (!transpileResult || !transpileResult.success) {
    throw new Error(`Transpile error: ${transpileResult?.error || "Unknown"}`);
  }

  // 4. Evaluate
  const $runtime = createRuntime();
  const classInstances = await evaluateTranspiledCode(
    transpileResult.typescript,
    $runtime,
  );

  // 5. Register handlers
  const registeredRoutes: string[] = [];
  for (const route of routes) {
    const key = `${route.className}/${route.methodName}`;
    const classObj = classInstances[route.className];
    if (!classObj) {
      console.warn(`[apex-server] Class ${route.className} not found`);
      continue;
    }

    const method = classObj[route.methodName];
    if (typeof method !== "function") {
      console.warn(`[apex-server] Method ${key} not found`);
      continue;
    }

    handlers.set(key, {
      className: route.className,
      methodName: route.methodName,
      params: route.params,
      fn: method,
      cacheable: route.cacheable,
    });
    registeredRoutes.push(key);
    console.log(`[apex-server] Registered /apex/${key}`);
  }

  return { routes: registeredRoutes, transpiled: transpileResult.typescript };
}

// =============================================================================
// Call Apex — look up handler, execute, return result
// =============================================================================

async function callApex(path: string, params: Record<string, any>) {
  if (!ready) {
    return {
      status: 503,
      body: { error: "Server not initialized. POST to /deploy first." },
    };
  }

  const handler = handlers.get(path);
  if (!handler) {
    return {
      status: 404,
      body: {
        error: `No handler for /apex/${path}`,
        availableRoutes: Array.from(handlers.keys()).map((r) => `/apex/${r}`),
      },
    };
  }

  try {
    const args = handler.params.map((name) =>
      params[name] !== undefined ? params[name] : null,
    );

    console.log(`[apex-server] /apex/${path}`, args);
    const result = await handler.fn(...args);
    const body = JSON.parse(JSON.stringify(result, mapReplacer));

    return { status: 200, body };
  } catch (err: any) {
    console.error(`[apex-server] Error in /apex/${path}:`, err);
    return { status: 500, body: { error: err.message } };
  }
}

// =============================================================================
// $runtime — bridge from transpiled Apex to bun:sqlite
// =============================================================================

function createRuntime() {
  return {
    async query(soql: string, binds?: Record<string, any>) {
      const convResult = convertSoqlToSql(soql, schema, "sqlite");
      if (!convResult.success) {
        throw new Error(`SOQL Error: ${convResult.error}`);
      }

      let sql: string = convResult.sql;

      if (binds && convResult.parameters) {
        for (const param of convResult.parameters) {
          const value = binds[param.originalName];
          let sqlValue: string;
          if (value === null || value === undefined) {
            sqlValue = "NULL";
          } else if (typeof value === "string") {
            sqlValue = `'${value.replace(/'/g, "''")}'`;
          } else if (typeof value === "number") {
            sqlValue = String(value);
          } else if (typeof value === "boolean") {
            sqlValue = value ? "1" : "0";
          } else {
            sqlValue = `'${String(value)}'`;
          }
          sql = sql.replace(param.placeholder, sqlValue);
        }
      }

      console.log("[apex-server] SQL:", sql);

      // bun:sqlite returns objects with column names as keys
      const rows = db!.query(sql).all();

      if (rows.length === 0) {
        if (soql.toUpperCase().includes("SELECT COUNT()")) return 0;
        return [];
      }

      // Check for COUNT query
      const firstRow = rows[0] as Record<string, any>;
      const keys = Object.keys(firstRow);
      if (keys.length === 1 && keys[0].toLowerCase().includes("count")) {
        return firstRow[keys[0]];
      }

      // Normalize column names to match Apex field naming
      return rows.map((row: any) => {
        const obj: Record<string, any> = {};
        for (const [col, val] of Object.entries(row)) {
          const fieldName = col
            .replace(/__c$/i, "__c")
            .replace(/^(\w)/, (m) => m.toUpperCase())
            .replace(/_(\w)/g, (_m, c) => "_" + c.toUpperCase());
          obj[fieldName] = val;
        }
        return obj;
      });
    },

    async insert(records: any) {
      const arr = Array.isArray(records) ? records : [records];
      console.log("[apex-server] Insert:", arr.length, "records");
      return arr.map(() => "new-id-" + Math.random().toString(36).substr(2, 9));
    },

    async update(records: any) {
      console.log("[apex-server] Update:", records);
    },

    async delete(records: any) {
      console.log("[apex-server] Delete:", records);
    },

    debug(msg: any) {
      console.log("[Apex Debug]", msg);
    },
  };
}

// =============================================================================
// Evaluate transpiled JavaScript
// =============================================================================

function evaluateTranspiledCode(jsCode: string, $runtime: any) {
  let cleanCode = jsCode
    .replace(/^export /gm, "")
    .replace(/export default /g, "")
    .replace(/export \{[^}]*\};?\n?/g, "")
    .replace(/^\/\/ Generated by ApexRust Transpiler\n/m, "")
    .replace(/^\/\/ Do not edit directly\n/m, "");

  const classNames: string[] = [];
  const classRegex = /(?:^|\s)class\s+(\w+)/gm;
  let match;
  while ((match = classRegex.exec(cleanCode)) !== null) {
    classNames.push(match[1]);
  }

  if (classNames.length === 0) return {};

  const shimCode = getApexShims();
  const returnObj = classNames.map((n) => `${n}: ${n}`).join(", ");
  const wrappedCode = `
    ${shimCode}
    ${cleanCode}
    return { ${returnObj} };
  `;

  const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
  const fn = new AsyncFunction("$runtime", wrappedCode);
  return fn($runtime);
}

// =============================================================================
// Extract @AuraEnabled routes from Apex source text
// =============================================================================

function extractRoutesFromSource(source: string) {
  const routes: Array<{
    className: string;
    methodName: string;
    params: string[];
    cacheable: boolean;
    urlMapping: string | null;
    httpMethod: string | null;
  }> = [];
  const lines = source.split("\n");

  const classMap = new Map<string, { urlMapping: string }>();
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    const restMatch = line.match(
      /@RestResource\s*\(\s*urlMapping\s*=\s*'([^']+)'/i,
    );
    if (restMatch) {
      for (let j = i + 1; j < lines.length; j++) {
        const classMatch = lines[j].match(/\bclass\s+(\w+)/i);
        if (classMatch) {
          classMap.set(classMatch[1], { urlMapping: restMatch[1] });
          break;
        }
      }
    }
  }

  let braceDepth = 0;
  let classStack: Array<{ name: string; depth: number }> = [];
  let pendingAnnotation: { cacheable: boolean; httpMethod?: string } | null =
    null;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    if (line.startsWith("@")) {
      pendingAnnotation = null;
    }

    const auraMatch = line.match(/@AuraEnabled(\s*\(([^)]*)\))?/i);
    if (auraMatch) {
      const annotationParams = auraMatch[2] || "";
      pendingAnnotation = {
        cacheable: /cacheable\s*=\s*true/i.test(annotationParams),
      };
      continue;
    }

    const httpMatch = line.match(
      /@(HttpGet|HttpPost|HttpPut|HttpPatch|HttpDelete)/i,
    );
    if (httpMatch) {
      pendingAnnotation = {
        cacheable: false,
        httpMethod: httpMatch[1].replace(/^Http/i, "").toUpperCase(),
      };
      continue;
    }

    const classMatch = line.match(/\bclass\s+(\w+)/i);
    if (classMatch) {
      classStack.push({ name: classMatch[1], depth: braceDepth });
    }

    if (pendingAnnotation) {
      const hasMethodStart =
        /(?:public|private|protected|global)\s/.test(line) &&
        line.includes("(");
      if (hasMethodStart) {
        let fullSig = line;
        let j = i;
        while (!fullSig.includes(")") && j < lines.length - 1) {
          j++;
          fullSig += " " + lines[j].trim();
        }

        const methodMatch = fullSig.match(
          /(?:public|private|protected|global)\s+(?:static\s+)?(?:[\w<>,\s\[\]]+)\s+(\w+)\s*\(([^)]*)\)/,
        );
        if (methodMatch) {
          const methodName = methodMatch[1];
          const paramsStr = methodMatch[2].trim();

          if (methodName !== classStack[classStack.length - 1]?.name) {
            const params = paramsStr
              ? paramsStr.split(",").map((p) => {
                  const parts = p.trim().split(/\s+/);
                  return parts[parts.length - 1];
                })
              : [];

            const className =
              classStack.length > 0
                ? classStack[classStack.length - 1].name
                : "Unknown";
            const classInfo = classMap.get(className) || {};

            routes.push({
              className,
              methodName,
              params,
              cacheable: pendingAnnotation.cacheable,
              urlMapping: (classInfo as any).urlMapping || null,
              httpMethod: pendingAnnotation.httpMethod || null,
            });
          }
          pendingAnnotation = null;
        }
      }
    }

    for (const ch of line) {
      if (ch === "{") braceDepth++;
      if (ch === "}") {
        braceDepth--;
        if (
          classStack.length > 0 &&
          braceDepth <= classStack[classStack.length - 1].depth
        ) {
          classStack.pop();
        }
      }
    }
  }

  return routes;
}

// =============================================================================
// JSON Map serializer
// =============================================================================

function mapReplacer(_key: string, value: any) {
  if (value instanceof Map) {
    const obj: Record<string, any> = {};
    for (const [k, v] of value) {
      obj[k] = v;
    }
    return obj;
  }
  return value;
}

// =============================================================================
// Apex Standard Library Shims
// =============================================================================

function getApexShims() {
  return `
    const System = {
      debug: function(msg) { console.log('[System.debug]', msg); },
      now: function() { return new Date(); },
      today: function() { const d = new Date(); d.setHours(0,0,0,0); return d; },
    };

    const _nativeParse = JSON.parse.bind(JSON);
    const _nativeStringify = JSON.stringify.bind(JSON);

    function toApexObject(val) {
      if (val === null || val === undefined) return val;
      if (Array.isArray(val)) return val.map(toApexObject);
      if (typeof val === 'object') {
        const map = new Map();
        for (const [k, v] of Object.entries(val)) {
          map.set(k, toApexObject(v));
        }
        return map;
      }
      return val;
    }

    JSON.serialize = function(obj) { return _nativeStringify(obj); };
    JSON.deserialize = function(jsonString, apexType) { return _nativeParse(jsonString); };
    JSON.deserializeUntyped = function(jsonString) { return toApexObject(_nativeParse(jsonString)); };
    JSON.serializePretty = function(obj) { return _nativeStringify(obj, null, 2); };

    class HttpRequest {
      constructor() { this._endpoint = ''; this._method = 'GET'; this._headers = new Map(); this._body = ''; this._timeout = 120000; }
      setEndpoint(url) { this._endpoint = url; }
      getEndpoint() { return this._endpoint; }
      setMethod(m) { this._method = m.toUpperCase(); }
      getMethod() { return this._method; }
      setHeader(k, v) { this._headers.set(k, v); }
      getHeader(k) { return this._headers.get(k) || null; }
      getHeaders() { return this._headers; }
      setBody(b) { this._body = b; }
      getBody() { return this._body; }
      setTimeout(t) { this._timeout = t; }
      getTimeout() { return this._timeout; }
      setCompressed(c) { this._compressed = c; }
      getCompressed() { return this._compressed || false; }
    }

    class HttpResponse {
      constructor(statusCode, status, body, headers) {
        this._statusCode = statusCode; this._status = status;
        this._body = body; this._headers = headers || new Map();
      }
      getStatusCode() { return this._statusCode; }
      getStatus() { return this._status; }
      getBody() { return this._body; }
      getHeader(k) { return this._headers.get(k.toLowerCase()) || null; }
      getHeaderKeys() { return Array.from(this._headers.keys()); }
    }

    class Http {
      async send(request) {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), request.getTimeout());
        try {
          const opts = { method: request.getMethod(), headers: Object.fromEntries(request.getHeaders()), signal: controller.signal };
          if (request.getMethod() !== 'GET' && request.getMethod() !== 'HEAD') opts.body = request.getBody();
          const response = await fetch(request.getEndpoint(), opts);
          const responseHeaders = new Map();
          response.headers.forEach((v, k) => responseHeaders.set(k.toLowerCase(), v));
          const body = await response.text();
          return new HttpResponse(response.status, response.statusText, body, responseHeaders);
        } catch (error) {
          if (error.name === 'AbortError') throw new Error('CalloutException: Request timed out');
          throw new Error('CalloutException: ' + error.message);
        } finally { clearTimeout(timeoutId); }
      }
    }

    if (!Array.prototype.get) { Array.prototype.get = function(i) { return this[i]; }; }
    if (!Array.prototype.set) { Array.prototype.set = function(i, v) { this[i] = v; }; }
    if (!Array.prototype.add) { Array.prototype.add = function(v) { this.push(v); }; }
    if (!('size' in Array.prototype)) {
      Object.defineProperty(Array.prototype, 'size', { configurable: true, get: function() { return this.length; } });
    }
  `;
}
