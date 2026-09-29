-- ==============================================================================
-- OpenObserve SQL Alert: Node.js / Docker 500 & Unhandled Exceptions
-- ==============================================================================
-- Monitors coolify_apps stream for uncaught exceptions, fatal errors, or 5xx spikes.
-- ==============================================================================

SELECT 
    app_name,
    platform,
    message,
    level,
    _timestamp
FROM "coolify_apps"
WHERE 
    level = 'error' 
    OR message LIKE '%UnhandledPromiseRejection%' 
    OR message LIKE '%uncaughtException%'
    OR message LIKE '%500 Internal Server Error%'
    OR message LIKE '%FATAL%'
