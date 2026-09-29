-- ==============================================================================
-- OpenObserve SQL Alert: PHP Fatal Errors & Uncaught Exceptions
-- ==============================================================================
-- Runs every 1 minute over a 2-minute rolling window.
-- Triggers when any PHP fatal error, parse error, or uncaught exception occurs.
-- ==============================================================================

SELECT 
    app_name,
    platform,
    message,
    file,
    _timestamp
FROM "cpanel_apps"
WHERE 
    message LIKE '%PHP Fatal error%' 
    OR message LIKE '%Uncaught Error%' 
    OR message LIKE '%Uncaught Exception%'
    OR message LIKE '%PHP Parse error%'
