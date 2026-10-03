"""
Example Sentry configuration for Python / FastAPI applications.
Install: pip install sentry-sdk
"""

import os
import sentry_sdk
from sentry_sdk.integrations.fastapi import FastApiIntegration
from sentry_sdk.integrations.starlette import StarletteIntegration

APP_NAME = os.getenv("APP_NAME_SLUG", "my-python-api")
GITHUB_REPO = os.getenv("GITHUB_REPOSITORY", f"https://github.com/ihkaru/{APP_NAME}")
VERIFY_CMD = os.getenv("SRE_VERIFY_CMD", "pytest")
BRANCH = os.getenv("GIT_BRANCH", "main")

sentry_sdk.init(
    # DSN format: https://<public_key>@sre.yourdomain.com/<app_name>
    dsn=os.getenv("SENTRY_DSN", f"https://public@sre.yourdomain.com/{APP_NAME}"),
    integrations=[
        StarletteIntegration(),
        FastApiIntegration(),
    ],
    traces_sample_rate=0.2,
    environment=os.getenv("APP_ENV", "production"),
)

# Set self-describing SRE telemetry tags
with sentry_sdk.configure_scope() as scope:
    scope.set_tag("app_name", APP_NAME)
    scope.set_tag("repository", GITHUB_REPO)
    scope.set_tag("verification_command", VERIFY_CMD)
    scope.set_tag("branch", BRANCH)
