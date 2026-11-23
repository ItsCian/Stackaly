import type { ApiErrorCode } from '@/lib/api';
import { m } from '@/paraglide/messages.js';

// One translated message per error code.
// Record<ApiErrorCode, …> makes this exhaustive: if a new code is added
// to api.ts, TypeScript reports an error here until it gets a message.
const messages: Record<ApiErrorCode, () => string> = {
  not_found: m.error_not_found,
  validation_error: m.error_validation,
  conflict: m.error_conflict,
  invalid_request: m.error_invalid_request,
  internal_error: m.error_internal,
  network_error: m.error_network,
  unknown_error: m.error_unknown,
};

// Turns an API error code into a message in the user's language.
// Called at render time, so it follows the active Paraglide locale.
export function errorMessage(code: ApiErrorCode): string {
  return messages[code]();
}
