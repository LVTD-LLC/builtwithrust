const { test } = require('node:test');
const assert = require('node:assert/strict');
const load = () => import('../frontend/privacy.mjs');
test('replay is restricted to query-free public catalog pages', async () => {
  const { replayAllowed } = await load();
  for (const path of ['/', '/categories', '/projects/zed', '/categories/tools']) assert.equal(replayAllowed(new URL('https://builtwithrust.com' + path)), true);
  for (const path of ['/?q=private', '/#secret', '/submit', '/newsletter', '/feature/success?session_id=secret', '/api/admin/projects']) assert.equal(replayAllowed(new URL('https://builtwithrust.com' + path)), false);
});
test('errors retain stack frames but remove free text, request and breadcrumbs', async () => {
  const { cleanEvent } = await load();
  const result = cleanEvent({message:'secret', user:{email:'secret'}, request:{url:'https://x/?email=secret'}, extra:{token:'secret'}, breadcrumbs:[{message:'secret'}], exception:{values:[{type:'TypeError',value:'secret',stacktrace:{frames:[{filename:'https://builtwithrust.com/assets/monitoring.js?v=123',lineno:4,vars:{token:'secret'}}]}}]}});
  assert.equal(JSON.stringify(result).includes('secret'), false);
  assert.equal(result.exception.values[0].stacktrace.frames[0].lineno, 4);
});
test('span sanitizer removes query strings and arbitrary attributes', async () => {
  const { cleanSpan } = await load();
  const result = cleanSpan({op:'http.client',description:'GET https://x/?token=secret',data:{'http.url':'https://x/?token=secret',secret:'secret'},span_id:'abc'});
  assert.equal(JSON.stringify(result).includes('secret'), false);
  assert.equal(result.span_id, 'abc');
});

test('span privacy filter preserves SDK-only profile linkage', async () => {
  const { cleanSpan, cleanEvent } = await load();
  const result = cleanSpan({op:'pageload', data:{'sentry.profiler_id':'0123456789abcdef0123456789abcdef','sentry.profile_id':'abcdef0123456789abcdef0123456789','thread.id':'0','thread.name':'main','http.url':'https://example.com/?token=secret'}});
  assert.equal(result.data['sentry.profiler_id'], '0123456789abcdef0123456789abcdef');
  assert.equal(result.data['thread.id'], '0');
  const event = cleanEvent({contexts:{trace:{data:{...result.data, token:'secret'}}}});
  assert.equal(event.contexts.trace.data['sentry.profiler_id'], result.data['sentry.profiler_id']);
  assert.equal(JSON.stringify(event).includes('secret'), false);
  assert.equal(JSON.stringify(result).includes('secret'), false);
});

test('actual SDK v11 streamed envelopes scrub private URLs and retain linkage', async () => {
  const Sentry = require('@sentry/browser');
  const { cleanSpan, cleanEnvelope } = await load();
  const envelopes = [];
  const client = Sentry.init({dsn:'https://public@sentry.invalid/1',environment:'test',release:'test-release',defaultIntegrations:false,tracesSampleRate:1,beforeSendSpan:cleanSpan,
    transport:() => ({send:envelope => {envelopes.push(envelope);return Promise.resolve({statusCode:200});},flush:() => Promise.resolve(true)})});
  client.on('beforeEnvelope', cleanEnvelope);
  Sentry.startSpan({name:'GET https://example.com/?token=secret',op:'http.client',attributes:{'http.url':'https://example.com/?token=secret','sentry.profiler_id':'0123456789abcdef0123456789abcdef','thread.id':'0'}},()=>{});
  await client.flush();
  const raw = JSON.stringify(envelopes);
  assert.equal(raw.includes('secret'), false);
  assert.equal(raw.includes('0123456789abcdef0123456789abcdef'), true);
  assert.equal(raw.includes('test-release'), true);
  await client.close();
});
