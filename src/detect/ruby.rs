//! Ruby project detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct RubyDetector;

impl Detector for RubyDetector {
    fn id(&self) -> &'static str {
        "ruby"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        if !dir.join("Gemfile").exists() {
            return None;
        }
        Some(Project {
            name: dir_name(dir).to_string(),
            root: dir.to_path_buf(),
            kind: ProjectKind::Ruby,
            package_manager: Some("bundler".to_string()),
            scripts: HashMap::new(),
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let cwd = project.root.clone();

        let plan = match verb {
            Verb::Setup => {
                CommandPlan::simple("bundle", ["install"], cwd, "bundle install: Gemfile found")
            }
            Verb::Test => {
                // Use rspec if spec/ directory exists, otherwise rake test
                if cwd.join("spec").is_dir() {
                    CommandPlan::simple(
                        "bundle",
                        ["exec", "rspec"],
                        cwd,
                        "bundle exec rspec (spec/ found)",
                    )
                } else {
                    CommandPlan::simple(
                        "bundle",
                        ["exec", "rake", "test"],
                        cwd,
                        "bundle exec rake test (no spec/ dir)",
                    )
                }
            }
            Verb::Lint => {
                CommandPlan::simple("bundle", ["exec", "rubocop"], cwd, "bundle exec rubocop")
            }
            Verb::Fmt => CommandPlan::simple(
                "bundle",
                ["exec", "rubocop", "-a"],
                cwd,
                "bundle exec rubocop -a",
            ),
            Verb::Dev => {
                // Rails
                if cwd.join("config/routes.rb").exists() || cwd.join("bin/rails").exists() {
                    CommandPlan::simple(
                        "bundle",
                        ["exec", "rails", "server"],
                        cwd,
                        "Rails project: bundle exec rails server",
                    )
                } else {
                    return None;
                }
            }
            Verb::Run(script) => CommandPlan::simple(
                "bundle",
                ["exec", "ruby", script.as_str()],
                cwd,
                "bundle exec ruby",
            ),
            _ => return None,
        };
        Some(plan)
    }

    fn required_tools(&self, _project: &Project) -> Vec<ToolRequirement> {
        vec![
            ToolRequirement {
                tool: "ruby".to_string(),
                version_req: String::new(),
                install_hint: InstallHint {
                    brew: Some("ruby".to_string()),
                    apt: Some("ruby".to_string()),
                    winget: Some("RubyInstallerTeam.RubyWithDevKit.3.2".to_string()),
                    url: Some("https://www.ruby-lang.org/en/downloads/".to_string()),
                },
            },
            ToolRequirement {
                tool: "bundle".to_string(),
                version_req: String::new(),
                install_hint: InstallHint {
                    brew: None,
                    apt: None,
                    winget: None,
                    url: Some("gem install bundler".to_string()),
                },
            },
        ]
    }
}

fn dir_name(dir: &Path) -> &str {
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_ruby() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Gemfile"), "source 'https://rubygems.org'").unwrap();

        let d = RubyDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Ruby);
    }

    #[test]
    fn plan_test_rspec_when_spec_dir_exists() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Gemfile"), "").unwrap();
        fs::create_dir(dir.join("spec")).unwrap();

        let d = RubyDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Test).unwrap();
        assert_eq!(plan.program, "bundle");
        assert!(plan.args.contains(&"rspec".to_string()));
    }

    #[test]
    fn plan_test_rake_when_no_spec_dir() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Gemfile"), "").unwrap();

        let d = RubyDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Test).unwrap();
        assert_eq!(plan.program, "bundle");
        assert!(plan.args.contains(&"rake".to_string()));
        assert!(plan.args.contains(&"test".to_string()));
    }

    #[test]
    fn plan_lint_rubocop() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Gemfile"), "").unwrap();

        let d = RubyDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Lint).unwrap();
        assert_eq!(plan.program, "bundle");
        assert!(plan.args.contains(&"rubocop".to_string()));
    }

    #[test]
    fn no_gemfile_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = RubyDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
