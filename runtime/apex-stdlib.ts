/**
 * Apex Standard Library Shims
 *
 * JavaScript implementations of common Apex standard library classes and methods.
 * These provide compatibility for transpiled Apex code running in JavaScript.
 */

// ============================================================================
// System Namespace
// ============================================================================

export namespace System {
  /**
   * Debug logging - outputs to console
   */
  export function debug(message: any): void {
    console.log('[DEBUG]', message);
  }

  export function debug(level: LoggingLevel, message: any): void {
    const prefix = `[${LoggingLevel[level]}]`;
    switch (level) {
      case LoggingLevel.ERROR:
        console.error(prefix, message);
        break;
      case LoggingLevel.WARN:
        console.warn(prefix, message);
        break;
      default:
        console.log(prefix, message);
    }
  }

  /**
   * Assert - throws if condition is false
   */
  export function assert(condition: boolean, message?: string): void {
    if (!condition) {
      throw new AssertException(message || 'Assertion failed');
    }
  }

  export function assertEquals(expected: any, actual: any, message?: string): void {
    if (expected !== actual) {
      throw new AssertException(
        message || `Expected: ${expected}, Actual: ${actual}`
      );
    }
  }

  export function assertNotEquals(expected: any, actual: any, message?: string): void {
    if (expected === actual) {
      throw new AssertException(
        message || `Expected values to be different but both were: ${expected}`
      );
    }
  }

  /**
   * Get current time in milliseconds
   */
  export function currentTimeMillis(): number {
    return Date.now();
  }

  /**
   * Get current date/time
   */
  export function now(): Date {
    return new Date();
  }

  /**
   * Get today's date (without time)
   */
  export function today(): Date {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    return d;
  }
}

export enum LoggingLevel {
  NONE = 0,
  ERROR = 1,
  WARN = 2,
  INFO = 3,
  DEBUG = 4,
  FINE = 5,
  FINER = 6,
  FINEST = 7,
}

export class AssertException extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AssertException';
  }
}

// ============================================================================
// String Extensions
// ============================================================================

/**
 * Apex String class methods
 * In transpiled code, these are called as static methods or instance methods
 */
export class ApexString {
  private value: string;

  constructor(value: string = '') {
    this.value = value;
  }

  // Static methods
  static isBlank(s: string | null | undefined): boolean {
    return s == null || s.trim().length === 0;
  }

  static isNotBlank(s: string | null | undefined): boolean {
    return !ApexString.isBlank(s);
  }

  static isEmpty(s: string | null | undefined): boolean {
    return s == null || s.length === 0;
  }

  static isNotEmpty(s: string | null | undefined): boolean {
    return !ApexString.isEmpty(s);
  }

  static valueOf(obj: any): string {
    if (obj == null) return 'null';
    return String(obj);
  }

  static join(values: Iterable<any>, separator: string): string {
    return Array.from(values).join(separator);
  }

  static format(template: string, ...args: any[]): string {
    return template.replace(/\{(\d+)\}/g, (match, index) => {
      const i = parseInt(index, 10);
      return i < args.length ? String(args[i]) : match;
    });
  }

