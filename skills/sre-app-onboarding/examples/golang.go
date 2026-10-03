package main

/**
 * Example Sentry configuration for Go applications.
 * Install: go get github.com/getsentry/sentry-go
 */

import (
	"log"
	"os"
	"time"

	"github.com/getsentry/sentry-go"
)

func initSentry() {
	appName := getEnv("APP_NAME_SLUG", "my-go-service")
	githubRepo := getEnv("GITHUB_REPOSITORY", "https://github.com/ihkaru/"+appName)
	verifyCmd := getEnv("SRE_VERIFY_CMD", "go test ./...")
	branch := getEnv("GIT_BRANCH", "main")

	dsn := getEnv("SENTRY_DSN", "https://public@sre.yourdomain.com/"+appName)

	err := sentry.Init(sentry.ClientOptions{
		Dsn:              dsn,
		Environment:      getEnv("APP_ENV", "production"),
		TracesSampleRate: 0.2,
	})
	if err != nil {
		log.Fatalf("sentry.Init: %s", err)
	}

	// Configure self-describing SRE telemetry tags
	sentry.ConfigureScope(func(scope *sentry.Scope) {
		scope.SetTag("app_name", appName)
		scope.SetTag("repository", githubRepo)
		scope.SetTag("verification_command", verifyCmd)
		scope.SetTag("branch", branch)
	})
}

func getEnv(key, fallback string) string {
	if val, ok := os.LookupEnv(key); ok {
		return val
	}
	return fallback
}
