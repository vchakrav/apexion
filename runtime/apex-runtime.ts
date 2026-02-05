/**
 * ApexRuntime - Universal JavaScript runtime for transpiled Apex code
 *
 * Works in:
 * - Browser (via Service Worker or main thread)
 * - Node.js
 * - Deno
 * - Edge functions (Cloudflare Workers, etc.)
 *
 * Requires a database adapter to be provided for SOQL/DML operations.
 */

// ============================================================================
// Type Definitions
// ============================================================================

export interface SObject {
  Id?: string;
  [key: string]: any;
}

export interface QueryResult<T> {
  records: T[];
  totalSize: number;
  done: boolean;
}

export interface SaveResult {
  id: string;
  success: boolean;
  errors: SaveError[];
}

export interface SaveError {
  message: string;
  statusCode: string;
  fields: string[];
}

export interface DeleteResult {
  id: string;
  success: boolean;
  errors: SaveError[];
}

export interface DatabaseAdapter {
  /**
   * Execute a SOQL query and return results
   */
  query<T extends SObject>(soql: string, binds?: Record<string, any>): Promise<T[]>;

  /**
   * Insert one or more records
   */
  insert(sobjectType: string, records: SObject[]): Promise<SaveResult[]>;

  /**
   * Update one or more records
   */
  update(sobjectType: string, records: SObject[]): Promise<SaveResult[]>;

  /**
   * Upsert one or more records
   */
  upsert(sobjectType: string, records: SObject[], externalIdField?: string): Promise<SaveResult[]>;

  /**
   * Delete one or more records by ID
   */
  delete(sobjectType: string, ids: string[]): Promise<DeleteResult[]>;

  /**
   * Undelete one or more records by ID
   */
  undelete(sobjectType: string, ids: string[]): Promise<SaveResult[]>;
}

// ============================================================================
// ApexRuntime - The main runtime class injected into transpiled code
// ============================================================================

export class ApexRuntime {
  private db: DatabaseAdapter;

  constructor(db: DatabaseAdapter) {
    this.db = db;
  }

  /**
   * Execute a SOQL query
   */
  async query<T extends SObject>(soql: string, binds?: Record<string, any>): Promise<T[]> {
    return this.db.query<T>(soql, binds);
  }

  /**
   * Insert records - handles both single record and array
   */
  async insert(records: SObject | SObject[]): Promise<string[]> {
    const recordArray = Array.isArray(records) ? records : [records];
    if (recordArray.length === 0) return [];

    // Infer SObject type from the records (would need metadata in real impl)
    const sobjectType = this.inferSObjectType(recordArray[0]);
    const results = await this.db.insert(sobjectType, recordArray);

    // Check for errors
    const errors = results.filter(r => !r.success);
    if (errors.length > 0) {
      const errorMsg = errors.map(e => e.errors.map(err => err.message).join(', ')).join('; ');
      throw new DmlException(`Insert failed: ${errorMsg}`);
    }

    // Return IDs and update records with their new IDs
    return results.map((r, i) => {
      recordArray[i].Id = r.id;
      return r.id;
    });
  }

  /**
   * Update records
   */
  async update(records: SObject | SObject[]): Promise<void> {
    const recordArray = Array.isArray(records) ? records : [records];
    if (recordArray.length === 0) return;

    const sobjectType = this.inferSObjectType(recordArray[0]);
    const results = await this.db.update(sobjectType, recordArray);

    const errors = results.filter(r => !r.success);
    if (errors.length > 0) {
      const errorMsg = errors.map(e => e.errors.map(err => err.message).join(', ')).join('; ');
      throw new DmlException(`Update failed: ${errorMsg}`);
    }
  }

  /**
   * Upsert records
   */
  async upsert(records: SObject | SObject[], externalIdField?: string): Promise<void> {
    const recordArray = Array.isArray(records) ? records : [records];
    if (recordArray.length === 0) return;

    const sobjectType = this.inferSObjectType(recordArray[0]);
    const results = await this.db.upsert(sobjectType, recordArray, externalIdField);

    const errors = results.filter(r => !r.success);
    if (errors.length > 0) {
      const errorMsg = errors.map(e => e.errors.map(err => err.message).join(', ')).join('; ');
      throw new DmlException(`Upsert failed: ${errorMsg}`);
    }

    // Update records with their IDs
    results.forEach((r, i) => {
      if (r.id) recordArray[i].Id = r.id;
    });
  }