  static escapeSingleQuotes(s: string): string {
    return s.replace(/'/g, "\\'");
  }

  // Instance methods (called on string values)
  abbreviate(maxWidth: number): string {
    if (this.value.length <= maxWidth) return this.value;
    return this.value.substring(0, maxWidth - 3) + '...';
  }

  capitalize(): string {
    if (this.value.length === 0) return '';
    return this.value.charAt(0).toUpperCase() + this.value.slice(1);
  }

  center(size: number, padStr: string = ' '): string {
    if (this.value.length >= size) return this.value;
    const padding = size - this.value.length;
    const left = Math.floor(padding / 2);
    const right = padding - left;
    return padStr.repeat(left) + this.value + padStr.repeat(right);
  }

  contains(substring: string): boolean {
    return this.value.includes(substring);
  }

  containsIgnoreCase(substring: string): boolean {
    return this.value.toLowerCase().includes(substring.toLowerCase());
  }

  containsOnly(chars: string): boolean {
    const charSet = new Set(chars.split(''));
    return this.value.split('').every(c => charSet.has(c));
  }

  containsWhitespace(): boolean {
    return /\s/.test(this.value);
  }

  countMatches(substring: string): number {
    if (substring.length === 0) return 0;
    let count = 0;
    let pos = 0;
    while ((pos = this.value.indexOf(substring, pos)) !== -1) {
      count++;
      pos += substring.length;
    }
    return count;
  }

  deleteWhitespace(): string {
    return this.value.replace(/\s/g, '');
  }

  endsWith(suffix: string): boolean {
    return this.value.endsWith(suffix);
  }

  endsWithIgnoreCase(suffix: string): boolean {
    return this.value.toLowerCase().endsWith(suffix.toLowerCase());
  }

  equals(other: string): boolean {
    return this.value === other;
  }

  equalsIgnoreCase(other: string): boolean {
    return this.value.toLowerCase() === other.toLowerCase();
  }

  indexOf(substring: string, startIndex: number = 0): number {
    return this.value.indexOf(substring, startIndex);
  }

  indexOfIgnoreCase(substring: string, startIndex: number = 0): number {
    return this.value.toLowerCase().indexOf(substring.toLowerCase(), startIndex);
  }

  isAllLowerCase(): boolean {
    return this.value === this.value.toLowerCase() && /[a-z]/.test(this.value);
  }

  isAllUpperCase(): boolean {
    return this.value === this.value.toUpperCase() && /[A-Z]/.test(this.value);
  }

  isAlpha(): boolean {
    return /^[a-zA-Z]+$/.test(this.value);
  }

  isAlphanumeric(): boolean {
    return /^[a-zA-Z0-9]+$/.test(this.value);
  }

  isNumeric(): boolean {
    return /^[0-9]+$/.test(this.value);
  }

  lastIndexOf(substring: string): number {
    return this.value.lastIndexOf(substring);
  }

  left(len: number): string {
    return this.value.substring(0, len);
  }

  leftPad(len: number, padStr: string = ' '): string {
    if (this.value.length >= len) return this.value;
    return padStr.repeat(len - this.value.length) + this.value;
  }

  length(): number {
    return this.value.length;
  }

  mid(startIndex: number, len: number): string {
    return this.value.substring(startIndex, startIndex + len);
  }

  normalizeSpace(): string {
    return this.value.trim().replace(/\s+/g, ' ');
  }

  remove(substring: string): string {
    return this.value.split(substring).join('');
  }

  removeEnd(suffix: string): string {
    if (this.value.endsWith(suffix)) {
      return this.value.slice(0, -suffix.length);
    }
    return this.value;
  }

  removeEndIgnoreCase(suffix: string): string {
    if (this.value.toLowerCase().endsWith(suffix.toLowerCase())) {
      return this.value.slice(0, -suffix.length);
    }
    return this.value;
  }

  removeStart(prefix: string): string {
    if (this.value.startsWith(prefix)) {
      return this.value.slice(prefix.length);
    }
    return this.value;
  }

  repeat(times: number): string {
    return this.value.repeat(times);
  }

  replace(target: string, replacement: string): string {
    return this.value.split(target).join(replacement);
  }

  replaceAll(regex: string, replacement: string): string {
    return this.value.replace(new RegExp(regex, 'g'), replacement);
  }

  replaceFirst(regex: string, replacement: string): string {
    return this.value.replace(new RegExp(regex), replacement);
  }

  reverse(): string {
    return this.value.split('').reverse().join('');
  }

  right(len: number): string {
    return this.value.slice(-len);
  }

  rightPad(len: number, padStr: string = ' '): string {
    if (this.value.length >= len) return this.value;
    return this.value + padStr.repeat(len - this.value.length);
  }

  split(regex: string): string[] {
    return this.value.split(new RegExp(regex));
  }

  splitByCharacterType(): string[] {
    const result: string[] = [];
    let current = '';
    let lastType: 'upper' | 'lower' | 'digit' | 'other' | null = null;

    for (const char of this.value) {
      let type: 'upper' | 'lower' | 'digit' | 'other';
      if (/[A-Z]/.test(char)) type = 'upper';
      else if (/[a-z]/.test(char)) type = 'lower';
      else if (/[0-9]/.test(char)) type = 'digit';
      else type = 'other';

      if (lastType !== null && type !== lastType) {
        result.push(current);
        current = '';
      }
      current += char;
      lastType = type;
    }

    if (current) result.push(current);
    return result;
  }

  startsWith(prefix: string): boolean {
    return this.value.startsWith(prefix);
  }

  startsWithIgnoreCase(prefix: string): boolean {
    return this.value.toLowerCase().startsWith(prefix.toLowerCase());
  }

  substring(startIndex: number, endIndex?: number): string {
    return this.value.substring(startIndex, endIndex);
  }

  substringAfter(separator: string): string {
    const index = this.value.indexOf(separator);
    return index === -1 ? '' : this.value.slice(index + separator.length);
  }

  substringAfterLast(separator: string): string {
    const index = this.value.lastIndexOf(separator);
    return index === -1 ? '' : this.value.slice(index + separator.length);
  }

  substringBefore(separator: string): string {
    const index = this.value.indexOf(separator);
    return index === -1 ? this.value : this.value.slice(0, index);
  }

  substringBeforeLast(separator: string): string {
    const index = this.value.lastIndexOf(separator);
    return index === -1 ? this.value : this.value.slice(0, index);
  }

  substringBetween(open: string, close: string): string | null {
    const startIndex = this.value.indexOf(open);
    if (startIndex === -1) return null;
    const endIndex = this.value.indexOf(close, startIndex + open.length);
    if (endIndex === -1) return null;
    return this.value.slice(startIndex + open.length, endIndex);
  }

  toLowerCase(): string {
    return this.value.toLowerCase();
  }

  toUpperCase(): string {
    return this.value.toUpperCase();
  }

  trim(): string {
    return this.value.trim();
  }

  uncapitalize(): string {
    if (this.value.length === 0) return '';
    return this.value.charAt(0).toLowerCase() + this.value.slice(1);
  }

  valueOf(): string {
    return this.value;
  }

  toString(): string {
    return this.value;
  }
}

// ============================================================================
// List Class
// ============================================================================

export class ApexList<T> extends Array<T> {
  constructor(...items: T[]) {
    super(...items);
    Object.setPrototypeOf(this, ApexList.prototype);
  }

