package main

import (
	"os"
	"path/filepath"
	"strings"
)

var KnownFrameworks = []*Framework{
	{
		Name: "nextjs",
		IdentityFiles: []string{
			"next.config.js",
			"next.config.ts",
			"next.config.mjs",
			"next.config.cjs",
		},
		Commands: []string{"dev", "build", "start"},
	},
	{
		Name:          "nuxt",
		IdentityFiles: []string{"nuxt.config.js", "nuxt.config.ts"},
		Commands:      []string{"dev", "build", "start"},
	},
	{
		Name:                 "angular",
		IdentityFiles:        []string{"angular.json"},
		ProjIdentityKeywords: []string{"@angular"},
		Commands:             []string{"serve", "build"},
	},
	{
		Name: "nestjs",
		DeepDetectionMatchers: []DeepDetectionMatcher{
			{Path: "src/main.ts", Keyword: "@nestjs/common"},
		},
		Commands: []string{"start", "build"},
	},
	{
		Name: "cypress",
		IdentityFiles: []string{
			"cypress.config.ts",
			"cypress.json",
			"cypress.config.js",
			"cypress.config.mjs",
			"cypress.config.cjs",
		},
		Commands: []string{"open-cypress", "e2e"},
	},
	{
		Name: "Vite",
		IdentityFiles: []string{
			"vite.config.ts",
			"vite.config.js",
			"vite.config.mjs",
			"vite.config.cjs",
			"vite.config.mts",
			"vite.config.cts",
		},
		Commands: []string{"serve", "build"},
	},
}

func detectFramework(projectPath string) *Framework {
	for _, framework := range KnownFrameworks {
		// Check identity files
		for _, identityFile := range framework.IdentityFiles {
			if _, err := os.Stat(filepath.Join(projectPath, identityFile)); err == nil {
				return framework
			}
		}

		// Check project.json keywords if any
		if len(framework.ProjIdentityKeywords) > 0 {
			if data, err := os.ReadFile(filepath.Join(projectPath, "project.json")); err == nil {
				content := string(data)
				for _, keyword := range framework.ProjIdentityKeywords {
					if strings.Contains(content, keyword) {
						return framework
					}
				}
			}
		}
	}
	return nil
}

func deepDetectFramework(projectPath string) *Framework {
	for _, framework := range KnownFrameworks {
		for _, matcher := range framework.DeepDetectionMatchers {
			matcherPath := filepath.Join(projectPath, matcher.Path)
			if data, err := os.ReadFile(matcherPath); err == nil {
				if strings.Contains(string(data), matcher.Keyword) {
					return framework
				}
			}
		}
	}
	return nil
}
