/**
 * Web Worker for executing Apex code with SOQL support
 *
 * This worker provides an isolated environment for:
 * 1. Parsing Apex code (via WASM)
 * 2. Converting SOQL to SQL (via WASM)
 * 3. Executing SQL against an in-memory SQLite database (via sql.js)
 *
 * Usage:
 *   const worker = new Worker('apex-worker.js', { type: 'module' });
 *   worker.postMessage({ type: 'init' });
 *   worker.postMessage({ type: 'execute', apex: '...', schema: {...} });
 */

import init, {
  parseApex,
  convertSoqlToSql,
  generateDdl,
  WasmSchema,
  type SObjectDefinition,
  type ConversionResult,
} from "../../pkg/apexrust.js";

// Types for sql.js (would come from @types/sql.js in a real project)
declare const initSqlJs: () => Promise<SqlJsStatic>;
interface SqlJsStatic {
  Database: new () => Database;
}
interface Database {
  run(sql: string, params?: unknown[]): void;
  exec(sql: string, params?: unknown[]): QueryExecResult[];
  close(): void;
}
interface QueryExecResult {
  columns: string[];
  values: unknown[][];
}

// ============================================================================
// Message Types
// ============================================================================

interface InitMessage {
  type: "init";
}

interface ExecuteApexMessage {
  type: "execute";
  apex: string;
  schema: { objects: SObjectDefinition[] };
  records?: Record<string, Record<string, unknown>[]>;
}

interface ParseMessage {
  type: "parse";
  apex: string;
}

interface ConvertSoqlMessage {
  type: "convert-soql";
  soql: string;
  schema: { objects: SObjectDefinition[] };
  dialect: "sqlite" | "postgres";
}

interface ExecuteSqlMessage {
  type: "execute-sql";
  sql: string;
  params?: unknown[];
}

type WorkerMessage =
  | InitMessage
  | ExecuteApexMessage
  | ParseMessage
  | ConvertSoqlMessage
  | ExecuteSqlMessage;

interface WorkerResponse {
  type: "result" | "error" | "ready";
  data?: unknown;
  error?: string;
}

// ============================================================================
// Worker State
// ============================================================================

let db: Database | null = null;
let isInitialized = false;

// ============================================================================
// Message Handler
// ============================================================================

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;

  try {
    switch (message.type) {
      case "init":
        await handleInit();
        break;

      case "parse":
        handleParse(message.apex);
        break;

      case "convert-soql":
        handleConvertSoql(message.soql, message.schema, message.dialect);
        break;

      case "execute-sql":
        handleExecuteSql(message.sql, message.params);
        break;

      case "execute":
        await handleExecuteApex(message.apex, message.schema, message.records);
        break;

      default:
        throw new Error(`Unknown message type: ${(message as { type: string }).type}`);
    }
  } catch (error) {
    respond({
      type: "error",
      error: error instanceof Error ? error.message : String(error),
    });
  }
};

// ============================================================================
// Message Handlers
// ============================================================================

async function handleInit(): Promise<void> {
  if (isInitialized) {
    respond({ type: "ready" });
    return;
  }

  // Initialize apexrust WASM
  await init();

  // Initialize sql.js
  const SQL = await initSqlJs();
  db = new SQL.Database();

  isInitialized = true;
  respond({ type: "ready" });
}

function handleParse(apex: string): void {
  ensureInitialized();

  const result = parseApex(apex);
  respond({ type: "result", data: result });
}

function handleConvertSoql(
  soql: string,
  schemaJson: { objects: SObjectDefinition[] },
  dialect: "sqlite" | "postgres"
): void {
  ensureInitialized();

  const schema = new WasmSchema();
  schema.loadFromJson(schemaJson);

  const result = convertSoqlToSql(soql, schema, dialect);
  respond({ type: "result", data: result });
}

function handleExecuteSql(sql: string, params?: unknown[]): void {
  ensureInitialized();

  if (!db) {
    throw new Error("Database not initialized");
  }

  const results = db.exec(sql, params);
  respond({ type: "result", data: results });
}

async function handleExecuteApex(
  apex: string,
  schemaJson: { objects: SObjectDefinition[] },
  records?: Record<string, Record<string, unknown>[]>
): Promise<void> {
  ensureInitialized();

  if (!db) {
    throw new Error("Database not initialized");
  }

  // 1. Parse the Apex code
  const parseResult = parseApex(apex);
  if (!parseResult.success) {
    throw new Error(`Parse error: ${parseResult.error}`);
  }

  // 2. Set up the schema
  const schema = new WasmSchema();
  schema.loadFromJson(schemaJson);

  // 3. Generate and execute DDL
  const ddlResult = generateDdl(schema, "sqlite");
  if (!ddlResult.success) {
    throw new Error(`DDL error: ${ddlResult.error}`);
  }

  // Drop existing tables and recreate
  for (const objName of schema.getObjectNames()) {
    try {
      db.run(`DROP TABLE IF EXISTS "${toSnakeCase(objName)}"`);
    } catch {
      // Ignore errors
    }
  }

  // Execute DDL statements
  const ddlStatements = ddlResult.ddl!.split(";").filter((s) => s.trim());
  for (const stmt of ddlStatements) {
    if (stmt.trim()) {
      db.run(stmt);
    }
  }

  // 4. Insert test records if provided
  if (records) {
    for (const [objectName, rows] of Object.entries(records)) {
      const tableName = toSnakeCase(objectName);
      for (const row of rows) {
        const columns = Object.keys(row).map(toSnakeCase);
        const placeholders = columns.map(() => "?").join(", ");
        const values = Object.values(row);

        const sql = `INSERT INTO "${tableName}" (${columns.map((c) => `"${c}"`).join(", ")}) VALUES (${placeholders})`;
        db.run(sql, values);
      }
    }
  }

  // 5. Extract and convert SOQL queries
  const soqlQueries = parseResult.soqlQueries || [];
  const sqlResults: { soql: string; sql: string; result: QueryExecResult[] }[] = [];

  for (const soqlDebug of soqlQueries) {
    // Note: In a real implementation, we'd extract the actual SOQL string
    // For now, this demonstrates the flow
    // The SOQL would need to be reconstructed from the AST or stored separately

    // This is a simplified example - in practice you'd need to:
    // 1. Store the original SOQL strings during parsing
    // 2. Or reconstruct them from the AST
    console.log("Found SOQL query:", soqlDebug);
  }

  respond({
    type: "result",
    data: {
      parsed: true,
      soqlCount: soqlQueries.length,
      sqlResults,
    },
  });
}

// ============================================================================
// Utilities
// ============================================================================

function ensureInitialized(): void {
  if (!isInitialized) {
    throw new Error("Worker not initialized. Send { type: 'init' } first.");
  }
}

function respond(response: WorkerResponse): void {
  self.postMessage(response);
}

function toSnakeCase(s: string): string {
  return s
    .replace(/([A-Z])/g, "_$1")
    .toLowerCase()
    .replace(/^_/, "")
    .replace(/__/g, "_");
}

// ============================================================================
// Export for type checking (not used at runtime)
// ============================================================================

export type { WorkerMessage, WorkerResponse };