  /**
   * Delete records
   */
  async delete(records: SObject | SObject[] | string | string[]): Promise<void> {
    let ids: string[];
    let sobjectType: string | undefined;

    if (typeof records === 'string') {
      ids = [records];
    } else if (Array.isArray(records)) {
      if (records.length === 0) return;
      if (typeof records[0] === 'string') {
        ids = records as string[];
      } else {
        const sobjects = records as SObject[];
        ids = sobjects.map(r => r.Id!).filter(Boolean);
        sobjectType = this.inferSObjectType(sobjects[0]);
      }
    } else {
      ids = [records.Id!];
      sobjectType = this.inferSObjectType(records);
    }

    if (ids.length === 0) return;

    // Default to generic SObject if type not determined
    sobjectType = sobjectType || 'SObject';
    const results = await this.db.delete(sobjectType, ids);

    const errors = results.filter(r => !r.success);
    if (errors.length > 0) {
      const errorMsg = errors.map(e => e.errors.map(err => err.message).join(', ')).join('; ');
      throw new DmlException(`Delete failed: ${errorMsg}`);
    }
  }

  /**
   * Undelete records
   */
  async undelete(records: SObject | SObject[] | string | string[]): Promise<void> {
    let ids: string[];
    let sobjectType: string | undefined;

    if (typeof records === 'string') {
      ids = [records];
    } else if (Array.isArray(records)) {
      if (records.length === 0) return;
      if (typeof records[0] === 'string') {
        ids = records as string[];
      } else {
        const sobjects = records as SObject[];
        ids = sobjects.map(r => r.Id!).filter(Boolean);
        sobjectType = this.inferSObjectType(sobjects[0]);
      }
    } else {
      ids = [records.Id!];
      sobjectType = this.inferSObjectType(records);
    }

    if (ids.length === 0) return;

    sobjectType = sobjectType || 'SObject';
    const results = await this.db.undelete(sobjectType, ids);

    const errors = results.filter(r => !r.success);
    if (errors.length > 0) {
      const errorMsg = errors.map(e => e.errors.map(err => err.message).join(', ')).join('; ');
      throw new DmlException(`Undelete failed: ${errorMsg}`);
    }
  }

  /**
   * Infer SObject type from a record
   * In a real implementation, this would use metadata or a type registry
   */
  private inferSObjectType(record: SObject): string {
    // Check for explicit type annotation
    if (record.attributes?.type) {
      return record.attributes.type;
    }

    // Check for common patterns in ID prefix
    if (record.Id) {
      const prefix = record.Id.substring(0, 3);
      const typeMap: Record<string, string> = {
        '001': 'Account',
        '003': 'Contact',
        '006': 'Opportunity',
        '00Q': 'Lead',
        '500': 'Case',
        '00T': 'Task',
        '00U': 'Event',
      };
      if (typeMap[prefix]) {
        return typeMap[prefix];
      }
    }

    return 'SObject';
  }
}

// ============================================================================
// Apex Standard Exceptions
// ============================================================================

export class ApexException extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ApexException';
  }
}

export class DmlException extends ApexException {
  constructor(message: string) {
    super(message);
    this.name = 'DmlException';
  }
}

export class QueryException extends ApexException {
  constructor(message: string) {
    super(message);
    this.name = 'QueryException';
  }
}

export class NullPointerException extends ApexException {
  constructor(message: string = 'Attempt to de-reference a null object') {
    super(message);
    this.name = 'NullPointerException';
  }
}

export class ListException extends ApexException {
  constructor(message: string) {
    super(message);
    this.name = 'ListException';
  }
}

// ============================================================================
// SQLite Database Adapter (for browser/local use)
// ============================================================================

export interface SQLiteDatabase {
  exec(sql: string): { columns: string[]; values: any[][] }[];
  run(sql: string, params?: any[]): void;
}

/**
 * SQLite adapter that converts SOQL to SQL and executes against a SQLite database
 */
export class SQLiteDatabaseAdapter implements DatabaseAdapter {
  private db: SQLiteDatabase;
  private soqlToSql: (soql: string) => { sql: string; parameters: any[] };

  constructor(
    db: SQLiteDatabase,
    soqlToSql: (soql: string) => { sql: string; parameters: any[] }
  ) {
    this.db = db;
    this.soqlToSql = soqlToSql;
  }

  async query<T extends SObject>(soql: string, binds?: Record<string, any>): Promise<T[]> {
    // Convert SOQL to SQL
    const { sql, parameters } = this.soqlToSql(soql);

    // Substitute bind variables
    let finalSql = sql;
    if (binds) {
      for (const [name, value] of Object.entries(binds)) {
        // Replace :name with the actual value (properly escaped)
        const placeholder = new RegExp(`\\?\\d+`, 'g'); // SQLite uses ?1, ?2, etc.
        // This is simplified - real impl would track parameter positions
      }
    }

    // Execute query
    const results = this.db.exec(finalSql);

    if (results.length === 0) {
      return [];
    }

    const { columns, values } = results[0];

    // Convert to SObject array
    return values.map(row => {
      const obj: any = {};
      columns.forEach((col, i) => {
        // Convert snake_case back to camelCase/PascalCase
        const fieldName = this.snakeToCamel(col);
        obj[fieldName] = row[i];
      });
      return obj as T;
    });
  }

