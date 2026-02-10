/**
 * Apex SharedWorker
 *
 * Long-lived worker that owns SQLite + WASM + transpiled Apex handlers.
 * Survives Service Worker restarts. Communicates with:
 *   - Main page: via SharedWorker port (init, registerApex, status)
 *   - Service Worker: via MessageChannel port (apex fetch requests)
 */

// Classic worker — use importScripts for all dependencies
importScripts(
  "https://cdnjs.cloudflare.com/ajax/libs/sql.js/1.10.3/sql-wasm.js",
);
importScripts("./pkg-nomodules/apexrust.js");

// State
let db = null;
let schema = null;
let apexrust = null;
let handlers = new Map(); // route -> { className, methodName, params, fn, cacheable }
let ready = false;
let initPromise = null;

// All connected ports (pages + service worker bridge)
const ports = [];

// ============================================================================
// SharedWorker connection handler
// ============================================================================

self.addEventListener("connect", (event) => {
  const port = event.ports[0];
  ports.push(port);

  port.addEventListener("message", (e) => {
    // The page may transfer a MessageChannel port for the Service Worker bridge
    if (e.data.type === "set-sw-port" && e.ports && e.ports[0]) {
      const swPort = e.ports[0];
      swPort.onmessage = (evt) => handleMessage(swPort, evt.data);
      swPort.start();
      console.log("[ApexWorker] Service Worker bridge port connected");
      return;
    }
    handleMessage(port, e.data);
  });
  port.start();
});

// ============================================================================
// Message dispatch
// ============================================================================

async function handleMessage(port, data) {
  const { type, id } = data;

  const respond = (result) => {
    port.postMessage({ type: "response", id, ...result });
  };

  try {
    switch (type) {
      case "init": {
        await handleInit(data);
        respond({ success: true });
        break;
      }
      case "registerApex": {
        const result = await handleRegisterApex(data);
        respond({ success: true, ...result });
        break;
      }
      case "exec-sql": {
        const result = await handleExecSql(data);
        respond({ success: true, ...result });
        break;
      }
      case "status": {
        respond({
          success: true,
          ready,
          routes: Array.from(handlers.keys()),
        });
        break;
      }
      // Service Worker forwards fetch requests here
      case "apex-fetch": {
        const result = await handleApexFetch(data);
        respond(result);
        break;
      }
      default:
        respond({ success: false, error: `Unknown message type: ${type}` });
    }
  } catch (err) {
    respond({ success: false, error: err.message });
  }
}

// ============================================================================
// Initialization — load WASM + SQLite, create schema and seed data
// ============================================================================

async function handleInit(data) {
  if (initPromise) return initPromise;

  initPromise = (async () => {
    // 1. Initialize ApexRust WASM
    await wasm_bindgen("./pkg-nomodules/apexrust_bg.wasm");

    apexrust = {
      WasmSchema: wasm_bindgen.WasmSchema,
      convertSoqlToSql: wasm_bindgen.convertSoqlToSql,
      transpileApex: wasm_bindgen.transpileApex,
      parseApex: wasm_bindgen.parseApex,
    };

    // 2. Load SQLite
    const SQL = await initSqlJs({
      locateFile: (file) =>
        `https://cdnjs.cloudflare.com/ajax/libs/sql.js/1.10.3/${file}`,
    });
    db = new SQL.Database();

    // 3. Create schema from provided object definitions
    schema = new apexrust.WasmSchema();
    if (data.schema) {
      for (const obj of data.schema) {
        schema.addObject(obj);
      }
    }

    // 4. Execute DDL and seed SQL if provided
    if (data.ddl) {
      for (const stmt of data.ddl) {
        db.run(stmt);
      }
    }
    if (data.seedSql) {
      for (const stmt of data.seedSql) {
        db.run(stmt);
      }
    }

    ready = true;
  })();

  return initPromise;
}

// ============================================================================
// Register Apex — transpile source, find @AuraEnabled methods, register routes
// ============================================================================

