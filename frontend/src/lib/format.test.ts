import { describe, expect, test } from 'bun:test';
import { formatAmount } from './format';

// French formatting uses invisible special spaces: U+202F between
// thousands, U+00A0 before "€" (the exact characters can vary between
// JavaScript engines). They look like normal spaces but aren't equal to
// them, so we normalise them before comparing.
const normalize = (s: string) => s.replace(/[  ]/g, ' ');

describe('formatAmount', () => {
  describe('French', () => {
    test('formats cents as euros', () => {
      expect(normalize(formatAmount(1250, 'fr'))).toBe('12,50 €');
    });

    test('formats zero', () => {
      expect(normalize(formatAmount(0, 'fr'))).toBe('0,00 €');
    });

    test('formats a negative amount (a shortfall)', () => {
      expect(normalize(formatAmount(-500, 'fr'))).toBe('-5,00 €');
    });

    test('groups thousands', () => {
      expect(normalize(formatAmount(123456, 'fr'))).toBe('1 234,56 €');
    });

    test('keeps single-digit cents', () => {
      // 5 cents → 0,05 €, not 0,5 €
      expect(normalize(formatAmount(5, 'fr'))).toBe('0,05 €');
    });
  });

  describe('English', () => {
    test('uses English separators and symbol position', () => {
      expect(formatAmount(123456, 'en')).toBe('€1,234.56');
    });

    test('formats a negative amount', () => {
      expect(formatAmount(-500, 'en')).toBe('-€5.00');
    });
  });

  describe('invalid input', () => {
    test('rejects a non-integer (euros passed instead of cents)', () => {
      expect(() => formatAmount(12.5, 'fr')).toThrow(RangeError);
    });

    test('rejects NaN', () => {
      expect(() => formatAmount(Number.NaN, 'fr')).toThrow(RangeError);
    });
  });
});
