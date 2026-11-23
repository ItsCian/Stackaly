// Thin wrapper around fetch for the Stackaly API.
//
// Every call returns an ApiResult: either { ok: true, data } or
// { ok: false, error }. It never throws, so callers can't forget a
// try/catch: TypeScript forces them to check `ok` before using `data`.

// ── Error codes ─────────────────────────────────────────────────────

// Codes sent by the Rust back end (backend/api/src/errors.rs).
// Keep this list in sync when a variant is added to AppError.
const SERVER_ERROR_CODES = [
  'not_found',
  'validation_error',
  'conflict',
  'invalid_request',
  'internal_error',
] as const;

// Codes the front end produces itself, when there's no usable response
export type ClientErrorCode =
  | 'network_error' // the request never got an answer (offline, API down)
  | 'unknown_error'; // an answer we can't read (not JSON, unexpected code)

export type ServerErrorCode = (typeof SERVER_ERROR_CODES)[number];
export type ApiErrorCode = ServerErrorCode | ClientErrorCode;

export type ApiError = {
  code: ApiErrorCode;
  // HTTP status, or 0 when there was no response at all
  status: number;
  // The back end's English message: for logs and debugging only.
  // The UI shows a translated message built from `code` (see errors.ts).
  message?: string;
};

export type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: ApiError };

// ── Helpers ─────────────────────────────────────────────────────────

// Type guard: after `isServerErrorCode(x)` is true, TypeScript knows
// x is a ServerErrorCode, not just any value
function isServerErrorCode(value: unknown): value is ServerErrorCode {
  return (
    typeof value === 'string' &&
    (SERVER_ERROR_CODES as readonly string[]).includes(value)
  );
}

// Reads an error response body. The back end always sends
// { code, message }, but a proxy or crash page might send HTML instead,
// so nothing here is trusted blindly.
async function readError(response: Response): Promise<ApiError> {
  try {
    const body: unknown = await response.json();
    if (typeof body === 'object' && body !== null && 'code' in body) {
      const { code, message } = body as { code: unknown; message?: unknown };
      if (isServerErrorCode(code)) {
        return {
          code,
          status: response.status,
          message: typeof message === 'string' ? message : undefined,
        };
      }
    }
  } catch {
    // Body wasn't JSON: fall through to unknown_error
  }
  return { code: 'unknown_error', status: response.status };
}

// ── Core request ────────────────────────────────────────────────────

type Method = 'GET' | 'POST' | 'PATCH' | 'DELETE';

async function request<T>(
  method: Method,
  path: string,
  body?: unknown,
): Promise<ApiResult<T>> {
  let response: Response;

  try {
    // Relative URL: the API is on the same origin (Vite proxy in dev,
    // Axum serving both in production), so no host or port here
    response = await fetch(`/api${path}`, {
      method,
      headers:
        body === undefined ? undefined : { 'Content-Type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
      // Send the session cookie. 'same-origin' is fetch's default,
      // written out so the intent is visible
      credentials: 'same-origin',
    });
  } catch {
    // fetch only throws when there's no HTTP response at all
    return { ok: false, error: { code: 'network_error', status: 0 } };
  }

  if (!response.ok) {
    return { ok: false, error: await readError(response) };
  }

  // 204 No Content (e.g. DELETE): there's no body to parse
  if (response.status === 204) {
    return { ok: true, data: undefined as T };
  }

  try {
    return { ok: true, data: (await response.json()) as T };
  } catch {
    // A 2xx response that isn't valid JSON: a back-end bug
    return {
      ok: false,
      error: { code: 'unknown_error', status: response.status },
    };
  }
}

// ── Public API ──────────────────────────────────────────────────────
//
// Usage:
//   const result = await api.get<Company[]>('/companies');
//   if (!result.ok) {
//     toast.error(errorMessage(result.error.code));
//     return;
//   }
//   result.data // typed as Company[]

export const api = {
  get: <T>(path: string) => request<T>('GET', path),
  post: <T>(path: string, body: unknown) => request<T>('POST', path, body),
  patch: <T>(path: string, body: unknown) => request<T>('PATCH', path, body),
  delete: <T = void>(path: string) => request<T>('DELETE', path),
};
