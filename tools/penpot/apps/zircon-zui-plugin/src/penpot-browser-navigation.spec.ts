import { withPenpotStartupRetry } from '../tools/penpot-browser-navigation';

describe('Penpot workspace startup recovery', () => {
  const socketTimeout = () => new Error('Timed out waiting for Penpot notifications WebSocket');

  it('opens a fresh session after the notifications connection times out', async () => {
    let attempts = 0;
    const recovered: number[] = [];
    const result = await withPenpotStartupRetry(async () => {
      if (++attempts < 3) throw socketTimeout();
      return 'ready';
    }, (attempt) => recovered.push(attempt));
    expect(result).toBe('ready');
    expect(recovered).toEqual([1, 2]);
  });

  it('keeps persistent startup failure visible after three attempts', async () => {
    let attempts = 0;
    await expect(withPenpotStartupRetry(async () => {
      attempts += 1;
      throw socketTimeout();
    }, () => {})).rejects.toThrow('notifications WebSocket');
    expect(attempts).toBe(3);
  });

  it('does not retry a contract or resource error', async () => {
    let attempts = 0;
    await expect(withPenpotStartupRetry(async () => {
      attempts += 1;
      throw new Error('Unexpected mocked RPC');
    }, () => {})).rejects.toThrow('Unexpected mocked RPC');
    expect(attempts).toBe(1);
  });
});