  static of<T>(...items: T[]): ApexList<T> {
    return new ApexList(...items);
  }

  add(item: T): void;
  add(index: number, item: T): void;
  add(indexOrItem: number | T, item?: T): void {
    if (typeof indexOrItem === 'number' && item !== undefined) {
      this.splice(indexOrItem, 0, item);
    } else {
      this.push(indexOrItem as T);
    }
  }

  addAll(items: T[] | Set<T>): void {
    for (const item of items) {
      this.push(item);
    }
  }

  clear(): void {
    this.length = 0;
  }

  clone(): ApexList<T> {
    return new ApexList(...this);
  }

  contains(item: T): boolean {
    return this.includes(item);
  }

  deepClone(): ApexList<T> {
    return new ApexList(...JSON.parse(JSON.stringify(this)));
  }

  get(index: number): T {
    if (index < 0 || index >= this.length) {
      throw new ListException(`List index out of bounds: ${index}`);
    }
    return this[index];
  }

  isEmpty(): boolean {
    return this.length === 0;
  }

  remove(index: number): T {
    if (index < 0 || index >= this.length) {
      throw new ListException(`List index out of bounds: ${index}`);
    }
    return this.splice(index, 1)[0];
  }

  set(index: number, item: T): void {
    if (index < 0 || index >= this.length) {
      throw new ListException(`List index out of bounds: ${index}`);
    }
    this[index] = item;
  }

  size(): number {
    return this.length;
  }

  // Sort with optional comparator
  sortList(compareFn?: (a: T, b: T) => number): void {
    this.sort(compareFn);
  }
}

export class ListException extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ListException';
  }
}

// ============================================================================
// Set Class
// ============================================================================

export class ApexSet<T> extends Set<T> {
  constructor(items?: Iterable<T>) {
    super(items);
  }

  addAll(items: Iterable<T>): void {
    for (const item of items) {
      this.add(item);
    }
  }

  clone(): ApexSet<T> {
    return new ApexSet(this);
  }

  contains(item: T): boolean {
    return this.has(item);
  }

  containsAll(items: Iterable<T>): boolean {
    for (const item of items) {
      if (!this.has(item)) return false;
    }
    return true;
  }

  isEmpty(): boolean {
    return this.size === 0;
  }

  remove(item: T): boolean {
    return this.delete(item);
  }

  removeAll(items: Iterable<T>): void {
    for (const item of items) {
      this.delete(item);
    }
  }

  retainAll(items: Iterable<T>): void {
    const toKeep = new Set(items);
    for (const item of this) {
      if (!toKeep.has(item)) {
        this.delete(item);
      }
    }
  }
}

