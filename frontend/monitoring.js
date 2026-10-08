import * as Sentry from '@sentry/browser';
import { replayAllowed, routeName, cleanEvent, cleanSpan } from './privacy.mjs';

const config = document.querySelector('meta[name="sentry-dsn"]');
if (config?.content) {
  const route = routeName(location.pathname);
  const replay = replayAllowed(new URL(location.href));
  const integrations = [Sentry.browserTracingIntegration({
    beforeStartSpan: context => ({ ...context, name: route }),
    // Requests are instrumented without collecting their query strings/bodies.
    shouldCreateSpanForRequest: url => !/posthog|sentry|\/api\/admin/.test(url),
  })];
  if (replay) integrations.push(Sentry.browserProfilingIntegration());
  if (replay) integrations.push(Sentry.replayIntegration({
    maskAllText: true, maskAllInputs: true, blockAllMedia: true,
    block: ['script', 'noscript'], networkDetailAllowUrls: [],
    networkCaptureBodies: false,
    beforeAddRecordingEvent: event => event.type === 5 ? null : event,
  }));
  Sentry.init({
    dsn: config.content,
    environment: document.querySelector('meta[name="sentry-environment"]').content,
    release: document.querySelector('meta[name="sentry-release"]').content,
    integrations,
    sendDefaultPii: false,
    // Remove free-text console/DOM breadcrumbs at their source.
    beforeBreadcrumb: () => null,
    tracesSampleRate: 0.2,
    tracePropagationTargets: [/^\/api\/(?!admin)/, /^\/newsletter$/],
    profileSessionSampleRate: 0.2, profileLifecycle: 'trace',
    replaysSessionSampleRate: replay ? 0.1 : 0,
    replaysOnErrorSampleRate: replay ? 1 : 0,
    enableLogs: true,
    beforeSend: cleanEvent,
    beforeSendSpan: cleanSpan,
    beforeSendTransaction: event => {
      cleanEvent(event); event.transaction = route;
      event.spans = (event.spans || []).map(cleanSpan);
      return event;
    },
    beforeSendLog: log => {
      if (log.message !== 'Public page loaded') return null;
      log.attributes = { 'page.route': route };
      return log;
    },
    beforeSendMetric: metric => {
      metric.attributes = { 'page.route': route };
      return metric;
    },
  });
  Sentry.setTag('page.route', route);
  Sentry.logger.info('Public page loaded', { 'page.route': route });
  Sentry.metrics.count('browser.page_loaded', 1, { attributes: { 'page.route': route } });
}
