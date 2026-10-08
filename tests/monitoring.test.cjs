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
