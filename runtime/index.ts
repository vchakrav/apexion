/**
 * ApexRust Runtime - Universal JavaScript runtime for transpiled Apex code
 *
 * This package provides everything needed to run transpiled Apex code:
 * - ApexRuntime: The main runtime class that handles SOQL queries and DML operations
 * - DatabaseAdapter: Interface for implementing custom database backends
 * - SQLiteDatabaseAdapter: Ready-to-use SQLite adapter for browser/local use
 * - Apex Standard Library: JavaScript implementations of Apex standard classes
 *
 * Usage:
 * ```typescript
 * import { ApexRuntime, SQLiteDatabaseAdapter, ApexList, System } from '@apexrust/runtime';
 *
 * // Create runtime with SQLite backend
 * const runtime = createSQLiteRuntime(sqliteDb, soqlToSqlConverter);
 *
 * // Transpiled Apex code uses this runtime
 * const $runtime = runtime;
 *
 * // Now transpiled code can run:
 * // const accounts = await $runtime.query('SELECT Id, Name FROM Account');
 * ```
 */

// Core runtime
export {
  ApexRuntime,
  createRuntime,
  createSQLiteRuntime,
  SQLiteDatabaseAdapter,
  type DatabaseAdapter,
  type SObject,
  type QueryResult,
  type SaveResult,
  type SaveError,
  type DeleteResult,
  type SQLiteDatabase,
} from './apex-runtime';

// Exceptions
export {
  ApexException,
  DmlException,
  QueryException,
  NullPointerException,
  ListException as ListIndexException,
} from './apex-runtime';

// Standard library
export {
  System,
  LoggingLevel,
  AssertException,
  ApexString,
  ApexList,
  ApexSet,
  ApexMap,
  ApexDate,
  ApexDateTime,
  ApexMath,
  ApexJSON,
  ListException,
} from './apex-stdlib';

// Convenience re-exports with Apex-style names
export {
  ApexList as List,
  ApexSet as Set_,  // 'Set' conflicts with built-in
  ApexMap as Map_,  // 'Map' conflicts with built-in
  ApexDate as Date_,
  ApexDateTime as DateTime,
  ApexMath as Math_,
  ApexJSON as JSON_,
} from './apex-stdlib';

// Default export
export { ApexRuntime as default } from './apex-runtime';
