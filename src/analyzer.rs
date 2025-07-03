use crate::config::BackworksConfig;
use crate::error::BackworksResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub blueprint_path: String,
    pub status: AnalysisStatus,
    pub summary: AnalysisSummary,
    pub issues: Vec<AnalysisIssue>,
    pub suggestions: Vec<AnalysisSuggestion>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisStatus {
    Valid,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub endpoints: usize,
    pub runtime_endpoints: usize,
    pub database_endpoints: usize,
    pub transformations: usize,
    pub potential_conflicts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub location: IssueLocation,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Configuration,
    Routing,
    Transformation,
    Performance,
    Security,
    Compatibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLocation {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSuggestion {
    pub title: String,
    pub description: String,
    pub diff: Option<GitDiff>,
    pub priority: SuggestionPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiff {
    pub file_path: String,
    pub original: String,
    pub suggested: String,
    pub line_start: usize,
    pub line_end: usize,
}

pub struct BlueprintAnalyzer;

impl BlueprintAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a blueprint configuration file
    pub async fn analyze_file(&self, blueprint_path: &str) -> BackworksResult<AnalysisReport> {
        info!("🔍 Analyzing blueprint: {}", blueprint_path);
        
        // Load and parse the configuration
        let config_path_buf = std::path::PathBuf::from(blueprint_path);
        let config = match crate::config::load_config(&config_path_buf).await {
            Ok(config) => config,
            Err(e) => {
                return Ok(AnalysisReport {
                    blueprint_path: blueprint_path.to_string(),
                    status: AnalysisStatus::Error,
                    summary: AnalysisSummary::default(),
                    issues: vec![AnalysisIssue {
                        severity: IssueSeverity::Error,
                        category: IssueCategory::Configuration,
                        message: format!("Failed to parse blueprint: {}", e),
                        location: IssueLocation {
                            path: blueprint_path.to_string(),
                            line: None,
                            column: None,
                            context: None,
                        },
                        help: Some("Check YAML syntax and required fields".to_string()),
                    }],
                    suggestions: vec![],
                    recommendations: vec![],
                });
            }
        };

        self.analyze_config(&config, blueprint_path).await
    }

    /// Analyze a loaded configuration
    pub async fn analyze_config(&self, config: &BackworksConfig, blueprint_path: &str) -> BackworksResult<AnalysisReport> {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut recommendations = Vec::new();

        // Generate summary
        let summary = self.generate_summary(config);

        // Run all analysis checks
        self.check_routing_conflicts(config, &mut issues, &mut suggestions);
        self.check_performance_considerations(config, &mut issues, &mut recommendations);
        self.check_security_considerations(config, &mut issues, &mut recommendations);
        self.suggest_improvements(config, &mut suggestions, &mut recommendations);

        // Determine overall status
        let status = if issues.iter().any(|i| matches!(i.severity, IssueSeverity::Error)) {
            AnalysisStatus::Error
        } else if issues.iter().any(|i| matches!(i.severity, IssueSeverity::Warning)) {
            AnalysisStatus::Warning
        } else {
            AnalysisStatus::Valid
        };

        Ok(AnalysisReport {
            blueprint_path: blueprint_path.to_string(),
            status,
            summary,
            issues,
            suggestions,
            recommendations,
        })
    }

    fn generate_summary(&self, config: &BackworksConfig) -> AnalysisSummary {
        let endpoints = config.endpoints.len();
        let mut runtime_endpoints = 0;
        let mut database_endpoints = 0;
        let transformations = 0;

        for endpoint in config.endpoints.values() {
            match endpoint.mode.as_ref().unwrap_or(&config.mode) {
                crate::config::ExecutionMode::Runtime => runtime_endpoints += 1,
                crate::config::ExecutionMode::Database => database_endpoints += 1,
                _ => {}
            }
        }

        AnalysisSummary {
            endpoints,
            runtime_endpoints,
            database_endpoints,
            transformations,
            potential_conflicts: 0, // Will be calculated during analysis
        }
    }

    fn check_routing_conflicts(&self, config: &BackworksConfig, issues: &mut Vec<AnalysisIssue>, suggestions: &mut Vec<AnalysisSuggestion>) {
        let mut path_conflicts = HashMap::new();
        
        // Check for path conflicts
        for (name, endpoint) in &config.endpoints {
            let key = format!("{}:{}", endpoint.methods.join(","), endpoint.path);
            if let Some(existing) = path_conflicts.get(&key) {
                issues.push(AnalysisIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Routing,
                    message: format!("Path conflict: '{}' and '{}' both handle {} {}", 
                        existing, name, endpoint.methods.join(","), endpoint.path),
                    location: IssueLocation {
                        path: "endpoints".to_string(),
                        line: None,
                        column: None,
                        context: Some(format!("endpoint: {}", name)),
                    },
                    help: Some("Each endpoint must have a unique combination of HTTP methods and path".to_string()),
                });
            } else {
                path_conflicts.insert(key, name.clone());
            }
        }

        // Check for ambiguous path patterns
        self.check_path_ambiguity(config, issues, suggestions);
    }

    fn check_path_ambiguity(&self, config: &BackworksConfig, issues: &mut Vec<AnalysisIssue>, suggestions: &mut Vec<AnalysisSuggestion>) {
        let paths: Vec<_> = config.endpoints.iter().collect();
        
        for (i, (name1, endpoint1)) in paths.iter().enumerate() {
            for (name2, endpoint2) in paths.iter().skip(i + 1) {
                if self.paths_may_conflict(&endpoint1.path, &endpoint2.path) {
                    let issue = AnalysisIssue {
                        severity: IssueSeverity::Warning,
                        category: IssueCategory::Routing,
                        message: format!("Potentially ambiguous paths: '{}' and '{}'", endpoint1.path, endpoint2.path),
                        location: IssueLocation {
                            path: "endpoints".to_string(),
                            line: None,
                            column: None,
                            context: Some(format!("endpoints: {}, {}", name1, name2)),
                        },
                        help: Some("Consider using more specific paths or different HTTP methods".to_string()),
                    };
                    issues.push(issue);

                    // Generate suggestion
                    suggestions.push(AnalysisSuggestion {
                        title: "Resolve path ambiguity".to_string(),
                        description: format!("Make paths '{}' and '{}' more distinct", endpoint1.path, endpoint2.path),
                        diff: self.generate_path_disambiguation_diff(name1, &endpoint1.path, name2, &endpoint2.path),
                        priority: SuggestionPriority::Medium,
                    });
                }
            }
        }
    }

    fn paths_may_conflict(&self, path1: &str, path2: &str) -> bool {
        // Simple heuristic for path conflicts
        if path1 == path2 {
            return true;
        }
        
        // Check for wildcard conflicts (simplified)
        let segments1: Vec<&str> = path1.trim_start_matches('/').split('/').collect();
        let segments2: Vec<&str> = path2.trim_start_matches('/').split('/').collect();
        
        if segments1.len() != segments2.len() {
            return false;
        }
        
        for (seg1, seg2) in segments1.iter().zip(segments2.iter()) {
            if seg1.starts_with(':') || seg2.starts_with(':') {
                // Potential parameter conflict
                continue;
            }
            if seg1 != seg2 {
                return false;
            }
        }
        
        true
    }

    fn check_performance_considerations(&self, config: &BackworksConfig, issues: &mut Vec<AnalysisIssue>, recommendations: &mut Vec<String>) {
        let endpoint_count = config.endpoints.len();
        
        if endpoint_count > 50 {
            issues.push(AnalysisIssue {
                severity: IssueSeverity::Info,
                category: IssueCategory::Performance,
                message: format!("High number of endpoints ({})", endpoint_count),
                location: IssueLocation {
                    path: "endpoints".to_string(),
                    line: None,
                    column: None,
                    context: None,
                },
                help: Some("Consider consolidating similar endpoints or using path parameters".to_string()),
            });
            
            recommendations.push("Consider implementing caching for frequently accessed endpoints".to_string());
            recommendations.push("Monitor endpoint performance and optimize as needed".to_string());
        }
    }

    fn check_security_considerations(&self, config: &BackworksConfig, _issues: &mut Vec<AnalysisIssue>, recommendations: &mut Vec<String>) {
        // Check CORS configuration
        if let Some(ref security) = &config.security {
            if let Some(ref cors) = &security.cors {
                if cors.enabled.unwrap_or(false) {
                    if let Some(ref origins) = &cors.origins {
                        if origins.contains(&"*".to_string()) {
                            recommendations.push("Avoid using '*' for CORS origins in production".to_string());
                        }
                    }
                }
            }
        }
        
        // General security recommendations
        recommendations.push("Consider implementing rate limiting for API endpoints".to_string());
        recommendations.push("Enable authentication and authorization for sensitive endpoints".to_string());
    }

    fn suggest_improvements(&self, config: &BackworksConfig, suggestions: &mut Vec<AnalysisSuggestion>, recommendations: &mut Vec<String>) {
        // Suggest adding monitoring
        if config.monitoring.is_none() {
            suggestions.push(AnalysisSuggestion {
                title: "Add monitoring configuration".to_string(),
                description: "Enable monitoring for better observability".to_string(),
                diff: None,
                priority: SuggestionPriority::Medium,
            });
            recommendations.push("Consider adding monitoring and logging configuration".to_string());
        }

        // Suggest adding caching for better performance
        if config.cache.is_none() && config.endpoints.len() > 5 {
            recommendations.push("Consider adding caching configuration for better performance".to_string());
        }
    }

    fn generate_path_disambiguation_diff(&self, name1: &str, path1: &str, _name2: &str, _path2: &str) -> Option<GitDiff> {
        // Generate a simple suggestion to make paths more specific
        Some(GitDiff {
            file_path: "blueprint.yaml".to_string(),
            original: format!("  {}:\n    path: \"{}\"", name1, path1),
            suggested: format!("  {}:\n    path: \"{}/v1\"", name1, path1.trim_end_matches('/')),
            line_start: 1,
            line_end: 2,
        })
    }

    /// Print analysis report in a user-friendly format
    pub fn print_report(&self, report: &AnalysisReport) {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🔍 Blueprint Analysis Report");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 File: {}", report.blueprint_path);
        println!("📊 Status: {}", self.format_status(&report.status));
        println!();

        // Summary
        println!("📈 Summary:");
        println!("   Endpoints: {}", report.summary.endpoints);
        println!("   ├─ Runtime: {}", report.summary.runtime_endpoints);
        println!("   ├─ Runtime: {}", report.summary.runtime_endpoints);
        println!("   └─ Database: {}", report.summary.database_endpoints);
        println!("   Transformations: {}", report.summary.transformations);
        println!();

        // Issues
        if !report.issues.is_empty() {
            println!("⚠️  Issues ({}):", report.issues.len());
            for issue in &report.issues {
                self.print_issue(issue);
            }
            println!();
        }

        // Suggestions
        if !report.suggestions.is_empty() {
            println!("💡 Suggestions ({}):", report.suggestions.len());
            for suggestion in &report.suggestions {
                self.print_suggestion(suggestion);
            }
            println!();
        }

        // Recommendations
        if !report.recommendations.is_empty() {
            println!("🎯 Recommendations:");
            for rec in &report.recommendations {
                println!("   • {}", rec);
            }
            println!();
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    fn format_status(&self, status: &AnalysisStatus) -> String {
        match status {
            AnalysisStatus::Valid => "✅ Valid".to_string(),
            AnalysisStatus::Warning => "⚠️  Warning".to_string(),
            AnalysisStatus::Error => "❌ Error".to_string(),
        }
    }

    fn print_issue(&self, issue: &AnalysisIssue) {
        let severity_icon = match issue.severity {
            IssueSeverity::Error => "❌",
            IssueSeverity::Warning => "⚠️ ",
            IssueSeverity::Info => "ℹ️ ",
            IssueSeverity::Hint => "💡",
        };

        println!("   {} {}", severity_icon, issue.message);
        println!("      └─ {}", issue.location.path);
        if let Some(ref context) = issue.location.context {
            println!("         Context: {}", context);
        }
        if let Some(ref help) = issue.help {
            println!("         Help: {}", help);
        }
        println!();
    }

    fn print_suggestion(&self, suggestion: &AnalysisSuggestion) {
        let priority_icon = match suggestion.priority {
            SuggestionPriority::Critical => "🔥",
            SuggestionPriority::High => "⭐",
            SuggestionPriority::Medium => "💫",
            SuggestionPriority::Low => "✨",
        };

        println!("   {} {}", priority_icon, suggestion.title);
        println!("      {}", suggestion.description);
        
        if let Some(ref diff) = suggestion.diff {
            self.print_diff(diff);
        }
        println!();
    }

    fn print_diff(&self, diff: &GitDiff) {
        println!("      Change in {}:", diff.file_path);
        for line in diff.original.lines() {
            println!("\x1b[31m        - {}\x1b[0m", line);
        }
        for line in diff.suggested.lines() {
            println!("\x1b[32m        + {}\x1b[0m", line);
        }
    }
}

impl Default for AnalysisSummary {
    fn default() -> Self {
        Self {
            endpoints: 0,
            runtime_endpoints: 0,
            database_endpoints: 0,
            transformations: 0,
            potential_conflicts: 0,
        }
    }
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSeverity::Error => write!(f, "error"),
            IssueSeverity::Warning => write!(f, "warning"),
            IssueSeverity::Info => write!(f, "info"),
            IssueSeverity::Hint => write!(f, "hint"),
        }
    }
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueCategory::Configuration => write!(f, "configuration"),
            IssueCategory::Routing => write!(f, "routing"),
            IssueCategory::Transformation => write!(f, "transformation"),
            IssueCategory::Performance => write!(f, "performance"),
            IssueCategory::Security => write!(f, "security"),
            IssueCategory::Compatibility => write!(f, "compatibility"),
        }
    }
}
