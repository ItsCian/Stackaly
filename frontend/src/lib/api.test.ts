import { afterEach, describe, expect, mock, test } from 'bun:test';
import { api } from './api';

// Replace the real fetch with a fake for each test, so the tests don't
// need the Rust server running
const originalFetch = globalThis.fetch;

function mockFetch(implementation: () => Promise<Response>) {
  const fake = mock(implementation);
  globalThis.fetch = fake as unknown as typeof fetch;
  return fake;
}

// Builds a JSON response, the way Axum would send it
const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe('api', () => {
  test('returns the data on success', async () => {
    mockFetch(async () => json({ status: 'ok' }));

    const result = await api.get<{ status: string }>('/health');

    expect(result).toEqual({ ok: true, data: { status: 'ok' } });
  });

  test('prefixes the path with /api and sends JSON', async () => {
    const fake = mockFetch(async () => json({ id: '1' }, 201));

    await api.post('/companies', { city: 'Namur' });

    const [url, init] = fake.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/companies');
    expect(init.method).toBe('POST');
    expect(init.body).toBe('{"city":"Namur"}');
  });

  test('handles 204 No Content', async () => {
    mockFetch(async () => new Response(null, { status: 204 }));

    const result = await api.delete('/companies/1');

    expect(result.ok).toBe(true);
  });

  test('returns the error code sent by the back end', async () => {
    mockFetch(async () =>
      json({ code: 'conflict', message: 'VAT already used' }, 409),
    );

    const result = await api.post('/companies', {});

    expect(result).toEqual({
      ok: false,
      error: { code: 'conflict', status: 409, message: 'VAT already used' },
    });
  });

  test('returns network_error when fetch throws', async () => {
    mockFetch(async () => {
      throw new TypeError('Failed to fetch');
    });

    const result = await api.get('/health');

    expect(result).toEqual({
      ok: false,
      error: { code: 'network_error', status: 0 },
    });
  });

  test('returns unknown_error for a non-JSON error body', async () => {
    mockFetch(
      async () => new Response('<h1>Bad Gateway</h1>', { status: 502 }),
    );

    const result = await api.get('/health');

    expect(result).toEqual({
      ok: false,
      error: { code: 'unknown_error', status: 502 },
    });
  });

  test('returns unknown_error for a code the front end does not know', async () => {
    mockFetch(async () => json({ code: 'teapot' }, 418));

    const result = await api.get('/health');

    expect(result.ok).toBe(false);
    if (!result.ok) expect(result.error.code).toBe('unknown_error');
  });
});
