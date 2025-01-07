package main

import (
	"encoding/json"
	"fmt"
	"github.com/go-git/go-git/v5/plumbing/format/gitignore"
	"io"
	"os"
	"path/filepath"
	"strings"
)

type ProjectType string

const (
	ProjectTypeLibrary     ProjectType = "library"
	ProjectTypeApplication ProjectType = "application"
)

type Task struct {
	Command     string
	Subcommands []string
}

type DeepDetectionMatcher struct {
	Path    string
	Keyword string
}

type Framework struct {
	Name                  string
	IdentityFiles         []string
	ProjIdentityKeywords  []string
	DeepDetectionMatchers []DeepDetectionMatcher
	Commands              []string
}

type Project struct {
	Name        string
	ProjectType ProjectType
	Tasks       []Task
	Framework   *Framework
}

type CommandEntry struct {
	ProjectType   ProjectType
	FrameworkName string
	ProjectName   string
	Command       string
	Subcommand    string
}

func (ce *CommandEntry) DisplayString() string {
	projectTypeStr := "app"
	if ce.ProjectType == ProjectTypeLibrary {
		projectTypeStr = "lib"
	}

	typeDisplay := projectTypeStr
	if ce.FrameworkName != "" {
		typeDisplay = fmt.Sprintf("%s:%s", projectTypeStr, ce.FrameworkName)
	}

	if ce.Subcommand != "" {
		return fmt.Sprintf("[%s] %s:%s:%s", typeDisplay, ce.ProjectName, ce.Command, ce.Subcommand)
	}
	return fmt.Sprintf("[%s] %s:%s", typeDisplay, ce.ProjectName, ce.Command)
}

func (ce *CommandEntry) ToNxCommand() string {
	if ce.Subcommand != "" {
		return fmt.Sprintf("%s:%s:%s", ce.ProjectName, ce.Command, ce.Subcommand)
	}
	return fmt.Sprintf("%s:%s", ce.ProjectName, ce.Command)
}

// findFiles recursively finds all files matching the given names, respecting .gitignore
func findFiles(dir string, targetFiles []string) ([]string, error) {
	var results []string
	var patterns []gitignore.Pattern

	// Load and parse .gitignore if it exists
	gitignorePath := filepath.Join(dir, ".gitignore")
	if f, err := os.Open(gitignorePath); err == nil {
		defer f.Close()
		content, err := io.ReadAll(f)
		if err == nil {
			for _, line := range strings.Split(string(content), "\n") {
				if line = strings.TrimSpace(line); line != "" && !strings.HasPrefix(line, "#") {
					ps := gitignore.ParsePattern(line, []string{})
					patterns = append(patterns, ps)
				}
			}
		}
	}

	matcher := gitignore.NewMatcher(patterns)

	err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Get relative path for gitignore matching
		relPath, err := filepath.Rel(dir, path)
		if err != nil {
			return err
		}

		// Skip if matched by gitignore
		if matcher.Match(strings.Split(relPath, "/"), info.IsDir()) {
			if info.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}

		if !info.IsDir() {
			for _, target := range targetFiles {
				if info.Name() == target {
					results = append(results, path)
				}
			}
		}
		return nil
	})

	return results, err
}

// DetectProjects finds all nx projects in the given directory
func DetectProjects(baseRepoPath string) ([]*Project, error) {
	var projects []*Project

	projectPaths, err := findFiles(baseRepoPath, []string{"project.json"})
	if err != nil {
		return nil, fmt.Errorf("failed to find project files: %w", err)
	}

	for _, projectPath := range projectPaths {
		project, err := parseProjectConfig(projectPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to parse project %s: %v\n", projectPath, err)
			continue
		}

		containingPath := filepath.Dir(projectPath)
		if framework := detectFramework(containingPath); framework != nil {
			project.Framework = framework
		} else if framework := deepDetectFramework(containingPath); framework != nil {
			project.Framework = framework
		}

		projects = append(projects, project)
	}

	return projects, nil
}

func parseProjectConfig(projectPath string) (*Project, error) {
	data, err := os.ReadFile(projectPath)
	if err != nil {
		return nil, err
	}

	var rawConfig map[string]interface{}
	if err := json.Unmarshal(data, &rawConfig); err != nil {
		return nil, err
	}

	name, ok := rawConfig["name"].(string)
	if !ok {
		return nil, fmt.Errorf("missing or invalid 'name' field")
	}

	projectTypeStr, ok := rawConfig["projectType"].(string)
	if !ok {
		projectTypeStr = "application"
	}

	projectType := ProjectTypeApplication
	if projectTypeStr == "library" {
		projectType = ProjectTypeLibrary
	}

	tasks, err := parseTasks(rawConfig)
	if err != nil {
		return nil, err
	}

	return &Project{
		Name:        name,
		ProjectType: projectType,
		Tasks:       tasks,
		Framework:   nil,
	}, nil
}

func parseTasks(rawConfig map[string]interface{}) ([]Task, error) {
	targets, ok := rawConfig["targets"].(map[string]interface{})
	if !ok {
		return nil, nil
	}

	var tasks []Task
	for key, value := range targets {
		task := Task{Command: key}

		if targetMap, ok := value.(map[string]interface{}); ok {
			if configs, ok := targetMap["configurations"].(map[string]interface{}); ok {
				for configKey := range configs {
					task.Subcommands = append(task.Subcommands, configKey)
				}
			}
		}
		tasks = append(tasks, task)
	}

	return tasks, nil
}
