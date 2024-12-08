use super::project::{DeepDetectionMatcher, Framework};

pub const KNOWN_FRAMEWORKS: &[Framework] = &[
    Framework {
        name: "nextjs",
        identity_files: &[
            "next.config.js",
            "next.config.ts",
            "next.config.mjs",
            "next.config.cjs",
        ],
        proj_identity_keywords: &[],
        deep_detection_matchers: &[],
        commands: &["dev", "build", "start"],
    },
    Framework {
        name: "nuxt",
        identity_files: &["nuxt.config.js", "nuxt.config.ts"],
        proj_identity_keywords: &[],
        deep_detection_matchers: &[],
        commands: &["dev", "build", "start"],
    },
    Framework {
        name: "angular",
        identity_files: &["angular.json"],
        proj_identity_keywords: &["@angular"],
        deep_detection_matchers: &[],
        commands: &["serve", "build"],
    },
    Framework {
        name: "nestjs",
        identity_files: &[],
        proj_identity_keywords: &[],
        deep_detection_matchers: &[DeepDetectionMatcher {
            path: "src/main.ts",
            keyword: "@nestjs/common",
        }],
        commands: &["start", "build"],
    },
    Framework {
        name: "cypress",
        identity_files: &[
            "cypress.config.ts",
            &"cypress.json",
            &"cypress.config.js",
            &"cypress.config.mjs",
            &"cypress.config.cjs",
        ],
        proj_identity_keywords: &[],
        deep_detection_matchers: &[],
        commands: &["open-cypress", "e2e"],
    },
    Framework {
        name: "Vite",
        identity_files: &[
            "vite.config.ts",
            "vite.config.js",
            "vite.config.mjs",
            "vite.config.cjs",
            "vite.config.mts",
            "vite.config.cts",
        ],
        proj_identity_keywords: &[],
        deep_detection_matchers: &[],
        commands: &["serve", "build"],
    },
];
