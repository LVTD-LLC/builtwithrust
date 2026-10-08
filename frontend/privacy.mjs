export function replayAllowed(url) {
  return !url.search && !url.hash && /^(\/|\/categories|\/(projects|categories)\/[a-z0-9-]+)$/.test(url.pathname);
}
export function routeName(path) {
  if (/^\/projects\/[^/]+$/.test(path)) return '/projects/{slug}';
  if (/^\/categories\/[^/]+$/.test(path)) return '/categories/{slug}';
  return ['/', '/categories', '/submit', '/submit/thanks', '/newsletter', '/newsletter/thanks', '/feature', '/feature/success', '/feature/cancel', '/privacy'].includes(path) ? path : '/other';
}
export function cleanEvent(event) {
  delete event.message; delete event.logentry; delete event.user; delete event.request;
  delete event.extra; delete event.breadcrumbs;
  if (event.contexts?.trace?.data) event.contexts.trace.data = cleanSpan({data: event.contexts.trace.data}).data;
  for (const error of event.exception?.values || []) {
    error.value = '[redacted]';
    for (const frame of error.stacktrace?.frames || []) {
      delete frame.vars; delete frame.pre_context; delete frame.post_context; delete frame.context_line;
      for (const key of ['filename', 'abs_path']) if (frame[key]) frame[key] = frame[key].split(/[?#]/)[0];
    }
  }
  return event;
}
const SAFE_SPAN_ATTRIBUTES = new Set([
  'http.response.status_code', 'http.request.method', 'sentry.origin', 'sentry.op',
  'sentry.profiler_id', 'sentry.profile_id', 'thread.id', 'thread.name',
  'sentry.environment', 'sentry.release', 'sentry.sdk.name', 'sentry.sdk.version',
  'sentry.trace_lifecycle', 'sentry.segment.id', 'sentry.status',
  'sentry.client_sample_rate', 'sentry.sample_rate', 'sentry.exclusive_time',
]);
function cleanAttributes(attributes) {
  return Object.fromEntries(Object.entries(attributes || {}).filter(([key]) => SAFE_SPAN_ATTRIBUTES.has(key)));
}
export function cleanSpan(span) {
  // v11 defaults to streamed spans (name/attributes), not legacy description/data.
  if ('name' in span || 'attributes' in span) {
    const original = span.attributes || {};
    span.name = span.is_segment ? routeName(span.name || '') : (original['sentry.op'] || 'operation');
    span.attributes = cleanAttributes(original);
    if (original['sentry.segment.name']) span.attributes['sentry.segment.name'] = routeName(original['sentry.segment.name']);
    delete span.data; delete span.description;
  } else {
    span.description = span.op || 'operation';
    span.data = cleanAttributes(span.data);
  }
  return span;
}

export function cleanEnvelope(envelope) {
  // Dynamic sampling context lives outside span payloads in the envelope header.
  const trace = envelope[0]?.trace;
  if (trace?.transaction) trace.transaction = routeName(trace.transaction);
}
