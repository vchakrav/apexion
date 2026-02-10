/**
 * <apex-runtime> Web Component
 *
 * Self-contained Apex execution environment. Owns WASM + SQLite + transpiled
 * Apex handlers. No workers needed — everything runs in the main thread.
 *
 * Usage:
 *   <apex-runtime id="apex"></apex-runtime>
 *   <script type="module">
 *     const apex = document.getElementById('apex');
 *     await apex.initialize({ schema, ddl, seedSql });
 *     await apex.registerApex(apexSource);
 *     const result = await apex.callApex('ClassName/methodName', { param1: 'value' });
 *     console.log(apex.routes);
 *   </script>
 */

import initWasm, {
  WasmSchema,
  convertSoqlToSql,
  transpileApex,
  parseApex,
} from "./pkg/apexrust.js";

class ApexRuntime extends HTMLElement {
  constructor() {
    super();
    this._db = null;
    this._schema = null;
    this._handlers = new Map();
    this._ready = false;
    this._initPromise = null;
  }

  // ===========================================================================
  // Public API
  // ===========================================================================

  /**
   * Initialize WASM + SQLite, create tables, seed data.
   * @param {{ schema?: object[], ddl?: string[], seedSql?: string[] }} options
   */
  async initialize({ schema, ddl, seedSql } = {}) {
    if (this._initPromise) return this._initPromise;

    this._initPromise = (async () => {
      // 1. Initialize ApexRust WASM
      await initWasm();

      // 2. Load SQLite
      const SQL = await initSqlJs({
        locateFile: (file) =>
          `https://cdnjs.cloudflare.com/ajax/libs/sql.js/1.10.3/${file}`,
      });
      this._db = new SQL.Database();

      // 3. Create schema from provided object definitions
      this._schema = new WasmSchema();
      if (schema) {
        for (const obj of schema) {
          this._schema.addObject(obj);
        }
      }

      // 4. Execute DDL and seed SQL
      if (ddl) {
        for (const stmt of ddl) {
          this._db.run(stmt);
        }
      }
      if (seedSql) {
        for (const stmt of seedSql) {
          this._db.run(stmt);
        }
      }

      this._ready = true;
    })();

    return this._initPromise;
  }

  /**
   * Register Apex source code — transpiles and registers @AuraEnabled methods.
   * @param {string} source - Apex source code
   * @returns {{ routes: string[], transpiled: string }}
   */
  async registerApex(source) {
    if (!this._ready) throw new Error("Not initialized. Call initialize() first.");

    // 1. Verify the source parses
    const parseResult = parseApex(source);
    if (!parseResult || !parseResult.success) {
      throw new Error(`Parse error: ${parseResult?.error || "Unknown"}`);
    }

    // 2. Extract route info from source text
    const routes = this._extractRoutesFromSource(source);

    // 3. Transpile to JavaScript
    const transpileResult = transpileApex(source, {
      typescript: false,
      asyncDatabase: true,
      includeImports: false,
    });

    if (!transpileResult || !transpileResult.success) {
      throw new Error(`Transpile error: ${transpileResult?.error || "Unknown"}`);
    }

    // 4. Evaluate the transpiled code
    const $runtime = this._createRuntime();
    const classInstances = await this._evaluateTranspiledCode(
      transpileResult.typescript,
      $runtime,
    );

    // 5. Register each @AuraEnabled method as a route
    const registeredRoutes = [];
    for (const route of routes) {
      const key = `${route.className}/${route.methodName}`;
      const classObj = classInstances[route.className];
      if (!classObj) {
        console.warn(`[apex-runtime] Class ${route.className} not found in transpiled output`);
        continue;
      }

      const method = classObj[route.methodName];
      if (typeof method !== "function") {
        console.warn(`[apex-runtime] Method ${route.className}.${route.methodName} not found`);
        continue;
      }

      this._handlers.set(key, {
        className: route.className,
        methodName: route.methodName,
        params: route.params,
        fn: method,
        cacheable: route.cacheable,
      });
      registeredRoutes.push(key);
    }

    return { routes: registeredRoutes, transpiled: transpileResult.typescript };
  }

  /**
   * Call a registered Apex method.
   * @param {string} route - "ClassName/methodName"
   * @param {object} params - named parameters
   * @returns {any} - the method result
   */
  async callApex(route, params = {}) {
    if (!this._ready) throw new Error("Not initialized. Call initialize() first.");

    const handler = this._handlers.get(route);
    if (!handler) {
      throw new Error(
        `No handler for ${route}. Available: ${this.routes.join(", ")}`,
      );
    }

    const args = handler.params.map((name) =>
      params[name] !== undefined ? params[name] : null,
    );

    const result = await handler.fn(...args);
    // Convert Maps to plain objects for clean serialization
    return JSON.parse(JSON.stringify(result, _mapReplacer));
  }

  /**
   * Execute raw SQL against the SQLite database.
   * @param {string} sql
   * @returns {{ columns?: string[], rows: any[][] }}
   */
  execSql(sql) {
    if (!this._db) throw new Error("Database not initialized");
    const results = this._db.exec(sql);
    if (results.length === 0) return { rows: [] };
    return { columns: results[0].columns, rows: results[0].values };
  }

  /** @returns {string[]} - list of registered routes */
  get routes() {
    return Array.from(this._handlers.keys());
  }

  /** @returns {boolean} */
  get ready() {
    return this._ready;
  }

  // ===========================================================================
  // Internal — route extraction from Apex source text
  // ===========================================================================

  _extractRoutesFromSource(source) {
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

  // ===========================================================================
  // Internal — $runtime bridge from transpiled Apex to SQLite
  // ===========================================================================

  _createRuntime() {
    const db = this._db;
    const schema = this._schema;

    return {
      async query(soql, binds) {
        const convResult = convertSoqlToSql(soql, schema, "sqlite");
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

        console.log("[apex-runtime] SQL:", sql);

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
        console.log("[apex-runtime] Insert:", arr.length, "records");
        return arr.map(() => "new-id-" + Math.random().toString(36).substr(2, 9));
      },

      async update(records) {
        console.log("[apex-runtime] Update:", records);
      },

      async delete(records) {
        console.log("[apex-runtime] Delete:", records);
      },

      debug(msg) {
        console.log("[Apex Debug]", msg);
      },
    };
  }

  // ===========================================================================
  // Internal — evaluate transpiled JavaScript
  // ===========================================================================

  async _evaluateTranspiledCode(jsCode, $runtime) {
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

    const shimCode = _getApexShims();
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
}

// =============================================================================
// Helpers (module-level)
// =============================================================================

function _mapReplacer(key, value) {
  if (value instanceof Map) {
    const obj = {};
    for (const [k, v] of value) {
      obj[k] = v;
    }
    return obj;
  }
  return value;
}

function _getApexShims() {
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
      static getOrgDomainUrl() { return new Url(location.origin); }
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

customElements.define("apex-runtime", ApexRuntime);