// ============================================================================
// Map Class
// ============================================================================

export class ApexMap<K, V> extends Map<K, V> {
  constructor(entries?: Iterable<[K, V]>) {
    super(entries);
  }

  clone(): ApexMap<K, V> {
    return new ApexMap(this);
  }

  containsKey(key: K): boolean {
    return this.has(key);
  }

  isEmpty(): boolean {
    return this.size === 0;
  }

  keySet(): ApexSet<K> {
    return new ApexSet(this.keys());
  }

  put(key: K, value: V): V | undefined {
    const old = this.get(key);
    this.set(key, value);
    return old;
  }

  putAll(other: Map<K, V>): void {
    for (const [k, v] of other) {
      this.set(k, v);
    }
  }

  remove(key: K): V | undefined {
    const value = this.get(key);
    this.delete(key);
    return value;
  }
}

// ============================================================================
// Date/DateTime Utilities
// ============================================================================

export class ApexDate {
  private date: Date;

  constructor(date?: Date) {
    this.date = date || new Date();
    this.date.setHours(0, 0, 0, 0);
  }

  static newInstance(year: number, month: number, day: number): ApexDate {
    return new ApexDate(new Date(year, month - 1, day));
  }

  static today(): ApexDate {
    return new ApexDate();
  }

  static valueOf(str: string): ApexDate {
    return new ApexDate(new Date(str));
  }

  addDays(days: number): ApexDate {
    const d = new Date(this.date);
    d.setDate(d.getDate() + days);
    return new ApexDate(d);
  }

  addMonths(months: number): ApexDate {
    const d = new Date(this.date);
    d.setMonth(d.getMonth() + months);
    return new ApexDate(d);
  }

  addYears(years: number): ApexDate {
    const d = new Date(this.date);
    d.setFullYear(d.getFullYear() + years);
    return new ApexDate(d);
  }

  day(): number {
    return this.date.getDate();
  }

  dayOfYear(): number {
    const start = new Date(this.date.getFullYear(), 0, 0);
    const diff = this.date.getTime() - start.getTime();
    return Math.floor(diff / (1000 * 60 * 60 * 24));
  }

  daysBetween(other: ApexDate): number {
    const diff = other.date.getTime() - this.date.getTime();
    return Math.floor(diff / (1000 * 60 * 60 * 24));
  }

  format(): string {
    return this.date.toISOString().split('T')[0];
  }

  isSameDay(other: ApexDate): boolean {
    return this.format() === other.format();
  }

  month(): number {
    return this.date.getMonth() + 1;
  }

  monthsBetween(other: ApexDate): number {
    const months = (other.year() - this.year()) * 12;
    return months + other.month() - this.month();
  }

  toStartOfMonth(): ApexDate {
    return ApexDate.newInstance(this.year(), this.month(), 1);
  }

  toStartOfWeek(): ApexDate {
    const d = new Date(this.date);
    const day = d.getDay();
    d.setDate(d.getDate() - day);
    return new ApexDate(d);
  }

  year(): number {
    return this.date.getFullYear();
  }

  valueOf(): Date {
    return new Date(this.date);
  }

  toString(): string {
    return this.format();
  }
}

export class ApexDateTime {
  private date: Date;

  constructor(date?: Date) {
    this.date = date || new Date();
  }

  static newInstance(year: number, month: number, day: number, hour: number = 0, minute: number = 0, second: number = 0): ApexDateTime {
    return new ApexDateTime(new Date(year, month - 1, day, hour, minute, second));
  }

  static now(): ApexDateTime {
    return new ApexDateTime();
  }

  static valueOf(str: string): ApexDateTime {
    return new ApexDateTime(new Date(str));
  }

  addDays(days: number): ApexDateTime {
    const d = new Date(this.date);
    d.setDate(d.getDate() + days);
    return new ApexDateTime(d);
  }

  addHours(hours: number): ApexDateTime {
    const d = new Date(this.date);
    d.setHours(d.getHours() + hours);
    return new ApexDateTime(d);
  }

  addMinutes(minutes: number): ApexDateTime {
    const d = new Date(this.date);
    d.setMinutes(d.getMinutes() + minutes);
    return new ApexDateTime(d);
  }

  addMonths(months: number): ApexDateTime {
    const d = new Date(this.date);
    d.setMonth(d.getMonth() + months);
    return new ApexDateTime(d);
  }

