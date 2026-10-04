import type { App } from "vue";
import * as Sentry from "@sentry/vue";
import { makeBrowserOfflineTransport, makeFetchTransport } from "@sentry/browser";

/**
 * Example Sentry initialization for Vue.js / Capacitor Mobile Apps
 * connecting to OpenObserve SRE Gateway with Offline-First IndexedDB Transport
 * and Self-Describing Device & Deployment Telemetry.
 */
export async function initSentry(app: App) {
  const dsn = import.meta.env.VITE_SENTRY_DSN || "https://public@sre.yourdomain.com/my-client-app";
  const release = import.meta.env.VITE_APP_VERSION || "my-client-app@1.0.0";
  const environment = import.meta.env.MODE || "production";

  // Check if running inside native shell (Capacitor / Cordova)
  const isCapacitor = typeof (window as any).Capacitor !== "undefined";
  const isNative = isCapacitor && (window as any).Capacitor.isNativePlatform();

  let platformName = "browser";
  let deviceModel = "desktop";
  let osVersion = "unknown";

  if (isNative) {
    try {
      const { Device } = await import("@capacitor/device");
      const info = await Device.getInfo();
      platformName = info.platform; // 'android' | 'ios'
      deviceModel = info.model;
      osVersion = `${info.operatingSystem} ${info.osVersion}`;
    } catch {
      // Graceful fallback if native bridge not ready
    }
  }

  Sentry.init({
    app,
    dsn,
    release,
    environment,
    // Offline-First Transport: caches envelopes in IndexedDB when offline, auto-flushes on reconnect
    transport: makeBrowserOfflineTransport(makeFetchTransport),
    initialScope: {
      tags: {
        app_name: "my-client-app",
        repository: "https://github.com/your-org/my-client-app",
        verification_command: "npm test",
        branch: "main",
        device_platform: platformName,
        device_model: deviceModel,
        is_native: String(isNative),
      },
    },
    // Filter noise like aborted network calls or pre-mount auth redirects
    beforeSend(event, hint) {
      const error = hint.originalException as Error | undefined;
      if (error?.name === "AbortError") {
        return null;
      }
      return event;
    },
  });

  // Attach rich device context
  Sentry.setContext("device_details", {
    platform: platformName,
    model: deviceModel,
    os: osVersion,
    is_native: isNative,
  });

  // Setup auto-flush on network reconnect & app resume (Capacitor)
  if (isNative) {
    try {
      const { Network } = await import("@capacitor/network");
      const { App: CapApp } = await import("@capacitor/app");

      Network.addListener("networkStatusChange", (status) => {
        if (status.connected) {
          Sentry.getClient()?.getTransport()?.flush(5000);
        }
      });

      CapApp.addListener("appStateChange", (state) => {
        if (state.isActive) {
          Sentry.getClient()?.getTransport()?.flush(5000);
        }
      });
    } catch {
      // Plugins optional
    }
  }
}
