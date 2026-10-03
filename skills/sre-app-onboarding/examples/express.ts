/**
 * Example Sentry configuration for Node.js / Express / TypeScript applications.
 * Install: npm install @sentry/node
 */

import * as Sentry from "@sentry/node";
import express from "express";

const app = express();

Sentry.init({
  // DSN format: https://<public_key>@sre.yourdomain.com/<app_name>
  dsn: process.env.SENTRY_DSN || "https://public@sre.yourdomain.com/my-express-app",
  environment: process.env.NODE_ENV || "production",
  tracesSampleRate: 0.2,

  // Self-describing SRE Telemetry Tags
  initialScope: {
    tags: {
      app_name: process.env.APP_NAME_SLUG || "my-express-app",
      repository: process.env.GITHUB_REPOSITORY || "https://github.com/your-org/my-express-app",
      verification_command: process.env.SRE_VERIFY_CMD || "npm test",
      branch: process.env.GIT_BRANCH || "main",
    },
  },
});

// Setup Sentry express error handler
Sentry.setupExpressErrorHandler(app);

export default app;