  async insert(sobjectType: string, records: SObject[]): Promise<SaveResult[]> {
    const results: SaveResult[] = [];
    const tableName = this.camelToSnake(sobjectType);

    for (const record of records) {
      try {
        // Generate ID if not provided
        const id = record.Id || this.generateId(sobjectType);
        record.Id = id;

        const fields = Object.keys(record).filter(k => k !== 'attributes');
        const columns = fields.map(f => this.camelToSnake(f));
        const values = fields.map(f => record[f]);
        const placeholders = fields.map(() => '?').join(', ');

        const sql = `INSERT INTO "${tableName}" (${columns.map(c => `"${c}"`).join(', ')}) VALUES (${placeholders})`;
        this.db.run(sql, values);

        results.push({ id, success: true, errors: [] });
      } catch (e: any) {
        results.push({
          id: '',
          success: false,
          errors: [{ message: e.message, statusCode: 'UNKNOWN_ERROR', fields: [] }]
        });
      }
    }

    return results;
  }

  async update(sobjectType: string, records: SObject[]): Promise<SaveResult[]> {
    const results: SaveResult[] = [];
    const tableName = this.camelToSnake(sobjectType);

    for (const record of records) {
      try {
        if (!record.Id) {
          throw new Error('Id is required for update');
        }

        const fields = Object.keys(record).filter(k => k !== 'Id' && k !== 'attributes');
        const setClauses = fields.map(f => `"${this.camelToSnake(f)}" = ?`).join(', ');
        const values = [...fields.map(f => record[f]), record.Id];

        const sql = `UPDATE "${tableName}" SET ${setClauses} WHERE "id" = ?`;
        this.db.run(sql, values);

        results.push({ id: record.Id, success: true, errors: [] });
      } catch (e: any) {
        results.push({
          id: record.Id || '',
          success: false,
          errors: [{ message: e.message, statusCode: 'UNKNOWN_ERROR', fields: [] }]
        });
      }
    }

    return results;
  }

  async upsert(sobjectType: string, records: SObject[], externalIdField?: string): Promise<SaveResult[]> {
    // Simplified upsert - just check if ID exists
    const results: SaveResult[] = [];

    for (const record of records) {
      if (record.Id) {
        // Try update first
        const updateResults = await this.update(sobjectType, [record]);
        results.push(...updateResults);
      } else {
        // Insert new
        const insertResults = await this.insert(sobjectType, [record]);
        results.push(...insertResults);
      }
    }

    return results;
  }

  async delete(sobjectType: string, ids: string[]): Promise<DeleteResult[]> {
    const results: DeleteResult[] = [];
    const tableName = this.camelToSnake(sobjectType);

    for (const id of ids) {
      try {
        const sql = `DELETE FROM "${tableName}" WHERE "id" = ?`;
        this.db.run(sql, [id]);
        results.push({ id, success: true, errors: [] });
      } catch (e: any) {
        results.push({
          id,
          success: false,
          errors: [{ message: e.message, statusCode: 'UNKNOWN_ERROR', fields: [] }]
        });
      }
    }

    return results;
  }

  async undelete(sobjectType: string, ids: string[]): Promise<SaveResult[]> {
    // SQLite doesn't have soft delete by default
    // This would need to be implemented based on schema design
    return ids.map(id => ({
      id,
      success: false,
      errors: [{ message: 'Undelete not supported in SQLite adapter', statusCode: 'NOT_SUPPORTED', fields: [] }]
    }));
  }

  private generateId(sobjectType: string): string {
    const prefixMap: Record<string, string> = {
      'Account': '001',
      'Contact': '003',
      'Opportunity': '006',
      'Lead': '00Q',
      'Case': '500',
      'Task': '00T',
      'Event': '00U',
    };
    const prefix = prefixMap[sobjectType] || '000';
    const random = Math.random().toString(36).substring(2, 15).padEnd(15, '0');
    return prefix + random;
  }

  private camelToSnake(str: string): string {
    return str.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
  }

  private snakeToCamel(str: string): string {
    return str.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
  }
}

// ============================================================================
// Factory function for creating runtime
// ============================================================================

/**
 * Create an ApexRuntime with the specified database adapter
 */
export function createRuntime(db: DatabaseAdapter): ApexRuntime {
  return new ApexRuntime(db);
}

/**
 * Create an ApexRuntime with SQLite backend
 */
export function createSQLiteRuntime(
  db: SQLiteDatabase,
  soqlToSql: (soql: string) => { sql: string; parameters: any[] }
): ApexRuntime {
  const adapter = new SQLiteDatabaseAdapter(db, soqlToSql);
  return new ApexRuntime(adapter);
}

// Default export for convenience
export default ApexRuntime;
