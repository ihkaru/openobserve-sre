<?php

/**
 * Example Sentry configuration for Laravel applications connecting to OpenObserve SRE Gateway.
 * Save this file to config/sentry.php and configure your .env
 */

return [
    /*
    |--------------------------------------------------------------------------
    | Sentry DSN
    |--------------------------------------------------------------------------
    |
    | Formatted as: https://<public_key>@sre.yourdomain.com/<app_name>
    | Example: https://public@sre.yourdomain.com/my-app
    |
    */
    'dsn' => env('SENTRY_LARAVEL_DSN', env('SENTRY_DSN')),

    /*
    |--------------------------------------------------------------------------
    | Self-Describing SRE Telemetry Tags (Tier 1 SSOT)
    |--------------------------------------------------------------------------
    |
    | These tags are transmitted in every Sentry envelope and extracted by
    | the OpenObserve SRE Gateway to provide zero-config auto-triage to Aina.
    |
    */
    'tags' => [
        'app_name' => env('APP_NAME_SLUG', 'my-app'),
        'repository' => env('GITHUB_REPOSITORY', 'https://github.com/your-org/my-app'),
        'verification_command' => env('SRE_VERIFY_CMD', 'php artisan test'),
        'branch' => env('GIT_BRANCH', 'main'),
    ],

    /*
    |--------------------------------------------------------------------------
    | Performance Monitoring & Tracing
    |--------------------------------------------------------------------------
    */
    'traces_sample_rate' => (float) env('SENTRY_TRACES_SAMPLE_RATE', 0.2),

    'send_default_pii' => false,
];
