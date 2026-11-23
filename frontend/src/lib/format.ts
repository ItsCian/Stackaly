import { getLocale } from '@/paraglide/runtime.js';

// Creating an Intl.NumberFormat is relatively expensive, and a table of
// amounts can call formatAmount hundreds of times per render. So we keep
// one formatter per locale and reuse it.
const formatters = new Map<string, Intl.NumberFormat>();

function getFormatter(locale: string): Intl.NumberFormat {
  let formatter = formatters.get(locale);
  if (!formatter) {
    formatter = new Intl.NumberFormat(locale, {
      style: 'currency',
      currency: 'EUR',
    });
    formatters.set(locale, formatter);
  }
  return formatter;
}

/**
 * Formats an amount in integer cents as euros for display.
 *
 *   formatAmount(1250, 'fr')  → "12,50 €"
 *   formatAmount(1250, 'en')  → "€12.50"
 *   formatAmount(-500, 'fr')  → "-5,00 €"
 *
 * Amounts travel through the app as integer cents (never floats:
 * 0.1 + 0.2 !== 0.3 in JavaScript). Converting to euros happens only
 * here, at the last moment, and the result is a string for display:
 * never store it or calculate with it.
 *
 * @param cents  Integer amount in cents, as sent by the API
 * @param locale Defaults to the active Paraglide locale
 */
export function formatAmount(cents: number, locale = getLocale()): string {
  // A non-integer here means a bug upstream (e.g. euros sent instead of
  // cents): fail loudly instead of silently showing a wrong amount
  if (!Number.isSafeInteger(cents)) {
    throw new RangeError(
      `formatAmount expects an integer number of cents, got ${cents}`,
    );
  }

  // Dividing by 100 is safe for display: Intl rounds to 2 decimals,
  // so a float artefact like 12.499999999 still prints as 12,50
  return getFormatter(locale).format(cents / 100);
}
