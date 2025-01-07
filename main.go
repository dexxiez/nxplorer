package main

import (
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

var (
	version = "0.1.0"
	name    = "nxplorer"
)

type Flags struct {
	Help    bool
	Version bool
	Debug   bool
	Verbose bool
	Error   bool
}

func parseFlags() (*Flags, string) {
	flags := &Flags{}
	flag.BoolVar(&flags.Help, "help", false, "Print help menu")
	flag.BoolVar(&flags.Help, "h", false, "Print help menu (shorthand)")
	flag.BoolVar(&flags.Version, "version", false, "Print version")
	flag.BoolVar(&flags.Version, "v", false, "Print version (shorthand)")
	flag.BoolVar(&flags.Debug, "debug", false, "Enable debug mode")
	flag.BoolVar(&flags.Debug, "d", false, "Enable debug mode (shorthand)")
	flag.BoolVar(&flags.Verbose, "verbose", false, "Enable verbose mode")
	flag.BoolVar(&flags.Verbose, "V", false, "Enable verbose mode (shorthand)")
	flag.BoolVar(&flags.Error, "error", false, "Enable error mode")
	flag.BoolVar(&flags.Error, "E", false, "Enable error mode (shorthand)")

	flag.Parse()

	// Get search path (default to current directory)
	searchPath := "."
	if flag.NArg() > 0 {
		searchPath = flag.Arg(0)
	}

	return flags, searchPath
}

func printHelp() {
	fmt.Printf("Usage: %s [options]\n\n", name)
	fmt.Println("Options:")
	fmt.Println("  -h, --help     Print this help menu")
	fmt.Println("  -v, --version  Print the version")
	fmt.Println("  -d, --debug    Enable debug mode")
	fmt.Println("  -V, --verbose  Enable verbose mode")
	fmt.Println("  -E, --error    Enable error mode")
}

func checkNxAvailable(errorEnabled bool) bool {
	if errorEnabled {
		return false
	}

	// Try 'which' command first (Unix systems)
	if _, err := exec.LookPath("nx"); err == nil {
		return true
	}

	// Try 'where' command (Windows)
	cmd := exec.Command("where", "nx")
	return cmd.Run() == nil
}

func checkProjectPaths(searchPath string) error {
	absPath, err := filepath.Abs(searchPath)
	if err != nil {
		return fmt.Errorf("failed to resolve path: %w", err)
	}

	// Check if path exists
	if _, err := os.Stat(absPath); os.IsNotExist(err) {
		return fmt.Errorf("the path %q does not exist", absPath)
	}

	// Check for nx.json
	nxJsonPath := filepath.Join(absPath, "nx.json")
	if _, err := os.Stat(nxJsonPath); os.IsNotExist(err) {
		return fmt.Errorf("the path %q does not appear to be an nx repo", absPath)
	}

	// Check for node_modules
	nodeModulesPath := filepath.Join(absPath, "node_modules")
	if _, err := os.Stat(nodeModulesPath); os.IsNotExist(err) {
		return fmt.Errorf("please install the node_modules in the project root first")
	}

	return nil
}

func printNxInstallInstructions() {
	fmt.Fprintln(os.Stderr, "nx is NOT installed globally. Currently nx is required to be installed globally.")
	fmt.Fprintln(os.Stderr, "Please install nx globally and try again.\n")
	fmt.Fprintln(os.Stderr, "For npm:")
	fmt.Fprintln(os.Stderr, "  npm install -g nx")
	fmt.Fprintln(os.Stderr, "For the package manager with the cat:")
	fmt.Fprintln(os.Stderr, "  yarn global add nx")
	fmt.Fprintln(os.Stderr, "For pnpm:")
	fmt.Fprintln(os.Stderr, "  pnpm add -g nx")
}

func main() {
	flags, searchPath := parseFlags()

	if flags.Help {
		printHelp()
		return
	}

	if flags.Version {
		fmt.Printf("%s %s\n", name, version)
		return
	}

	if flags.Debug {
		fmt.Println("Debug mode enabled, which does nothing lmao gottemmm")
		return
	}

	if !checkNxAvailable(flags.Error) {
		printNxInstallInstructions()
		os.Exit(1)
	}

	if err := checkProjectPaths(searchPath); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	// TODO: Initialize and run TUI
	if err := setupAndRunTUI(searchPath); err != nil {
		fmt.Fprintln(os.Stderr, "Error running TUI:", err)
		os.Exit(1)
	}
}
