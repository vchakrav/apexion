/**
 * Example: Using the Apex Web Worker
 *
 * This demonstrates how to use the apex-worker from the main thread.
 */

import type { WorkerResponse, SObjectDefinition } from "./apex-worker.js";

// ============================================================================
// Worker Management
// ============================================================================

class ApexWorker {
  private worker: Worker;
  private pendingRequests: Map<
    number,
    { resolve: (value: unknown) => void; reject: (error: Error) => void }
  > = new Map();
  private requestId = 0;
  private ready: Promise<void>;

  constructor() {
    this.worker = new Worker(new URL("./apex-worker.js", import.meta.url), {
      type: "module",
    });

    this.worker.onmessage = this.handleMessage.bind(this);
    this.worker.onerror = this.handleError.bind(this);

    // Initialize and wait for ready
    this.ready = this.init();
  }

  private async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const handler = (event: MessageEvent<WorkerResponse>) => {
        if (event.data.type === "ready") {
          this.worker.removeEventListener("message", handler);
          resolve();
        } else if (event.data.type === "error") {
          this.worker.removeEventListener("message", handler);
          reject(new Error(event.data.error));
        }
      };

      this.worker.addEventListener("message", handler);
      this.worker.postMessage({ type: "init" });
    });
  }

  private handleMessage(event: MessageEvent<WorkerResponse>): void {
    const response = event.data;

    // Handle responses to pending requests
    const pending = this.pendingRequests.get(this.requestId - 1);
    if (pending) {
      this.pendingRequests.delete(this.requestId - 1);

      if (response.type === "error") {
        pending.reject(new Error(response.error));
      } else {
        pending.resolve(response.data);
      }
    }
  }

  private handleError(error: ErrorEvent): void {
    console.error("Worker error:", error);
  }

  private async send(message: object): Promise<unknown> {
    await this.ready;

    return new Promise((resolve, reject) => {
      const id = this.requestId++;
      this.pendingRequests.set(id, { resolve, reject });
      this.worker.postMessage(message);
    });
  }

  /**
   * Parse Apex source code
   */
  async parseApex(apex: string): Promise<{
    success: boolean;
    ast?: string;
    soqlQueries?: string[];
    error?: string;
  }> {
    return (await this.send({ type: "parse", apex })) as {
      success: boolean;
      ast?: string;
      soqlQueries?: string[];
      error?: string;
    };
  }

  /**
   * Convert SOQL to SQL
   */
  async convertSoql(
    soql: string,
    schema: { objects: SObjectDefinition[] },
    dialect: "sqlite" | "postgres" = "sqlite"
  ): Promise<{
    success: boolean;
    sql?: string;
    parameters?: { name: string; placeholder: string; originalName: string }[];
    warnings?: string[];
    error?: string;
  }> {
    return (await this.send({
      type: "convert-soql",
      soql,
      schema,
      dialect,
    })) as {
      success: boolean;
      sql?: string;
      parameters?: { name: string; placeholder: string; originalName: string }[];
      warnings?: string[];
      error?: string;
    };
  }

  /**
   * Execute raw SQL against the in-memory database
   */
  async executeSql(
    sql: string,
    params?: unknown[]
  ): Promise<{ columns: string[]; values: unknown[][] }[]> {
    return (await this.send({ type: "execute-sql", sql, params })) as {
      columns: string[];
      values: unknown[][];
    }[];
  }

  /**
   * Terminate the worker
   */
  terminate(): void {
    this.worker.terminate();
  }
}

// ============================================================================
// Example Usage
// ============================================================================

async function main() {
  console.log("Starting Apex Worker example...\n");

  const worker = new ApexWorker();

  // Example 1: Parse Apex code
  console.log("=== Example 1: Parse Apex ===");
  const apexCode = `
    public class AccountService {
      public List<Account> getActiveAccounts() {
        return [SELECT Id, Name, Industry FROM Account WHERE IsDeleted = false];
      }

      public Account getAccountById(Id accountId) {
        return [SELECT Id, Name, (SELECT Id, FirstName, LastName FROM Contacts)
                FROM Account WHERE Id = :accountId];
      }
    }
  `;

  const parseResult = await worker.parseApex(apexCode);
  console.log("Parse success:", parseResult.success);
  console.log("SOQL queries found:", parseResult.soqlQueries?.length || 0);
  console.log();

  // Example 2: Convert SOQL to SQL
  console.log("=== Example 2: Convert SOQL to SQL ===");
  const schema = {
    objects: [
      {
        name: "Account",
        fields: [
          { name: "Id", type: "Id" as const },
          { name: "Name", type: "String" as const },
          { name: "Industry", type: "Picklist" as const },
          { name: "IsDeleted", type: "Boolean" as const },
        ],
        childRelationships: [
          { name: "Contacts", childObject: "Contact", field: "AccountId" },
        ],
      },
      {
        name: "Contact",
        fields: [
          { name: "Id", type: "Id" as const },
          { name: "FirstName", type: "String" as const },
          { name: "LastName", type: "String" as const },
          {
            name: "AccountId",
            type: "Lookup" as const,
            referenceTo: "Account",
            relationshipName: "Account",
          },
        ],
      },
    ],
  };

  const soql = "SELECT Id, Name, Industry FROM Account WHERE Industry = 'Technology'";
  const conversionResult = await worker.convertSoql(soql, schema, "sqlite");

  console.log("Conversion success:", conversionResult.success);
  if (conversionResult.success) {
    console.log("SQL:", conversionResult.sql);
    console.log("Warnings:", conversionResult.warnings);
  } else {
    console.log("Error:", conversionResult.error);
  }
  console.log();

  // Example 3: SOQL with bind variable
  console.log("=== Example 3: SOQL with bind variable ===");
  const soqlWithBind = "SELECT Id, Name FROM Account WHERE Id = :accountId";
  const bindResult = await worker.convertSoql(soqlWithBind, schema, "postgres");

  console.log("Conversion success:", bindResult.success);
  if (bindResult.success) {
    console.log("SQL:", bindResult.sql);
    console.log("Parameters:", bindResult.parameters);
  }
  console.log();

  // Example 4: Parent relationship query
  console.log("=== Example 4: Parent relationship ===");
  const parentSoql = "SELECT Id, FirstName, Account.Name FROM Contact";
  const parentResult = await worker.convertSoql(parentSoql, schema, "sqlite");

  console.log("Conversion success:", parentResult.success);
  if (parentResult.success) {
    console.log("SQL:", parentResult.sql);
  }
  console.log();

  // Cleanup
  worker.terminate();
  console.log("Worker terminated.");
}

// Run if this is the main module
main().catch(console.error);
