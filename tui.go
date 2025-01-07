package main

import (
	"fmt"
	"os/exec"
	"strings"

	"github.com/charmbracelet/bubbles/list"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/sahilm/fuzzy"
)

var (
	titleStyle = lipgloss.NewStyle().
			MarginLeft(2).
			MarginRight(2).
			Padding(0, 1).
			Foreground(lipgloss.Color("205")).
			Bold(true)

	statusStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("240")).
			Padding(0, 1)

	spinnerStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("69"))
)

// Custom item for our list
type item struct {
	cmd CommandEntry
}

func (i item) Title() string       { return i.cmd.DisplayString() }
func (i item) Description() string { return "" }
func (i item) FilterValue() string { return i.cmd.DisplayString() }

// Messages
type projectsLoadedMsg struct {
	projects []*Project
	err      error
}

// Model represents our application state
type model struct {
	list        list.Model
	searchInput textinput.Model
	spinner     spinner.Model
	projects    []*Project
	items       []item
	err         error
	width       int
	height      int
	loading     bool
	searchPath  string
}

func initialModel(searchPath string) model {
	// Set up list
	l := list.New([]list.Item{}, list.NewDefaultDelegate(), 0, 0)
	l.Title = "Loading projects..."
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false) // We handle filtering ourselves

	// Set up search input
	ti := textinput.New()
	ti.Placeholder = "Type to search projects..."
	ti.Focus()

	// Set up loading spinner
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = spinnerStyle

	return model{
		list:        l,
		searchInput: ti,
		spinner:     s,
		loading:     true,
		searchPath:  searchPath,
	}
}

func (m model) Init() tea.Cmd {
	return tea.Batch(
		m.spinner.Tick,
		func() tea.Msg {
			projects, err := DetectProjects(m.searchPath)
			return projectsLoadedMsg{projects: projects, err: err}
		},
	)
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if m.loading {
			return m, nil
		}

		switch msg.String() {
		case "ctrl+c", "q", "esc":
			return m, tea.Quit
		case "ctrl+r":
			exec.Command("nx", "reset").Run()
			return m, tea.Quit
		case "enter":
			if i, ok := m.list.SelectedItem().(item); ok {
				cmd := exec.Command("nx", "run", i.cmd.ToNxCommand())
				cmd.Run()
				return m, tea.Quit
			}
		}

	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height

		// Adjust list height to account for title and search input
		listHeight := m.height - 6 // Magic number: adjust based on other elements
		if listHeight < 0 {
			listHeight = 0
		}

		m.list.SetSize(msg.Width-4, listHeight)

	case projectsLoadedMsg:
		m.loading = false
		if msg.err != nil {
			m.err = msg.err
			return m, nil
		}

		m.projects = msg.projects
		entries := constructCommandEntries(msg.projects)
		items := make([]item, len(entries))
		for i, entry := range entries {
			items[i] = item{cmd: entry}
		}
		m.items = items
		m.list.SetItems(convertToListItems(items))

		return m, nil

	case spinner.TickMsg:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}

	// Handle search input
	if !m.loading {
		var cmd tea.Cmd
		m.searchInput, cmd = m.searchInput.Update(msg)

		// Filter items based on search
		query := m.searchInput.Value()
		filteredItems := m.filterItems(query)
		m.list.SetItems(convertToListItems(filteredItems))

		return m, cmd
	}

	return m, nil
}

func (m model) View() string {
	if m.err != nil {
		return fmt.Sprintf("Error: %v\nPress any key to exit...", m.err)
	}

	var s strings.Builder

	// Title section
	title := fmt.Sprintf("nxplorer %s", version)
	s.WriteString(titleStyle.Render(title) + "\n\n")

	if m.loading {
		s.WriteString(fmt.Sprintf("%s Loading projects...\n", m.spinner.View()))
		return s.String()
	}

	// Status line
	status := fmt.Sprintf("%d projects with %d tasks",
		len(m.projects),
		len(m.items))
	s.WriteString(statusStyle.Render(status) + "\n\n")

	// Search input
	s.WriteString(m.searchInput.View() + "\n\n")

	// Project list
	s.WriteString(m.list.View())

	return s.String()
}

func (m model) filterItems(query string) []item {
	if query == "" {
		return m.items
	}

	matcher := fuzzy.Find(
		strings.ToLower(query),
		convertToStringSlice(m.items),
	)

	filtered := make([]item, len(matcher))
	for i, match := range matcher {
		filtered[i] = m.items[match.Index]
	}

	return filtered
}

// Helper functions
func constructCommandEntries(projects []*Project) []CommandEntry {
	var entries []CommandEntry

	for _, project := range projects {
		frameworkName := ""
		if project.Framework != nil {
			frameworkName = project.Framework.Name
		}

		// Add regular tasks
		for _, task := range project.Tasks {
			// Main command
			entries = append(entries, CommandEntry{
				ProjectType:   project.ProjectType,
				FrameworkName: frameworkName,
				ProjectName:   project.Name,
				Command:       task.Command,
			})

			// Subcommands if any
			for _, subcmd := range task.Subcommands {
				entries = append(entries, CommandEntry{
					ProjectType:   project.ProjectType,
					FrameworkName: frameworkName,
					ProjectName:   project.Name,
					Command:       task.Command,
					Subcommand:    subcmd,
				})
			}
		}

		// Framework-specific commands if available
		if project.Framework != nil {
			for _, cmd := range project.Framework.Commands {
				entries = append(entries, CommandEntry{
					ProjectType:   project.ProjectType,
					FrameworkName: frameworkName,
					ProjectName:   project.Name,
					Command:       cmd,
				})
			}
		}
	}

	return entries
}
func convertToListItems(items []item) []list.Item {
	listItems := make([]list.Item, len(items))
	for i, item := range items {
		listItems[i] = item
	}
	return listItems
}

func convertToStringSlice(items []item) []string {
	strings := make([]string, len(items))
	for i, item := range items {
		strings[i] = item.cmd.DisplayString()
	}
	return strings
}

func setupAndRunTUI(searchPath string) error {
	p := tea.NewProgram(
		initialModel(searchPath),
		tea.WithAltScreen(),
		tea.WithMouseCellMotion(),
	)

	if _, err := p.Run(); err != nil {
		return fmt.Errorf("error running TUI: %w", err)
	}

	return nil
}