  addSeconds(seconds: number): ApexDateTime {
    const d = new Date(this.date);
    d.setSeconds(d.getSeconds() + seconds);
    return new ApexDateTime(d);
  }

  addYears(years: number): ApexDateTime {
    const d = new Date(this.date);
    d.setFullYear(d.getFullYear() + years);
    return new ApexDateTime(d);
  }

  date(): ApexDate {
    return new ApexDate(new Date(this.date));
  }

  day(): number {
    return this.date.getDate();
  }

  format(): string {
    return this.date.toISOString();
  }

  formatGmt(formatStr: string): string {
    // Simplified - full implementation would handle format string
    return this.date.toUTCString();
  }

  getTime(): number {
    return this.date.getTime();
  }

  hour(): number {
    return this.date.getHours();
  }

  isSameDay(other: ApexDateTime): boolean {
    return this.date.toDateString() === other.date.toDateString();
  }

  millisecond(): number {
    return this.date.getMilliseconds();
  }

  minute(): number {
    return this.date.getMinutes();
  }

  month(): number {
    return this.date.getMonth() + 1;
  }

  second(): number {
    return this.date.getSeconds();
  }

  year(): number {
    return this.date.getFullYear();
  }

  valueOf(): Date {
    return new Date(this.date);
  }

  toString(): string {
    return this.format();
  }
}

// ============================================================================
// Math Utilities
// ============================================================================

export namespace ApexMath {
  export function abs(n: number): number {
    return Math.abs(n);
  }

  export function acos(n: number): number {
    return Math.acos(n);
  }

  export function asin(n: number): number {
    return Math.asin(n);
  }

  export function atan(n: number): number {
    return Math.atan(n);
  }

  export function atan2(y: number, x: number): number {
    return Math.atan2(y, x);
  }

  export function cbrt(n: number): number {
    return Math.cbrt(n);
  }

  export function ceil(n: number): number {
    return Math.ceil(n);
  }

  export function cos(n: number): number {
    return Math.cos(n);
  }

  export function cosh(n: number): number {
    return Math.cosh(n);
  }

  export function exp(n: number): number {
    return Math.exp(n);
  }

  export function floor(n: number): number {
    return Math.floor(n);
  }

  export function log(n: number): number {
    return Math.log(n);
  }

  export function log10(n: number): number {
    return Math.log10(n);
  }

  export function max(a: number, b: number): number {
    return Math.max(a, b);
  }

  export function min(a: number, b: number): number {
    return Math.min(a, b);
  }

  export function mod(dividend: number, divisor: number): number {
    return dividend % divisor;
  }

  export function pow(base: number, exponent: number): number {
    return Math.pow(base, exponent);
  }

  export function random(): number {
    return Math.random();
  }

  export function rint(n: number): number {
    return Math.round(n);
  }

  export function round(n: number): number {
    return Math.round(n);
  }

  export function roundToLong(n: number): number {
    return Math.round(n);
  }

  export function signum(n: number): number {
    return Math.sign(n);
  }

  export function sin(n: number): number {
    return Math.sin(n);
  }

  export function sinh(n: number): number {
    return Math.sinh(n);
  }

  export function sqrt(n: number): number {
    return Math.sqrt(n);
  }

  export function tan(n: number): number {
    return Math.tan(n);
  }

  export function tanh(n: number): number {
    return Math.tanh(n);
  }
}

// ============================================================================
// JSON Utilities
// ============================================================================

export namespace ApexJSON {
  export function serialize(obj: any): string {
    return JSON.stringify(obj);
  }

  export function serializePretty(obj: any): string {
    return JSON.stringify(obj, null, 2);
  }

  export function deserialize<T>(jsonString: string, apexType?: any): T {
    return JSON.parse(jsonString);
  }

  export function deserializeUntyped(jsonString: string): any {
    return JSON.parse(jsonString);
  }

  export function deserializeStrict<T>(jsonString: string, apexType?: any): T {
    return JSON.parse(jsonString);
  }
}

// ============================================================================
// Export all
// ============================================================================

export {
  System as Apex_System,
  ApexString as Apex_String,
  ApexList as Apex_List,
  ApexSet as Apex_Set,
  ApexMap as Apex_Map,
  ApexDate as Apex_Date,
  ApexDateTime as Apex_DateTime,
  ApexMath as Apex_Math,
  ApexJSON as Apex_JSON,
};