async function handleRegisterApex(data) {
  if (!ready) throw new Error('Worker not initialized. Send "init" first.');

  const { source } = data;

  // 1. Verify the source parses correctly
  const parseResult = apexrust.parseApex(source);
  if (!parseResult || !parseResult.success) {
    throw new Error(`Parse error: ${parseResult?.error || "Unknown"}`);
  }

  // 2. Extract route info by scanning source text for @AuraEnabled methods
  const routes = extractRoutesFromSource(source);

  // 3. Transpile to JavaScript
  const transpileResult = apexrust.transpileApex(source, {
    typescript: false,
    asyncDatabase: true,
    includeImports: false,
  });

  if (!transpileResult || !transpileResult.success) {
    throw new Error(`Transpile error: ${transpileResult?.error || "Unknown"}`);
  }

  // 4. Evaluate the transpiled code and register handlers
  const $runtime = createRuntime();
  const classInstances = await evaluateTranspiledCode(
    transpileResult.typescript,
    $runtime,
  );

  // 5. Register each @AuraEnabled method as a route
  const registeredRoutes = [];
  for (const route of routes) {
    const key = `${route.className}/${route.methodName}`;
    const classObj = classInstances[route.className];
    if (!classObj) {
      console.warn(
        `[ApexWorker] Class ${route.className} not found in transpiled output`,
      );
      continue;
    }

    const method = classObj[route.methodName];
    if (typeof method !== "function") {
      console.warn(
        `[ApexWorker] Method ${route.className}.${route.methodName} not found`,
      );
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
  }

  return {
    routes: registeredRoutes,
    transpiled: transpileResult.typescript,
  };
}

// ============================================================================
// Handle Apex fetch — called by Service Worker via MessageChannel
// ============================================================================

async function handleApexFetch(data) {
  if (!ready) {
    return { success: false, status: 503, error: "Worker not initialized" };
  }

  const { path, method, params } = data;
  const handler = handlers.get(path);

  if (!handler) {
    return {
      success: false,
      status: 404,
      error: `No handler for /apex/${path}`,
      availableRoutes: Array.from(handlers.keys()).map((r) => `/apex/${r}`),
    };
  }

  try {
    const args = handler.params.map((name) => {
      return params[name] !== undefined ? params[name] : null;
    });

    console.log(`[ApexWorker] ${method} /apex/${path}`, args);
    const result = await handler.fn(...args);

    // Serialize, converting Maps to plain objects
    const body = JSON.stringify(result, mapReplacer);

    return {
      success: true,
      status: 200,
      body,
      cacheable: handler.cacheable,
    };
  } catch (err) {
    console.error(`[ApexWorker] Error in /apex/${path}:`, err);
    return {
      success: false,
      status: 500,
      error: err.message,
    };
  }
}

// ============================================================================
// Execute raw SQL
// ============================================================================

async function handleExecSql(data) {
  if (!db) throw new Error("Database not initialized");
  const results = db.exec(data.sql);
  if (results.length === 0) return { rows: [] };
  return { columns: results[0].columns, rows: results[0].values };
}

// ============================================================================
// Extract @AuraEnabled/@RestResource routes from Apex source text
// ============================================================================

function extractRoutesFromSource(source) {
  const routes = [];
  const lines = source.split("\n");

  // Pass 1: Find @RestResource class annotations
  const classMap = new Map();
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

  // Pass 2: Find @AuraEnabled methods
  let braceDepth = 0;
  let classStack = [];
  let pendingAnnotation = null;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    // Any annotation resets the pending state
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
              urlMapping: classInfo.urlMapping || null,
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

// ============================================================================
// Create $runtime — bridge from transpiled Apex to SQLite
// ============================================================================

function createRuntime() {
  return {
    async query(soql, binds) {
      const convResult = apexrust.convertSoqlToSql(soql, schema, "sqlite");
      if (!convResult.success) {
        throw new Error(`SOQL Error: ${convResult.error}`);
      }

      let sql = convResult.sql;

      if (binds && convResult.parameters) {
        for (const param of convResult.parameters) {
          const value = binds[param.originalName];
          let sqlValue;
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

      console.log("[ApexWorker] SQL:", sql);

      const results = db.exec(sql);
      if (results.length === 0) {
        if (soql.toUpperCase().includes("SELECT COUNT()")) return 0;
        return [];
      }

      const { columns, values } = results[0];
      if (columns.length === 1 && columns[0].toLowerCase().includes("count")) {
        return values[0][0];
      }

      return values.map((row) => {
        const obj = {};
        columns.forEach((col, i) => {
          const fieldName = col
            .replace(/__c$/i, "__c")
            .replace(/^(\w)/, (m) => m.toUpperCase())
            .replace(/_(\w)/g, (m, c) => "_" + c.toUpperCase());
          obj[fieldName] = row[i];
        });
        return obj;
      });
    },

    async insert(records) {
      const arr = Array.isArray(records) ? records : [records];
      console.log("[ApexWorker] Insert:", arr.length, "records");
      return arr.map(() => "new-id-" + Math.random().toString(36).substr(2, 9));
    },

    async update(records) {
      console.log("[ApexWorker] Update:", records);
    },

    async delete(records) {
      console.log("[ApexWorker] Delete:", records);
    },

    debug(msg) {
      console.log("[Apex Debug]", msg);
    },
  };
}

// ============================================================================
// Evaluate transpiled JavaScript
// ============================================================================

async function evaluateTranspiledCode(jsCode, $runtime) {
  let cleanCode = jsCode
    .replace(/^export /gm, "")
    .replace(/export default /g, "")
    .replace(/export \{[^}]*\};?\n?/g, "")
    .replace(/^\/\/ Generated by ApexRust Transpiler\n/m, "")
    .replace(/^\/\/ Do not edit directly\n/m, "");

  const classNames = [];
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
  return await fn($runtime);
}

// ============================================================================
// JSON Map serializer
// ============================================================================

function mapReplacer(key, value) {
  if (value instanceof Map) {
    const obj = {};
    for (const [k, v] of value) {
      obj[k] = v;
    }
    return obj;
  }
  return value;
}

// ============================================================================
// Apex Standard Library Shims
// ============================================================================

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

    class Url {
      constructor(url) { this._url = url; this._parsed = new URL(url); }
      static getOrgDomainUrl() { return new Url(self.location.origin); }
      toExternalForm() { return this._url; }
      getHost() { return this._parsed.hostname; }
      getProtocol() { return this._parsed.protocol.replace(':', ''); }
      toString() { return this._url; }
    }

    if (!Array.prototype.get) { Array.prototype.get = function(i) { return this[i]; }; }
    if (!Array.prototype.set) { Array.prototype.set = function(i, v) { this[i] = v; }; }
    if (!Array.prototype.add) { Array.prototype.add = function(v) { this.push(v); }; }
    if (!('size' in Array.prototype)) {
      Object.defineProperty(Array.prototype, 'size', { configurable: true, get: function() { return this.length; } });
    }
  `;
}
