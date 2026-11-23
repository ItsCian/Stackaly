// src/components/language-switcher.tsx
import { Button } from '@/components/ui/button';
import { getLocale, locales, setLocale } from '@/paraglide/runtime.js';

export function LanguageSwitcher() {
  // locales = ["fr", "en"], read from settings.json:
  // adding a language there adds a button here automatically
  return (
    <div className="flex gap-1">
      {locales.map((locale) => (
        <Button
          key={locale}
          size="sm"
          // Highlight the active language
          variant={locale === getLocale() ? 'default' : 'ghost'}
          // Writes the PARAGLIDE_LOCALE cookie, then reloads the page
          onClick={() => setLocale(locale)}
        >
          {locale.toUpperCase()}
        </Button>
      ))}
    </div>
  );
}
