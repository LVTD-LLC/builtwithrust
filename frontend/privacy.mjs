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
export function cleanSpan(span) {
  span.description = span.op || 'operation';
  span.data = Object.fromEntries(Object.entries(span.data || {}).filter(([key]) => ['http.response.status_code','http.request.method','sentry.origin','sentry.op','sentry.profiler_id','sentry.profile_id','thread.id','thread.name'].includes(key)));
  return span;
}
